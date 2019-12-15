use std::rc::Rc;
use std::cell::{RefCell, RefMut};
use std::borrow::{BorrowMut, Borrow};
use std::collections::HashMap;
use std::ops::Deref;

use std::mem::size_of_val;
use std::mem::size_of;

#[derive(Debug)]
struct Thing {
    n2: u32,
    n1: String,
}

#[derive(Debug)]
struct C1 {
    base: Vec<Rc<RefCell<Thing>>>,
}

#[derive(Debug)]
struct C2 {
    which:Rc<RefCell<Thing>>,
}


fn main () {
    println!("start");

    let mut tracking = C1 {
        base: Vec::new(),
    };

    for i in 0..10 {
        tracking.base.push(Rc::new(RefCell::new(Thing {
            n1: format!("{}", i),
            n2: i,
//            n3: i*1000,
        })));
    }

    {
        let mut t = tracking.base.get(2).unwrap().as_ref().borrow_mut();
        t.n1.clear();
        t.n1.push_str("reset here");
        t.n2=25;
    }

    let mut c2 = {
        let t = tracking.base.get(4).unwrap();
        C2 {
            which: t.clone(),
        }
    };

    c2.which.as_ref().borrow_mut().n2=1000;


    println!("{:#?}\nsize val tracking: {}\nsize val c2: {}\nsize of C1: {}\nsize of c2: {} ref cell of thing: {}",
             &tracking,
             size_of_val(&tracking),
             size_of_val(&c2),
             size_of::<C1>(),
             size_of::<C2>(),
             size_of::<RefCell<Thing>>());

    println!("size of Thing: {}", size_of::<Thing>());
}

