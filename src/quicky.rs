#![allow(warnings)]
use std::rc::Rc;
use std::cell::{RefCell, RefMut};
use std::borrow::{BorrowMut, Borrow};
use std::collections::{HashMap, BTreeMap};
use std::ops::Deref;

use std::mem::size_of_val;
use std::mem::size_of;
use std::cmp::Ordering;

#[derive(Debug,Eq)]
struct AKey {
    s: String,
}

#[derive(Debug)]
struct AVal {
    v: Vec<u32>,
}

#[derive(Debug)]
struct Contain {
    map: BTreeMap<AKey, AVal>,
}

impl Contain {
//    pub fn look_up<'a>(self: &'a mut Self, key: AKey)
//        -> (bool, &'a mut AVal) {
//
//        let found = self.map.contains_key(&key);
//
//        if !found {
//            let cd = AVal { v: vec![], };
//            self.map.insert(key, cd);
//            let &mut cdr = self.map.get_mut(&key).unwrap();
//            return (found, &mut AVal { v: vec![], });
//        } else {
//            let &mut cd = self.map.get_mut(&key).unwrap();
//            return (found, &mut AVal { v: vec![], });
//        }
//    }

    pub fn add_to_cluch<F>(self: & mut Self, key: &AKey, mut f: F) -> (u64)
        where F: Fn(&mut AVal) -> (u64)
     {
        let val = if self.map.contains_key(&key) {
            self.map.get_mut(&key).unwrap()
        } else {
            self.map.insert(AKey{s: String::from(key.s.as_str())},AVal { v: vec![], });
            self.map.get_mut(&key).unwrap()
        };

        let count = f(val);
        (count)
    }

}

fn main () {
    println!("start");

    let mut key = AKey {s:"test".to_string()};

    let mut c = Contain{map: BTreeMap::new()};
    c.add_to_cluch(&key, |v| {
        v.v.push(32);
        v.v.push(33);
        (2)
    });

    key.s.push_str("more stuff");
    c.add_to_cluch(&key, |v| {
        v.v.push(64);
        v.v.push(65);
        (2)
    });

    println!("{:#?}", c);

}



impl Ord for AKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.s.cmp(&other.s)
    }
}

impl PartialOrd for AKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for AKey {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(&other) == Ordering::Equal
    }
}
