use std::rc::Rc;
use std::collections::{HashMap, BTreeSet, BTreeMap};

use crate::bitset::BitSet;
use std::cell::RefCell;
use std::borrow::Borrow;
use std::cmp::Ordering;

const RES_SLOP: usize = 8;  // re allocate every 8 OMs?
// note that after the metadata matures this reallocation will be rarely done

type GroupIdx = u16;

#[derive(Debug)]
enum OmTypeEnum {
    //    TypeU32(u32,u32),
    TypeU32{id: u32, val:u32},
    TypeI32{id: u32, val:i32},
    TypeU64{id: u32, val:u64},
    TypeI64{id: u32, val:i64},
    TypeF32{id: u32, val:f32},
    TypeF64{id: u32, val:f64},
}


pub mod OmTypeInt {
    pub type OmType = u8;

    pub const TypeU32: OmType = 0;
    pub const TypeI32: OmType = 1;
    pub const TypeU64: OmType = 2;
    pub const TypeI64: OmType = 3;
    pub const TypeF32: OmType = 4;
    pub const TypeF64: OmType = 5;
    pub const TypeString: OmType = 6;
}

#[derive(Debug)]
pub struct OmMeta {
    om_type: OmTypeInt::OmType,
    id: u32,
    slot: u32,
}

impl OmMeta {
    pub fn new(om_type: OmTypeInt::OmType, id: u32, slot: u32) -> Self {
        OmMeta {
            om_type,
            id,
            slot,
        }
    }
}

#[derive(Debug)]
pub struct OmGroup {
    pub idx: GroupIdx,
    pub raw_group: String,
    pub virt_group: String,
    pub om_map: HashMap<u32, OmMeta>,
    pub om32_slots: u32,
    pub om64_slots: u32,
    pub omstr_slots: u32,
}

impl OmGroup {
    fn get_slot(self: &mut Self, id: u32, om_type: OmTypeInt::OmType) -> u32 {
        // add q new meta if it does not exist yet and return the slot to use
        let meta = self.om_map.entry(id).or_insert({
            let slot = match om_type {
                OmTypeInt::TypeU32 | OmTypeInt::TypeI32 | OmTypeInt::TypeF32 => {
                    self.om32_slots
                }
                OmTypeInt::TypeU32 | OmTypeInt::TypeI32 | OmTypeInt::TypeF32 => {
                    self.om64_slots
                }
                _ => panic!("type not implemented yet"),
            };
            self.om32_slots += 1;
            OmMeta {
                id: id,
                om_type: om_type,
                slot: slot,
            }
        });
        meta.slot
    }
//    fn get_slot_enum(self: &mut Self, om: &OmTypeEnumid) -> u32 {
//        // add q new meta if it does not exist yet and return the slot to use
//        let meta = self.om_map.entry(om.id).or_insert({
//            let (slot, OmTypeInt) = match om {
//                OmTypeEnum::TypeU32 {id,val} =>
//                OmTypeInt::TypeU32 | OmTypeInt::TypeI32 | OmTypeInt::TypeF32 => {
//                    self.om32_slots
//                }
//                OmTypeInt::TypeU32 | OmTypeInt::TypeI32 | OmTypeInt::TypeF32 => {
//                    self.om64_slots
//                }
//                _ => panic!("type not implemented yet"),
//            };
//            self.om32_slots += 1;
//            OmMeta {
//                id: id,
//                om_type: om_type,
//                slot: slot,
//            }
//        });
//        meta.slot
//    }



    fn get_slot32(self: &mut Self, id: u32, om_type: OmTypeInt::OmType) -> u32 {
        let meta = self.om_map.entry(id).or_insert({
            self.om32_slots += 1;
            OmMeta::new(om_type, id, self.om32_slots)
        });
        meta.slot
    }
    fn get_slot64(self: &mut Self, id: u32, om_type: OmTypeInt::OmType) -> u32 {
        let meta = self.om_map.entry(id).or_insert({
            self.om64_slots += 1;
            OmMeta::new(om_type, id, self.om64_slots)
        });
        meta.slot
    }
}


#[derive(Debug, Eq)]
pub struct ClutchKey {
    pub groupidx: u16,
    keys: Vec<String>,
    time: u64,
    dur: u32,
    offset: i32,
}

impl ClutchKey {
    pub fn new(groupidx: u16, keys: &[&str], time: u64, dur: u32, offset: i32) -> Self {
        ClutchKey {
            groupidx,
            keys: keys.iter().map(|x| x.to_string()).collect(),
            time,
            dur,
            offset,
        }
    }
    fn createCopy(self: &Self) -> Self {
        ClutchKey {
            groupidx: self.groupidx,
            keys: self.keys.iter().map(|x| x.clone()).collect::<Vec<String>>(),
            time: self.time,
            dur: self.dur,
            offset: self.offset,
        }
    }

    fn newEmpty(self: &Self) -> Self {
        ClutchKey {
            groupidx: 0,
            keys: Vec::new(),
            time: 0,
            dur: 0,
            offset: 0,
        }
    }
}

impl Ord for ClutchKey {
    fn cmp(&self, other: &Self) -> Ordering {
        let d = self.groupidx.cmp(&other.groupidx);
        if d != Ordering::Equal { return d; }
        let d = self.time.cmp(&other.time);
        if d != Ordering::Equal { return d; }
        let d = self.dur.cmp(&other.dur);
        if d != Ordering::Equal { return d; }
        let d = self.offset.cmp(&other.offset);
        if d != Ordering::Equal { return d; }

        let d = self.keys.len().cmp(&other.keys.len());
        if d != Ordering::Equal { return d; }

        for (l, r) in self.keys.iter().zip(other.keys.iter()) {
            let d = l.cmp(r);
            if d != Ordering::Equal {
                return d;
            }
        }
        Ordering::Equal
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


#[derive(Debug)]
pub struct ClutchData {
    om_null32: BitSet,
    om_null64: BitSet,

    om32: Vec<u32>,
    om64: Vec<u64>,
    om_str: Vec<(u32, String)>,
}

impl ClutchData {
    fn new(om32_size: usize, om64_size: usize) -> Self {
        ClutchData {
            om_null32: BitSet::new(),
            om_null64: BitSet::new(),
            om32: Vec::with_capacity(om32_size),
            om64: Vec::with_capacity(om64_size),
            om_str: vec![],
        }
    }

//    pub fn add_om(self: &mut Self, group: &mut OmGroup, om: &OmTypeEnum) {
//        let slot = group.get_slot(om.id, OmTypeInt::TypeU32) as usize;
//        self.om_null32.set(slot, true);
//        if self.om32.len() < slot + 1 {
//            self.om32.resize(slot + 1, 0);
//            self.om32[slot] = val;
//        }
//    }


    pub fn add_om_u32(self: &mut Self, group: &mut OmGroup, id: u32, val: u32) {
        let slot = group.get_slot(id, OmTypeInt::TypeU32) as usize;
        self.om_null32.set(slot, true);
        if self.om32.len() < slot + 1 {
            self.om32.resize(slot + 1, 0);
            self.om32[slot] = val;
        }
    }
    pub fn add_om_i32(self: &mut Self, group: &mut OmGroup, id: u32, val: i32) {
        let slot = group.get_slot(id, OmTypeInt::TypeI32) as usize;
        self.om_null32.set(slot, true);
        if self.om32.len() < slot + 1 {
            self.om32.resize(slot + 1, 0);
            self.om32[slot] = unsafe { std::mem::transmute_copy(&val) };
        }
    }

    pub fn add_om_f32(self: &mut Self, group: &mut OmGroup, id: u32, val: f32) {
        let slot = group.get_slot(id, OmTypeInt::TypeF32) as usize;
        self.om_null32.set(slot, true);
        if self.om32.len() < slot + 1 {
            self.om32.resize(slot + 1, 0);
        }
        self.om32[slot] = unsafe { std::mem::transmute_copy(&val) };
    }
}


#[derive(Debug)]
pub struct ClutchStore {
    groups: Vec<OmGroup>,
    group_map: BTreeMap<String, GroupIdx>,
    clutches: BTreeMap<ClutchKey, ClutchData>,
}


impl ClutchStore {
    pub fn new() -> ClutchStore {
        let mut cs = ClutchStore {
            groups: Vec::new(),
            group_map: BTreeMap::new(),
            clutches: BTreeMap::new(),
        };
        cs.newGroup("BAD_ZERO_GROUP", "BAD_ZERO_LEVEL");
        cs
    }

    pub fn newGroup(self: &mut Self, raw_group: &str, virt_group: &str) -> &mut OmGroup {
        let next_id = self.groups.len() as GroupIdx;
        let g = OmGroup {
            idx: next_id,
            raw_group: String::from(raw_group),
            virt_group: String::from(virt_group),
            om_map: HashMap::new(),
            om32_slots: 0,
            om64_slots: 0,
            omstr_slots: 0,
        };
        self.groups.push(g);
        self.group_map.insert(String::from(raw_group), next_id);
        self.groups.get_mut(next_id as usize)
            .expect(&format!("insert group {} to hopefully {} index, but failed to return", &virt_group, next_id))
    }

    pub fn get_group_by_name(self: &mut Self, virt_name: &str) -> Option<&mut OmGroup> {
        if let Some(idx) = self.group_map.get(virt_name) {
            self.groups.get_mut(*idx as usize)
        } else {
            None
        }
    }

    pub fn get_group_by_idx(self: &mut Self, idx: u16) -> Option<&mut OmGroup> {
        self.groups.get_mut(idx as usize)
    }

    pub fn add_to_cluch<F>(self: &mut Self, key: &ClutchKey, mut f: F) -> (u64)
        where F: FnMut(&ClutchKey, &mut OmGroup, &mut ClutchData) -> (u64)
    {
        let mut group = self.groups.get_mut(key.groupidx as usize).unwrap();
        let val = if self.clutches.contains_key(&key) {
            self.clutches.get_mut(&key).unwrap()
        } else {
            self.clutches.insert(ClutchKey::createCopy(&key),
                                 ClutchData::new(group.om32_slots as usize,
                                                 group.om64_slots as usize));
            self.clutches.get_mut(&key).unwrap()
        };


        let count = f(key, group, val);
        (count)
    }
}
