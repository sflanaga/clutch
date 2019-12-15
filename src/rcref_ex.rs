use std::rc::Rc;
use std::cell::{RefCell, RefMut};
use std::borrow::{BorrowMut, Borrow};
use std::collections::HashMap;
use std::ops::Deref;


#[derive(Debug)]
struct Thing {
    n1: String,
    n2: u32,
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


    println!("{:#?}", &tracking);
}

