#![allow(dead_code)]
#![allow(unused_imports)]

use std::cmp::{Ordering, max};
use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
use bit_vec::BitVec;
use anyhow::{anyhow, Result};
use snafu::Backtrace;

//use crate::bitset::BitSet;
use crate::clutch::OmType::{TypeF64, TypeU32};

pub const RESIZE_INC: usize = 8usize;

type GroupIdx = u16;

#[derive(Debug, Clone)]
pub enum OmType {
    TypeU32 = 1,
    TypeI32 = 2,
    TypeU64 = 3,
    TypeI64 = 4,
    TypeF32 = 5,
    TypeF64 = 6,
    TypeString = 7,
}

impl std::fmt::Display for OmType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            TypeF64 => "f64",
            TypeU32 => "u32",
            String => "str",
            _ => "unknown",
        };
        write!(f,"{}",  &s)
    }
}

#[derive(Debug)]
pub enum OmValue {
    NoMeta,
    NULL,
    U32(u32),
    F64(f64),
    String(String),
}

impl std::fmt::Display for OmValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            OmValue::NoMeta => write!(f, "NO META"),
            OmValue::NULL => write!(f, "NULL"),
            OmValue::F64(v) => write!(f, "{}", v),
            OmValue::U32(v) => write!(f, "{}", v),
            OmValue::String(s) => write!(f, "{}", s),
            //_ => panic!("no display mapping for {}", self),
        }
    }
}


#[derive(Debug)]
pub struct OmMeta {
    pub kind: OmType,
    pub id: u32,
    pub slot: usize,
}

#[derive(Debug)]
pub struct OmGroup {
    pub idx: GroupIdx,
    pub group: String,
    pub om_map: BTreeMap<u32, OmMeta>,
    pub om32_slots: usize,
    pub om64_slots: usize,
    pub omstr_slots: usize,
}

#[derive(Debug, Eq)]
pub struct ClutchKey {
    pub groupidx: u16,
    keys: String,
    time: u64,
    dur: u32,
    offset: i32,
}


#[derive(Debug)]
pub struct ClutchData {
    om_null32: BitVec,
    om_null64: BitVec,

    om32: Vec<u32>,
    om64: Vec<u64>,
    om_str: Vec<(u32, String)>,
}

#[derive(Debug)]
pub struct Stats {
    keys: usize,
    oms: usize,
    resizes: usize,
}

pub static mut CLUTCH_STATS: Stats = Stats {
    keys: 0,
    oms: 0,
    resizes: 0,
};

fn inc_keys() {
    unsafe {
        CLUTCH_STATS.keys += 1;
    }
}

fn inc_oms() {
    unsafe {
        CLUTCH_STATS.oms += 1;
    }
}

fn inc_resizes() {
    unsafe {
        CLUTCH_STATS.resizes += 1;
    }
}

pub fn print_clutch_stats() {
    unsafe { println!("{:?}", &crate::clutch::CLUTCH_STATS); }
}

pub fn clear_stats() {
    unsafe {
        CLUTCH_STATS.keys = 0;
        CLUTCH_STATS.oms = 0;
        CLUTCH_STATS.resizes = 0;
    }
}

#[derive(Debug)]
pub struct ClutchMeta {
    groups: Vec<OmGroup>,
    group_map: BTreeMap<String, GroupIdx>,
}

#[derive(Debug)]
pub struct ClutchStore {
    clutches: BTreeMap<ClutchKey, ClutchData>,
}


impl ClutchKey {
    pub fn new(groupidx: u16, keys: String, time: u64, dur: u32, offset: i32) -> Self {
        ClutchKey {
            groupidx,
            keys,
            time,
            dur,
            offset,
        }
    }
    fn create_copy(self: &Self) -> Self {
         Self::new(self.groupidx,self.keys.to_string(),self.time,self.dur,self.offset)
    }

    pub fn get_mut_key(&mut self) -> &mut String {
        &mut self.keys
    }

    fn new_empty(self: &Self) -> Self {
        ClutchKey {
            groupidx: 0,
            keys: String::new(),
            time: 0,
            dur: 0,
            offset: 0,
        }
    }
}

impl OmGroup {
    #[inline(always)]
    fn find_setup_meta_slot(&mut self, id: u32, kind: &OmType) -> usize {
        match kind {
            TypeU32 => {
                match self.om_map.get(&id) {
                    Some(meta) => meta.slot,
                    None => {
                        let this_slot = self.om32_slots;
                        self.om32_slots += 1; // next slot will be here
                        self.om_map.insert(id, OmMeta { kind: kind.clone(), id: id, slot: this_slot });
                        this_slot
                    }
                }
            }
            TypeF64 => {
                match self.om_map.get(&id) {
                    Some(meta) => meta.slot,
                    None => {
                        let this_slot = self.om64_slots;
                        self.om64_slots += 1; // next slot will be here
                        self.om_map.insert(id, OmMeta { kind: TypeF64, id: id, slot: this_slot });
                        this_slot
                    }
                }
            }
            _ => panic!("TYPE not handled yet"),
        }
    }
}

impl Ord for ClutchKey {
    fn cmp(&self, other: &Self) -> Ordering {

        let d = self.keys.cmp(&other.keys);
        if d != Ordering::Equal { return d; }

        let d = self.groupidx.cmp(&other.groupidx);
        if d != Ordering::Equal { return d; }
        let d = self.time.cmp(&other.time);
        if d != Ordering::Equal { return d; }
        let d = self.offset.cmp(&other.offset);
        if d != Ordering::Equal { return d; }
        let d = self.dur.cmp(&other.dur);
        if d != Ordering::Equal { return d; }

        //eprintln!("error on equal {}  vs  {}", &self.to_string(), &other.to_string());
        //println!("{:?}", backtrace::Backtrace::new());
        Ordering::Equal
    }
}

impl ToString for ClutchKey {
    fn to_string(&self) -> String {
        let s = self.keys.replace('\0', ", ");
        format!("g:{} t:{} d:{} o:{} k:{}", self.groupidx, self.time, self.dur, self.offset, s)
    }
}

impl PartialOrd for ClutchKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ClutchKey {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(&other) == Ordering::Equal
    }
}
impl ClutchMeta {
    pub fn new() -> ClutchMeta {
        let mut cm = ClutchMeta {
            groups: Vec::new(),
            group_map: BTreeMap::new(),
        };
        cm.new_group("BAD_ZERO_GROUP");
        cm
    }
    pub fn new_group(self: &mut Self, group: &str) -> &mut OmGroup {
        let next_id = self.groups.len() as GroupIdx;
        let g = OmGroup {
            idx: next_id,
            group: String::from(group),
            om_map: BTreeMap::new(),
            om32_slots: 0,
            om64_slots: 0,
            omstr_slots: 0,
        };
        self.groups.push(g);
        self.group_map.insert(String::from(group), next_id);
        self.groups.get_mut(next_id as usize)
            .expect(&format!("insert group {} to hopefully {} index, but failed to return", &group, next_id))
    }

    pub fn find_or_new_group(self: &mut Self, group: &str) -> &mut OmGroup {
        if let Some(idx) = self.group_map.get(group) {
//            println!("GRP FOUND");
            self.groups.get_mut(*idx as usize).expect("Inconsistent structure error: found a group in map but not in vector")
        } else {
//            println!("NEW GROUP");
            self.new_group(group)
        }
    }

    pub fn get_group_by_name(self: &mut Self, group: &str) -> Option<&mut OmGroup> {
        if let Some(idx) = self.group_map.get(group) {
            self.groups.get_mut(*idx as usize)
        } else {
            None
        }
    }

    pub fn get_group_by_idx(self: &mut Self, idx: u16) -> Option<&mut OmGroup> {
        self.groups.get_mut(idx as usize)
    }

}
impl ClutchStore {
    pub fn new() -> ClutchStore {
        ClutchStore {
            clutches: BTreeMap::new(),
        }
    }

    pub fn clear_oms(&mut self) {
        self.clutches.clear();
    }

    pub fn add_to_clutch(self: &mut Self, o32: usize, o64: usize, key: &ClutchKey) -> &mut ClutchData {
        let mut add_key = 0;

        let val = self.clutches
            .entry(key.create_copy())
            .or_insert(ClutchData::new(o32, o64));

        // let val = if self.clutches.contains_key(&key) {
        //     inc_keys();
        //     //println!("KEY returning existing {}", key.to_string());
        //     self.clutches.get_mut(&key).unwrap()
        // } else {
        //     add_key += 1;
        //     let newkey = key.create_copy();
        //     //println!("KEY new {}", newkey.to_string());
        //     self.clutches.insert(key.create_copy(),
        //                          ClutchData::new(o32,
        //                                          o64 as usize));
        //     self.clutches.get_mut(&key).unwrap()
        // };

        val
    }

    pub fn clear_data(self: &mut Self) {
        self.clutches.clear();
    }
    pub fn clear_all(self: &mut Self) {
        self.clutches.clear();
    }
}

impl ClutchData {
    fn new(om32_size: usize, om64_size: usize) -> Self {
        let om32 = max(om32_size, RESIZE_INC);
        let om64 = max(om64_size, RESIZE_INC);
        ClutchData {
            om_null32: BitVec::from_elem(om32_size, false),
            om_null64: BitVec::from_elem(om32_size, false),
            om32: vec![0u32; om32],
            om64: vec![0u64; om64],
            om_str: vec![],
        }
    }

    // TODO:  must check that id is NOT mapped 2 two different lost 32 vs 64

    pub fn get_value(&self, meta: &OmMeta) -> OmValue {
        match meta.kind {
            TypeU32 => {
                if self.om_null32[meta.slot] {
                    OmValue::U32(*self.om32.get(meta.slot).unwrap())
                } else {
                    OmValue::NULL
                }
            }
            TypeF64 => {
                if self.om_null64[meta.slot] {
                    let tmp = *self.om64.get(meta.slot).unwrap();
                    OmValue::F64(unsafe { std::mem::transmute::<u64, f64>(tmp) })
                } else {
                    OmValue::NULL
                }
            }
            _ => panic!("error in get value, kind not mapped"),
        }
    }
    #[inline(always)]
    pub fn is_32_set(&mut self, slot: usize) -> bool {
        if self.om_null32.len() < slot+1 {
            false
        } else {
            match self.om_null32.get(slot) {
                None => false,
                Some(b)=> b
            }
        }
    }
    #[inline(always)]
    pub fn is_64_set(&mut self, slot: usize) -> bool {
        if self.om_null64.len() < slot+1 {
            false
        } else {
            match self.om_null64.get(slot) {
                None => false,
                Some(b)=> b
            }
        }
    }
    #[inline(always)]
    pub fn set_64(&mut self, slot: usize) {
        if self.om_null64.len() < slot+1 {
            // println!("64 GROW TO {}", slot+1);
            let growth = ((slot+1 / 32)+1)*32;
            self.om_null64.grow(growth, false);
        }
        // println!("64 set slot {} with len: {}", slot, self.om_null64.len());
        self.om_null64.set(slot, true);
    }

    #[inline(always)]
    pub fn set_32(&mut self, slot: usize) {
        if self.om_null32.len() < slot+1 {
            let growth = ((slot+1 / 32)+1)*32;
            self.om_null32.grow(growth, false);
        }
        // println!("32 set slot {} with len: {}", slot, self.om_null32.len());
        self.om_null32.set(slot, true);
    }

    #[inline(always)]
    pub fn add_om_u32(self: &mut Self, overwrite: bool, group: &mut OmGroup, id: u32, val: u32) -> Result<()> {
        let slot = group.find_setup_meta_slot(id, &TypeU32);

        if !overwrite && self.is_32_set(slot) {
            Err(anyhow!("duplicate u32 OM id: {} val: {}", id,val))
        } else {
            self.set_32(slot);
            if self.om32.len() < slot + 1 {
                inc_resizes();
                self.om32.resize(slot + RESIZE_INC, 0);
            }
            inc_oms();
            self.om32[slot] = val;
            Ok(())
        }
    }
    #[inline(always)]
    pub fn add_om_f64(self: &mut Self, overwrite: bool, group: &mut OmGroup, id: u32, val: f64) -> Result<()> {
        let slot = group.find_setup_meta_slot(id, &TypeF64);

        if !overwrite && self.is_64_set(slot) {
            Err(anyhow!("duplicate f64 OM id: {} val: {}", id,val))
        } else {
            self.set_64(slot);
            if self.om64.len() < slot + 1 {
                inc_resizes();
                self.om64.resize(slot + RESIZE_INC, 0);
            }
            inc_oms();

            unsafe { self.om64[slot] =  std::mem::transmute::<f64, u64>(val) };
            Ok(())
        }
    }
}

pub fn dump(cm: &ClutchMeta, cs: &ClutchStore, first_last: bool) {
    let mut at = 0;
    if cs.clutches.len() == 0 { eprintln!("HEY no clutches here?"); }
    for (ck, cd) in &cs.clutches {
        at += 1;
        if !first_last || at == 1 || at == cs.clutches.len() {
            let g = cm.groups.get(ck.groupidx as usize).unwrap();

            println!("{} {{ group: {} key: {}  time: {} dur: {} os: {}", at, &g.group, ck.keys.replace('\0', ", "), ck.time, ck.dur, ck.offset);
            let mut non_null = 0;
            let mut null = 0;
            for (id, meta) in &g.om_map {
                match cd.get_value(&meta) {
                    OmValue::NULL => null += 1,
                    _ => non_null += 1,
                }
            }

            print!("\tc: {}/{}  ", non_null, null);
            print!("{}",
                   &g.om_map.iter().map(|x|
                       format!("{}:{} {}", x.0, cd.get_value(x.1), &x.1.kind)).
                       collect::<Vec<_>>().join(", "));

            // for (id, meta) in &g.om_map {
            //     print!("{}:{},", id, cd.get_value(&meta));
            // }
            println!("}}");
        }
    }
    let mut meta_e = 0;
    for g in &cm.groups {
        meta_e += g.om_map.len();
    }
    println!("g count: {}  g map entries {}  metas: {}", cm.groups.len(), cm.groups.len(), meta_e);
}
