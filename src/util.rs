use std::{sync, thread, time};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Condvar, Arc, Mutex};
use std::time::{Duration, Instant};
use std::fmt::Display;
use num_format::{Locale, ToFormattedString};
use std::convert::TryInto;
use num_traits::AsPrimitive;
use chrono::{DateTime, Local};
use std::any::Any;

pub fn rate<T: AsPrimitive<f64>>(cnt: T, dur: Duration) -> Box<dyn Display> {
    let cnt_f = cnt.as_();
    let t_secs = dur.as_secs_f64();
    let rate = (cnt_f / t_secs) as usize;
    Box::new(rate.to_formatted_string(&Locale::en))
}

pub fn comma<T: AsPrimitive<u64>>(cnt: T) -> Box<dyn Display> {
    Box::new((cnt.as_()).to_formatted_string(&Locale::en))
}

struct ThreadControl {
    keep_running: Mutex<bool>,
    condstop: Condvar,
}

pub struct PeriodicThread {
    handle: Option<thread::JoinHandle<()>>,
    ctrl: Arc<ThreadControl>,
    dur: Duration,
}

impl PeriodicThread {
    pub fn new(dur: Duration) -> PeriodicThread {
        PeriodicThread {
            handle: None,
            ctrl: Arc::new(ThreadControl {
                keep_running: Mutex::new(false),
                condstop: Condvar::new(),
            }),
            dur,
        }
    }

    pub fn start<F>(&mut self, mut fun: F)
    where F: 'static + Send + FnMut(bool) -> ()
    {
        *self.ctrl.clone().keep_running.lock().unwrap() = true;
        let c_ctrl = self.ctrl.clone();
        let c_dur = self.dur.clone();
        self.handle = Some(thread::spawn(move || {
            while *c_ctrl.keep_running.lock().unwrap() {
                let res = {
                    let mut lck = c_ctrl.keep_running.lock().unwrap();
                    c_ctrl.condstop.wait_timeout(lck, c_dur).unwrap()
                };
                if res.1.timed_out() {
                    // this is effectively redundant with the check in the while
                    fun(false);
                } else {
                    fun(true);
                    break;
                }
            }
            println!("exit thread");
        }));
    }

    pub fn stop(&mut self) {
        *self.ctrl.keep_running.lock().unwrap() = false;
        self.ctrl.condstop.notify_all();
        self.handle
            .take().expect("Called stop on non-running thread")
            .join().expect("Could not join spawned thread");
    }
}

#[test]
fn test_it() {
    let mut timer = PeriodicThread::new(Duration::from_secs(1));
    timer.start(|x| println!("working.... "));

    println!("main thread Feeling sleepy...");
    thread::sleep(time::Duration::from_millis(2000));

    println!("stop and wait...");
    timer.stop();
    println!("fast stop");

    timer.start(|x| println!("working again..."));
    println!("main thread Feeling sleepy...");
    thread::sleep(time::Duration::from_millis(2000));

    println!("stop and wait...");
    timer.stop();
    println!("fast stop");
}

struct Stat {
    name: String,
    stat: Arc<AtomicUsize>,
    verbosity: u32,
    max: usize,
}

impl Stat {
    pub fn new(name: &str, verbosity: u32, max: usize) -> Stat {
        Stat {
            name: name.to_string(),
            stat: Arc::new(AtomicUsize::new(0)),
            verbosity,
            max,
        }
    }
}

pub struct StatTrack {
    stats: Vec<Stat>,
    reset: AtomicBool,
    first: Instant,
    last: Instant,
    last_stats: Vec<usize>,
}

impl StatTrack {
    pub fn new() -> StatTrack {
        StatTrack {
            stats: Vec::new(),
            reset: AtomicBool::new(false),
            first: Instant::now(),
            last: Instant::now(),
            last_stats: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        for (stat, last) in self.stats.iter_mut().zip(self.last_stats.iter_mut()) {
            stat.stat.store(0, Ordering::Relaxed);
            *last = 0usize;
        }
        self.last = Instant::now();
        self.first = Instant::now();
    }

    pub fn add_stat(&mut self, name: &str, verbosity: u32, max: usize) -> Arc<AtomicUsize> {
        let stat = Stat::new(name, verbosity, max);
        self.stats.push(stat);
        self.last_stats.push(0usize);
        self.stats.last().unwrap().stat.clone()
    }
    fn now_str() -> String {
        let dt: DateTime<Local> = Local::now();
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    pub fn print_stats(&mut self, last: bool) {
        let now = Instant::now();
        if last {
            print!("[{}] LAST  ", StatTrack::now_str());
        } else {
            print!("[{}] ", StatTrack::now_str());
        }
        for (a_stat, last_stat) in self.stats.iter().zip(self.last_stats.iter_mut()) {
            let thisval = a_stat.stat.load(Ordering::Relaxed);
            let (diff, dur) = if last {
                (thisval, now - self.first)
            } else {
                (thisval - *last_stat, now - self.last)
            };
            let rate = (diff as f64 / dur.as_secs_f64()) as usize;
            let per:f64 = if a_stat.max > 0 {
                (thisval as f64) / (a_stat.max as f64) * 100.0
            } else {
                -1f64
            };
            *last_stat = thisval;
            match a_stat.verbosity {
                0 => print!("  [{}: {}/s]", &a_stat.name, rate.to_formatted_string(&Locale::en)),
                1 => print!("  [{}: {}, {}/s]", &a_stat.name, thisval.to_formatted_string(&Locale::en),
                            rate.to_formatted_string(&Locale::en)),
                _ => print!("   [{}: {}, {} | {}/s]", &a_stat.name, thisval.to_formatted_string(&Locale::en),
                            diff.to_formatted_string(&Locale::en), rate.to_formatted_string(&Locale::en)),
            }
            if  per > 0f64 {
                print!(" {:.2}%", per);
            }
        }
        self.last = now;
        println!();
    }

    pub fn start(mut self, dur: Duration) -> PeriodicThread {
        let mut t = PeriodicThread::new(dur);
        t.start(move|x| self.print_stats(x));
        t
    }

}


