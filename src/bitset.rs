#![allow(dead_code)]

use std::fmt;

//
// calc some machine specific constants
const fn bytelen_to_shift() -> usize {
    const BYTE_TO_SHIFTTA: [usize;8] = [3,4,0,5,0,0,0,6];
    let x = BYTE_TO_SHIFTTA[std::mem::size_of::<usize>()-1];
    x
}


/*

64 -> 1
63 -> 0

*/


const SHIFT_DIV:usize = bytelen_to_shift();
const IDX_MASK:usize = (1 << bytelen_to_shift())-1;
const MACHINE_BITS:usize=std::mem::size_of::<usize>()*8;
const MACHINE_BYTES:usize=std::mem::size_of::<usize>();

pub struct BitSet {
    vec: Vec<usize>,
}

pub fn print_stuff() {
    println!("div: {}  mask: {}  mach_bits: {}  mach_bytes: {}", SHIFT_DIV, IDX_MASK, MACHINE_BITS, MACHINE_BYTES);
}

impl BitSet {
    pub fn new() -> Self {
        BitSet {
            vec: Vec::new(),
        }
    }

    pub fn with_capacity(len:usize) -> Self {
        BitSet {
            vec: Vec::with_capacity(len>>SHIFT_DIV),
        }
    }

    pub fn set(self: &mut Self, idx: usize, val: bool) {
        let tru_idx = idx >> SHIFT_DIV;
        let tru_bit = 1 << (idx & IDX_MASK);
        // println!("set {}: {} at tru_idx: {}  tru_bit: {:x}", idx, val, tru_idx, tru_bit);
        if tru_idx+1 > self.vec.len() {
            self.vec.reserve_exact(tru_idx+1-self.vec.len());
            self.vec.resize(tru_idx+1, 0);
        }
        if val {
            // cut on
            self.vec[tru_idx] |= tru_bit;
        } else {
            // cut off
            self.vec[tru_idx] &= !tru_bit;
        }
    }

    pub fn get(self: &Self, idx: usize) -> bool {
        let tru_idx = idx >> SHIFT_DIV; // div 64
        let tru_bit = 1 << (idx & IDX_MASK);
        // println!("get at {} tru_idx: {}  tru_bit: {:x}", idx, tru_idx, tru_bit);
        if tru_idx+1 > self.vec.len() {
            return false;
        }
        (self.vec[tru_idx] & tru_bit) > 0
    }

    pub fn sizeof(self: &Self) -> usize {
        println!("cap: {}", self.vec.capacity());
        let s = std::mem::size_of_val(&self.vec) + std::mem::size_of::<usize>() * self.vec.capacity();
        s
    }

    pub fn clear(self: &mut Self) {
        self.vec.iter_mut().for_each( |x| {*x = 0;});
    }
}

impl fmt::Debug for BitSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BitVec {{ {} }}", self.vec.iter().enumerate().map(|(i,x)| format!("{}:{:0width$x}", i, x, width=MACHINE_BYTES*2)).collect::<Vec<String>>().join(", "))
    }
}

#[test]
fn test_bit_set() {


    // ummm do asserts...

    for i in 0..9 {
        let x = 1 << i;


    }


    let mut bv = BitSet::new();
    //bv.push(true);
    //bv.resize(1_000_000,false);

    let tests = [5,63,64,127,128,129, 5000];

    for v in &tests {
        bv.set(*v, true);
    }

    for v in &tests {
        println!("{}: {}", *v, bv.get(*v));

        println!("{}: {}", *v - 5, bv.get(*v - 5));
    }
    println!("{:#?}", &bv);

    let mut bv2 = BitSet::new();
    for i in 0..5000 {
        bv2.set(i, true);
    }
    println!("{:#?}", &bv2);

    for i in 0..5000 {
        if i != 2555 {
            bv2.set(i, false);
        }
    }
    println!("{:#?}", &bv2);

    println!("2555: {}", bv2.get(2555));
    println!("2554: {}  mem: {}  internal mem: {}", bv2.get(2554), std::mem::size_of_val(&bv2), bv2.sizeof());

}