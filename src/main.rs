#![allow(dead_code)]
#![allow(unused_imports)]

use std::fmt::Display;
use std::time::Instant;

use structopt::StructOpt;

use clutch::*;

use crate::cli::Cli;

mod clutch;
mod bitset;
mod util;
mod cli;

fn eval_result<T>(key: &ClutchKey, res: Result<(), T>) -> u32
    where T: Display {
    match res {
        Ok(_) => {
            //println!(" OK ***");
            1
        }
        Err(e) => {
            println!("Error: key: {}  {}", key.to_string(), e);
            0
        }
    }
}

fn main() {
    let cli: Cli = crate::cli::Cli::from_args();

    let mut cs = ClutchStore::new();
    for iteration in 1..=cli.iterations {
        let mut om_count = 0u64;
        let mut row_count = 0u64;
        let start_t = std::time::Instant::now();
        for pass in 1..=cli.iterations {
            //println!("o: {}", o);
            let g = cs.find_or_new_group("level1").idx;
            for k1 in 1..=cli.k1 {
                //println!("i: {}", i);
                for k2 in 1..=cli.k2 {
                    //println!("j: {}", j);
                    for k3 in 1..=cli.k3 {
                        //println!("k: {}", k);
                        let mut v: Vec<String> = vec![];
                        v.push(format!("{}", k1));
                        v.push(format!("{}", k2));
                        v.push(format!("{}", k3));

                        let key = ClutchKey::new(g, v, 1960, 32, 0);
                        let (new_row, new_om_count) = cs.add_to_clutch(&key, |group, data| {
                            let mut count = 0;
                            for om_num in 1..=cli.oms {
                                if cli.types & crate::cli::TU32 > 0 {
                                    let id = om_num + 1 + pass * 10000;
                                    if cli.random_nulls == 0 || k3 % cli.random_nulls == 1 {
                                        if cli.verbose > 1 {
                                            println!("u32 o: {} key: {} d: {}", pass, &key.to_string(), id);
                                        }
                                        count += eval_result(&key, data.add_om_u32(false, group, id, id * 2));
                                    }
                                }
                                if cli.types & crate::cli::TF64 > 0 {
                                    let id = om_num + 1 + pass * 10000 + 100000;
                                    if cli.random_nulls == 0 || k3 % cli.random_nulls == 0 {
                                        if cli.verbose > 1 {
                                            println!("u32 o: {} key: {} d: {}", pass, &key.to_string(), id);
                                        }
                                        count += eval_result(&key, data.add_om_f64(false, group, id, (id * 2) as f64 + 0.25 as f64));
                                    }
                                }
                            }
                            count as u64
                        });
                        om_count += new_om_count;
                        row_count += new_row;
                        if cli.verbose > 2 {
                            dump(&cs, true);
                        }
                    }
                }
            }
        }
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
        print_clutch_stats();

        dump(&cs, !cli.dump_full);
        println!();

        if cli.pause {
            let mut s = String::new();
            std::io::stdin().read_line(&mut s).unwrap();
        }
        let clear_time = Instant::now();
        cs.clear_oms();
        clear_stats();
        println!("cleared in {}", clear_time.elapsed().as_secs_f64());
        println!();
    }
}
