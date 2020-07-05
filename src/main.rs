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
mod bitset;
mod util;
mod cli;


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
    let cli: Cli = crate::cli::Cli::from_args();
    let mut cm = ClutchMeta::new();
    let total_cpu = ProcessTime::now();
    for iteration in 1..=cli.iterations {

        let mut st =  StatTrack::new();
        let max_rows:usize = (&cli.passes * &cli.k1 * &cli.k2 * &cli.k3) as usize;
        let max_oms:usize = max_rows * cli.oms as usize * {
            let mut cnt = 0;
            if cli.types & crate::cli::TU32> 0 { cnt += 1; }
            if cli.types & crate::cli::TF64> 0 { cnt += 1; }
            cnt
        };
        let mut row_stats = st.add_stat("Rows", 1, max_rows);
        let mut om_stats = st.add_stat("OMs", 0, 0);
        let mut ticker = st.start(Duration::from_secs(5));

        let mut cs = ClutchStore::new();

        let mut om_count = 0u64;
        let mut row_count = 0u64;
        let start_t = std::time::Instant::now();
        for pass in 1..=cli.passes {
            //println!("o: {}", o);

            let group = cm.find_or_new_group("level1");
            for k1 in 1..=cli.k1 {
                //println!("i: {}", i);
                for k2 in 1..=cli.k2 {
                    //println!("j: {}", j);
                    for k3 in 1..=cli.k3 {
                        //println!("k: {}", k);

                        let g: &OmGroup = &group;
                        let g_idx = g.idx as u16;
                        let om32 = g.om32_slots;
                        let om64 = g.om64_slots;
                        //let key = ;
                        {
                            let mut v = vec![];
                            v.push(format!("{}", k1));
                            v.push(format!("{}", k2));
                            v.push(format!("{}", k3));

                            let data = cs.add_to_clutch(om32, om64, ClutchKey::new(g_idx, v, 1960, 32, 0));
                            for om_num in 1..=cli.oms {
                                if cli.types & crate::cli::TU32 > 0 {
                                    let id = om_num + 1 + pass * 10000;
                                    if cli.random_nulls == 0 || k3 % cli.random_nulls == 1 {
                                        if cli.verbose > 1 {
                                            //println!("u32 o: {} key: {} d: {}", pass, &vk.to_string(), id);
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
                                            //println!("u32 o: {} key: {} d: {}", pass, &key.to_string(), id);
                                        }
                                        let tc = eval_result( data.add_om_f64(false, group, id, (id * 2) as f64 + 0.25 as f64));
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
        let clear_time = Instant::now();
        cs.clear_oms();
        println!("cleared in {}", clear_time.elapsed().as_secs_f64());
        ticker.stop();
        {
            use crate::util::{comma, rate};
            let dur = start_t.elapsed();
            println!("DONE final count: {} rows & {}/sec || {} oms / {}/sec  {} secs total",
                     comma(row_count),
                     rate(row_count, dur),
                     comma(om_count),
                     rate(om_count, dur),
                     dur.as_secs_f64());
        }
        {
            print_clutch_stats();

            dump(&cm, &cs, !cli.dump_full);
            println!();

            if cli.pause {
                println!("Paused for user input <ENTER>");
                let mut s = String::new();
                std::io::stdin().read_line(&mut s).unwrap();
                println!("Continuing...");
            }
            clear_stats();
            println!();
        }
    }
    println!("\n***  TOTAL CPU: {:.3}", total_cpu.elapsed().as_secs_f64());
}
