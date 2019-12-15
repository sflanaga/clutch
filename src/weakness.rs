use std::rc::Rc;
use std::cell::{RefCell, RefMut};
use std::borrow::{BorrowMut, Borrow};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Mutex;


#[derive(Debug)]
struct Thing {
    n1: String,
    n2: u32,
}

#[derive(Debug)]
struct C1 {
    base: Vec<RefCell<Thing>>,
}

#[derive(Debug)]
struct C2 {
    which:RefCell<Thing>,
}

fn main () {
    println!("start");

    let mut tracking = C1 {
        base: Vec::new(),
    };

    tracking.base.push(RefCell::new(Thing{ n1: "initial".to_string(), n2: 81}));

    {
        let x = tracking.base.get_mut(0).unwrap();
        x.borrow_mut().n1.push_str("more");

    }
//    let mut tracking2 = C2 {
//        which: tracking.base.get(0).unwrap().clone(),
//    };
//
//    tracking2.which.get_mut().n1.push_str("  -- even more");



//    {
//        let mut tracking2 = C2 {
//            which: tracking.base.get(0).unwrap().clone(),
//        };
//        let mut x1 = tracking2.which.clone();
//        if let Some(t) = Rc::get_mut(&mut x1) {
//            println!("adding here");
//            t.n1.push_str("more here");
//        } else {
//            println!("did not get it???");
//        }
//    }


//    if let Some(v) = Rc::get_mut(&mut s) {
//        v.push_str("even more");
//    }

//    println!("{:#?}", &tracking2);
//

    println!("{:#?}", &tracking);

}
