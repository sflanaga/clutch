#![allow(dead_code)]
#![allow(unused_imports)]

mod clutch;
mod bitset;
mod util;


use clutch::*;
use std::fmt::Display;
use std::time::Instant;

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
    let mut cs = ClutchStore::new();
    for iteration in 0..3 {
        let mut om_count = 0u64;
        let mut row_count = 0u64;
        let start_t = std::time::Instant::now();
        for o in 1..=2 {
            //println!("o: {}", o);
            let g = cs.find_or_new_group("level1").idx;
            for i in 0..200 {
                //println!("i: {}", i);
                for j in 0..20 {
                    //println!("j: {}", j);
                    for k in 0..80 {
                        //println!("k: {}", k);
                        let mut v: Vec<String> = vec![];
                        v.push(format!("{}", i));
                        v.push(format!("{}", j));
                        v.push(format!("{}", k));

                        let key = ClutchKey::new(g, v, 1960, 32, 0);
                        let (new_row, new_om_count) = cs.add_to_clutch(&key, |group, data| {
                            let mut count = 0;
                            for v in 0..40 {
                                //let id = v * 10000 + o * 10;
                                let id = v + 1 + o * 10000;
                                //println!("adding o: {}   k: {}|{}|{}  id:{} cnt:{}",o, i,j,k, id, row_count);
                                //println!("1 o: {}  i: {}  j: {}  k: {}  key: {} d: {}", o, i, j, k, &key.to_string(), id+1);

                                count += eval_result(&key, data.add_om_u32(false, group, id + 1, 5000 + o + v));
                                // //println!("2 o: {}  i: {}  j: {}  k: {}  key: {} d: {}", o, i, j, k, &key.to_string(), id+2);
                                // count += eval_result(&key, data.add_om_f64(false, group, id + 2, 5000.1 + (v + o) as f64));
                                // //println!("3 o: {}  i: {}  j: {}  k: {}  key: {} d: {}", o, i, j, k, &key.to_string(), id+3);
                                // count += eval_result(&key, data.add_om_u32(false, group, id + 3, 5000 + 0));
                            }
                            count as u64
                        });
                        om_count += new_om_count;
                        row_count += new_row;
                        //dump(&cs);
                        // if count % 1000000 == 0 || adds % 1000000 == 0 {
                        //     unsafe { println!("{:?}", &clutch::CLUTCH_STATS); }
                        //     println!("count: {}  adds: {}", count, adds);
                        // }
                        //println!("count: {}", count);
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
            let mut s = String::new();
            print_clutch_stats();

        dump(&cs, true);
        println!();
        //println!("{:#?}", &cs);
        std::io::stdin().read_line(&mut s).unwrap();
        let clear_time = Instant::now();
        cs.clear_oms();
        clear_stats();
        println!("cleared in {}", clear_time.elapsed().as_secs_f64());
        println!("\n\n");
    }
}
