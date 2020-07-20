use std::fmt::Write;
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
use cpu_time::{ProcessTime, ThreadTime};

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
            //println!("exit thread");
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
    name: String,
    stats: Vec<Stat>,
    reset: AtomicBool,
    first: Instant,
    last: Instant,
    last_stats: Vec<usize>,
    proc_time: ProcessTime,
}

impl StatTrack {
    pub fn new(name: &str) -> StatTrack {
        StatTrack {
            name: name.into(),
            stats: Vec::new(),
            reset: AtomicBool::new(false),
            first: Instant::now(),
            last: Instant::now(),
            last_stats: Vec::new(),
            proc_time: ProcessTime::now(),
        }
    }

    pub fn reset(&mut self) {
        for (stat, last) in self.stats.iter_mut().zip(self.last_stats.iter_mut()) {
            stat.stat.store(0, Ordering::Relaxed);
            *last = 0usize;
        }
        self.last = Instant::now();
        self.first = Instant::now();
        self.proc_time = ProcessTime::now();
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
        let mut buff= String::with_capacity(256);
        write!(&mut buff, "{} [{}] ", self.name, StatTrack::now_str());
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
                0 => write!(&mut buff, "  [{}: {}/s]", &a_stat.name, rate.to_formatted_string(&Locale::en)),
                1 => write!(&mut buff, "  [{}: {}, {}/s]", &a_stat.name, thisval.to_formatted_string(&Locale::en),
                            rate.to_formatted_string(&Locale::en)),
                _ => write!(&mut buff, "   [{}: {}, {} | {}/s]", &a_stat.name, thisval.to_formatted_string(&Locale::en),
                            diff.to_formatted_string(&Locale::en), rate.to_formatted_string(&Locale::en)),
            };
            if  per > 0f64 {
                write!(&mut buff, " {:.2}%", per);
            }
            add_mem_stats(&mut buff);
        }
        if last {
            println!("{} -- LAST cpu time: {:.3}", &buff, self.proc_time.elapsed().as_secs_f64());
        } else {
            println!("{}", &buff);
        }
        self.last = now;
    }

    pub fn start(mut self, dur: Duration) -> Option<PeriodicThread> {
        if dur.as_millis() > 0 {
            let mut t = PeriodicThread::new(dur);
            t.start(move |x| self.print_stats(x));
            Some(t)
        } else {
            None
        }
    }

}


#[cfg(target_family = "unix")]
use jemalloc_ctl::{stats, epoch};

#[cfg(target_family = "unix")]
fn add_mem_stats(s: &mut String) {
    epoch::advance().unwrap();
    s.push_str(&format!(" mem: {}/{}", mem_metric_digit(stats::resident::read().unwrap(),4),
                        mem_metric_digit(stats::active::read().unwrap(), 4)));
}

#[cfg(target_family = "windows")]
fn add_mem_stats(s: &mut String) {
    // do nothing on windows... maybe later?
}


fn mem_metric<'a>(v: usize) -> (f64, &'a str) {
    const METRIC: [&str; 8] = ["B ", "KB", "MB", "GB", "TB", "PB", "EB", "ZB"];

    let mut size = 1usize << 10;
    for e in &METRIC {
        if v < size {
            return ((v as f64 / (size >> 10) as f64) as f64, e);
        }
        size <<= 10;
    }
    (v as f64, "")
}

/// keep only a few significant digits of a simple float value
fn sig_dig(v: f64, digits: usize) -> String {
    let x = format!("{}", v);
    let mut d = String::new();
    let mut count = 0;
    let mut found_pt = false;
    for c in x.chars() {
        if c != '.' {
            count += 1;
        } else {
            if count >= digits {
                break;
            }
            found_pt = true;
        }

        d.push(c);

        if count >= digits && found_pt {
            break;
        }
    }
    d
}

pub fn mem_metric_digit(v: usize, sig: usize) -> String {
    if v == 0 || v > std::usize::MAX / 2 {
        return format!("{:>width$}", "unknown", width = sig + 3);
    }
    let vt = mem_metric(v);
    format!("{:>width$} {}", sig_dig(vt.0, sig), vt.1, width = sig + 1, )
}
