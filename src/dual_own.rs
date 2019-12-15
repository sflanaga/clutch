use std::rc::Rc;
use std::collections::{HashMap, BTreeSet, BTreeMap};

use std::cell::RefCell;
use std::borrow::Borrow;




#[derive(Debug)]
struct Storage {
    s1: Vec<Rc<String>>,
    s2: HashMap<String, Rc<String>>,
}

impl Storage {
    fn add_string(self: &mut Self, s: &str) {
        let ss = Rc::new(s.to_string());
        self.s2.insert(s.to_string(), ss.clone());
        self.s1.push(ss.clone());
    }

    fn add_to_string(self: &mut Self, s: &str) {
        let mut v = self.s2.get_mut(s).unwrap().;
        v.push_str(s);
    }
}

fn main() {
    let mut store = Storage {
        s1: Vec::new(),
        s2: HashMap::new(),
    };

    store.add_string("testing 1 2 3");

    println!("{:#?}", store);
}