#![allow(dead_code)]
#![allow(unused_imports)]

use std::fmt::Display;
use std::time::{Instant, Duration};

use structopt::StructOpt;

use clutch::*;

use crate::cli::Cli;
use std::rc::Rc;
use crate::util::{StatTrack, PeriodicThread};
use std::sync::atomic::Ordering;
use cpu_time::ProcessTime;

mod clutch;
mod util;
mod cli;


#[cfg(target_family = "unix")]
use jemallocator::Jemalloc;

use std::sync::Arc;
use std::thread::spawn;

#[cfg(target_family = "unix")]
#[global_allocator]
pub static GLOBAL_TRACKER: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn eval_result<T>(res: Result<(), T>) -> u64
    where T: Display {
    match res {
        Ok(_) => {
            //println!(" OK ***");
            1
        }
        Err(e) => {
            println!("Error: {}", e);
            0
        }
    }
}

fn main() {
    if let Err(err) = run_test() {
        eprintln!("error: {}", &err);
        std::process::exit(1);
    }
}

fn run_test() -> Result<(), Box<dyn std::error::Error>> {
    let cli= Arc::new(crate::cli::Cli::from_args());
    let mut v = vec![];
    for n in 0..cli.threads {
        let cli = cli.clone();
        let h = spawn(move || {
            match clutch_perf_test(n, cli) {
                Err(e) => println!("error: {}", e),
                _ => {},
            }
        });
        v.push(h);
    }
    for x in v {
        x.join().unwrap();
    }
    Ok(())
}

fn clutch_perf_test(n: u32, cli: Arc<Cli>) -> Result<(), Box<dyn std::error::Error>> {
    let mut cm = ClutchMeta::new();
    let total_cpu = ProcessTime::now();
    for iteration in 1..=cli.iterations {
        let mut st = StatTrack::new(&n.to_string());
        let max_rows: usize = (&cli.passes * &cli.k1 * &cli.k2 * &cli.k3) as usize;
        let max_oms: usize = max_rows * cli.oms as usize * {
            let mut cnt = 0;
            if cli.types & crate::cli::TU32 > 0 { cnt += 1; }
            if cli.types & crate::cli::TF64 > 0 { cnt += 1; }
            cnt
        };
        let mut row_stats = st.add_stat("Rows", 1, max_rows);
        let mut om_stats = st.add_stat("OMs", 0, 0);
        let mut ticker = st.start(Duration::from_millis(cli.interval_ms));

        let mut cs = ClutchStore::new();

        let mut om_count = 0u64;
        let mut row_count = 0u64;
        let start_t = std::time::Instant::now();
        let group = cm.find_or_new_group("level1");
        let mut c_key = ClutchKey::new(group.idx, String::with_capacity(32), 1960, 32, 0);
        for pass in 1..=cli.passes {
            //println!("o: {}", o);
            for k1 in 1..=cli.k1 {
                //println!("i: {}", i);
                for k2 in (1..=cli.k2).rev() {
                    //println!("j: {}", j);
                    for k3 in 1..=cli.k3 {
                        //println!("k: {}", k);

                        let g: &OmGroup = &group;
                        let g_idx = g.idx as u16;
                        let om32 = g.om32_slots;
                        let om64 = g.om64_slots;
                        //let key = ;
                        {
                            use std::io::Write;
                            use std::fmt::Write as FmtWrite;
                            use std::io::Write as IoWrite;
                            c_key.get_mut_key().clear();
                            write!(&mut c_key.get_mut_key(), "{}\0{}\0{}", k1,k2,k3)?;

                            // let mut v = vec![];
                            // v.push(format!("{}", k1));
                            // v.push(format!("{}", k2));
                            // v.push(format!("{}", k3));

                            let data = cs.add_to_clutch(om32, om64, &c_key);
                            for om_num in 1..=cli.oms {
                                if cli.types & crate::cli::TU32 > 0 {
                                    let id = om_num + 1 + pass * 10000;
                                    if cli.random_nulls == 0 || k3 % cli.random_nulls == 1 {
                                        if cli.verbose > 1 {
                                            println!("u32 o: {} key: {} d: {}", pass, &c_key.to_string(), id);
                                        }
                                        let tc = eval_result(data.add_om_u32(false, group, id, id * 2));
                                        om_stats.fetch_add(tc as usize, Ordering::Relaxed);
                                        om_count += tc;
                                    }
                                }
                                if cli.types & crate::cli::TF64 > 0 {
                                    let id = om_num + 1 + pass * 10000 + 100000;
                                    if cli.random_nulls == 0 || k3 % cli.random_nulls == 0 {
                                        if cli.verbose > 1 {
                                            println!("f64 o: {} key: {} d: {}", pass, &c_key.to_string(), id);
                                        }
                                        let tc = eval_result(data.add_om_f64(false, group, id, (id * 2) as f64 + 0.25 as f64));
                                        om_stats.fetch_add(tc as usize, Ordering::Relaxed);
                                        om_count += tc;
                                    }
                                }
                            }
                            row_stats.fetch_add(1, Ordering::Relaxed);
                            row_count += 1;
                        }
                    }
                }
            }
        } // pass loop
        if cli.dump_level > 0 {
            dump(&cm, &cs, !(cli.dump_level > 1));
        }
        let clear_time = Instant::now();
        cs.clear_oms();
        println!("cleared in {}", clear_time.elapsed().as_secs_f64());
        ticker.stop();
        {
            use crate::util::{comma, rate};
            let dur = start_t.elapsed();
            println!("DONE #{} final count: {} rows & {}/sec || {} oms / {}/sec  {} secs total",
                     iteration,
                     comma(row_count),
                     rate(row_count, dur),
                     comma(om_count),
                     rate(om_count, dur),
                     dur.as_secs_f64());
        }
        {
            print_clutch_stats();


            if cli.pause {
                println!("Paused for user input <ENTER>");
                let mut s = String::new();
                std::io::stdin().read_line(&mut s).unwrap();
                println!("Continuing...");
            }
            clear_stats();
            println!();
        }
    } // iteration loop
    println!("\n***  TOTAL CPU: {:.3}", total_cpu.elapsed().as_secs_f64());
    Ok(())
}
