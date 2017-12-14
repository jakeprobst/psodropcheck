use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::process::Command;
use std::collections::{BTreeMap, BTreeSet};
use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use std::cmp;
use item;

const DROPPOINTER: u64 = 0x00A8D8A4;// ephinea 1.9.1
const DROPSTEP: usize = 0x24;
const AREASTEP: u64 =  0x1B00;
const AREACOUNT: u64 = 18;
const MAXITEMS: u64 = 150;

const PMTPOINTER: u64 = 0x00A8DC94;
const PMTWEPOFFSET: u64 = 0x00;
const PMTARMOROFFSET: u64 = 0x04;
const PMTUNITOFFSET: u64 = 0x08;
const PMTTOOLOFFSET: u64 = 0x0C;
const PMTMAGOFFSET: u64 = 0x10;

const UNITTXTPOINTER: u64 = 0x00A9CD50;
const SPECIALTXTPOINTER: u64 = 0x005E4CBB;

#[derive(Debug)]
pub enum ItemType {
    Weapon(item::Weapon),
    Armor(item::Armor),
    Shield(item::Shield),
    Misc(item::Misc),
    Mag(item::Mag),
    Tech(item::Tech),
}

#[derive(Debug)]
pub enum DropChange {
    Add(u32, ItemType),
    Remove(u32)
}

pub struct ItemDrop {
    pid: String,
    seen: BTreeSet<DropBytes>,
}

fn read_u32(file: &mut File, addr: u64) -> u64 {
    file.seek(SeekFrom::Start(addr)).unwrap();
    file.read_u32::<LittleEndian>().unwrap() as u64
}

fn escape_string(s: String) -> String {
    s.replace("&", "&amp;")
}

impl ItemDrop {
    pub fn new() -> ItemDrop {
        ItemDrop {
            pid: ItemDrop::psopid(),
            seen: BTreeSet::new(),
        }
    }
    
    fn psopid() -> String {
        let cmd = Command::new("pgrep").arg("psobb.exe").output().ok().expect("aaa");
        let s = String::from_utf8_lossy(cmd.stdout.as_slice());
        return String::from(s.trim());
    }

    fn pmt_item_id(&self, itype: u8, group: u8, index: u8) -> u64 {
        let mut f = File::open(format!("/proc/{}/mem", self.pid)).unwrap();
        let pmtaddr = read_u32(&mut f, PMTPOINTER);

        if itype == 0 {
            let wepaddr = read_u32(&mut f, pmtaddr + PMTWEPOFFSET);
            let groupaddr = wepaddr + 8 * group as u64;
            let itemaddr = read_u32(&mut f, groupaddr + 4);

            let idaddr = itemaddr as u64 + 44 * index as u64;
            return read_u32(&mut f, idaddr)
        }
        else if itype == 1 {
            if group == 1 || group == 2 {
                let armoraddr = read_u32(&mut f, pmtaddr + PMTARMOROFFSET);

                let groupaddr = armoraddr + 8 * (group as u64 - 1);
                let itemaddr = read_u32(&mut f, groupaddr + 4);
                
                let idaddr = itemaddr as u64 + 32 * index as u64;
                return read_u32(&mut f, idaddr)
            }
            else if group == 3 {
                let unitaddr = read_u32(&mut f, pmtaddr + PMTUNITOFFSET);
                let itemaddr = read_u32(&mut f, unitaddr + 4);
                
                let idaddr = itemaddr as u64 + 20 * index as u64;
                return read_u32(&mut f, idaddr)
            }
        }
        else if itype == 2 {
            let magaddr = read_u32(&mut f, pmtaddr + PMTMAGOFFSET);
            let itemaddr = read_u32(&mut f, magaddr + 4);
            
            let idaddr = itemaddr as u64 + 28 * index as u64;
            return read_u32(&mut f, idaddr)
        }
        else if itype == 3 {
            let tooladdr = read_u32(&mut f, pmtaddr + PMTTOOLOFFSET);

            let groupaddr = tooladdr + 8 * group as u64;
            let itemaddr = read_u32(&mut f, groupaddr + 4);
            
            let idaddr = itemaddr as u64 + 24 * index as u64;
            return read_u32(&mut f, idaddr)
        }
        else if itype == 5 {
            return index as u64;
        }

        0
    }

    fn read_pmt_id(&self, group: u8, pmtid: u64) -> String {
        let mut f = File::open(format!("/proc/{}/mem", self.pid)).unwrap();
        f.seek(SeekFrom::Start(UNITTXTPOINTER)).unwrap();
        let unittxt = f.read_u32::<LittleEndian>().unwrap() as u64;

        
        f.seek(SeekFrom::Start(unittxt + (group as u64 * 4))).unwrap();
        let itemaddr = f.read_u32::<LittleEndian>().unwrap() as u64;

        f.seek(SeekFrom::Start(itemaddr + 4 * pmtid)).unwrap();
        let straddr = f.read_u32::<LittleEndian>().unwrap() as u64;

        let mut strbuf: [u8; 128] = [0; 128];
        f.seek(SeekFrom::Start(straddr)).unwrap();
        f.read(&mut strbuf).unwrap();

        let mut strbuf16: [u16; 64] = [0; 64];
        LittleEndian::read_u16_into(&strbuf[..], &mut strbuf16[..]);
        
        escape_string(String::from_utf16_lossy(&strbuf16[0..strbuf16.iter().position(|&k| k == 0).unwrap()]))
    }
    
    fn item_name(&self, itype: u8, group: u8, index: u8) -> String {
        let pmtid = self.pmt_item_id(itype, group, index);

        let mut pmtgroup = 1;
        if itype == 5 { // tech
            pmtgroup = 5;
        }

        self.read_pmt_id(pmtgroup, pmtid)
    }

    fn special_name(&self, special: u8) -> String {
        let mut f = File::open(format!("/proc/{}/mem", self.pid)).unwrap();
        let base = read_u32(&mut f, SPECIALTXTPOINTER);

        self.read_pmt_id(1, base + special as u64)
    }
       

    fn parseweapon(&self, item: &[u8; 12]) -> Option<ItemType> {
        let grind = item[3];

        let mut attr = BTreeMap::new();
        for i in 0..3 {
            if item[6+i*2] != 0 {
                attr.insert(item[6 + i*2], item[7 + i*2]);
            }
        }

        let name = self.item_name(item[0], item[1], item[2]);

        let specialtext = if item[4] != 0 {
            self.special_name(item[4] & 0x3F)
        }
        else {
            String::new()
        };
        
        let mut attrnum: Vec<u8> = vec![0; 5]; //= Vec::new(); //Vec<u8>;
        for i in 1..6 {
            match attr.get(&i) {
                Some(v) => {
                    attrnum[(i-1) as usize] = *v;
                },
                None => {
                    attrnum[(i-1) as usize] = 0;
                }
            };
        }

        return Some(ItemType::Weapon(item::Weapon {
            name: name,
            grind: grind,
            special: specialtext,
            native: attrnum[0],
            abeast: attrnum[1],
            machine: attrnum[2],
            dark: attrnum[3],
            hit: attrnum[4]
        }));
    }
    
    fn parsearmor(&self, item: &[u8; 12]) -> Option<ItemType> {
        let name = self.item_name(item[0], item[1], item[2]);
        let slots = item[5];
        let dfp = item[6];
        let evp = item[8];
        
        return Some(ItemType::Armor(item::Armor {
            name: name,
            slots: slots,
            dfp: dfp,
            evp: evp
        }));
    }
    
    fn parseshield(&self, item: &[u8; 12]) -> Option<ItemType> {
        let name = self.item_name(item[0], item[1], item[2]);
        let dfp = item[6];
        let evp = item[8];
        
        return Some(ItemType::Shield(item::Shield {
            name: name,
            dfp: dfp,
            evp: evp
        }));
    }
    
    fn parsemisc(&self, item: &[u8; 12]) -> Option<ItemType> {
        let name = self.item_name(item[0], item[1], item[2]);

        return Some(ItemType::Misc(item::Misc {
            name: name,
            count: item[5]
        }));
    }
    
    fn parsemag(&self, item: &[u8; 12]) -> Option<ItemType> {
        let mut nitem = item.clone();
        nitem[2] = 0; // what exists in item[2]?

        let name = self.item_name(item[0], item[1], item[2]);
        return Some(ItemType::Mag(item::Mag {
            name: name,
        }));
    }
    
    fn parsetech(&self, item: &[u8; 12]) -> Option<ItemType> {
        let level = item[2]+1;
        let name = self.item_name(item[0]+2, item[1], item[4]);

        return Some(ItemType::Tech(item::Tech {
            name: name,
            level: level
        }));
        
    }

    fn parseitem(&self, item: &[u8; 12]) ->Option<ItemType> {
        match item[0] {
            0x00 => self.parseweapon(item),
            0x01 => match item[1] {
                0x01 => self.parsearmor(item),
                0x02 => self.parseshield(item),
                0x03 => self.parsemisc(item),
                _ => None
            },

            0x02 => self.parsemag(item),
            0x03 => match item[1] {
                0x02 => self.parsetech(item),
                _ => self.parsemisc(item),
            },
            _ => None,
        }
    }

    pub fn getchanges(&mut self) -> Vec<DropChange> {
        let mut f = File::open(format!("/proc/{}/mem", self.pid)).unwrap();
        let mut buf: [u8; 4] = [0; 4];
        f.seek(SeekFrom::Start(DROPPOINTER)).unwrap();
        f.read(&mut buf).unwrap();
        let dropoffset = LittleEndian::read_i32(&buf) as u64;
        
        let mut newdrops: BTreeSet<DropBytes> = BTreeSet::new();
        for area in 0..AREACOUNT {
            for item in 0..MAXITEMS {
                let offset = dropoffset + AREASTEP * area + (DROPSTEP as u64) * item;
                f.seek(SeekFrom::Start(offset)).unwrap();
                let mut buf:[u8; DROPSTEP] = [0; DROPSTEP];
                f.read(&mut buf).unwrap();

                if buf[16] == 0 && buf[17] == 0 && buf[18] == 0 {
                    break;
                }
                
                let drop = DropBytes::new(buf);
                newdrops.insert(drop);
            }
        }

        let mut out: Vec<DropChange> = Vec::new();

        for d in newdrops.difference(&self.seen) {
             if let Some(s) = self.parseitem(&d.item) {
                 out.push(DropChange::Add(d.id, s));
             }
        }

        for d in self.seen.difference(&newdrops) {
            out.push(DropChange::Remove(d.id));
        }

        self.seen = newdrops;
        
        return out;
    }
}


#[derive(Debug)]
pub struct DropBytes {
    id: u32, // I think this is a global id, at least
    local_id: u16,
    item: [u8; 12],
}

impl DropBytes {
    fn new(buf: [u8; DROPSTEP]) -> DropBytes {
        let mut item: [u8; 12] = [0; 12];
        item.copy_from_slice(&buf[16..16+12]);
        DropBytes {
            //bytes: [0; DROPSTEP],
            local_id: LittleEndian::read_u16(&buf[14..16]),
            item: item,
            id: LittleEndian::read_u32(&buf[28..32]),
            //item: buf[16..16+12] as [u8; 12],
        }
    }
}

impl cmp::Eq for DropBytes {}

impl cmp::PartialEq for DropBytes {
    fn eq(&self, other: &DropBytes) -> bool {
        self.id == other.id
    }
}

impl cmp::PartialOrd for DropBytes {
    #[inline]
    fn partial_cmp(&self, other: &DropBytes) -> Option<cmp::Ordering> {
        PartialOrd::partial_cmp(&self.id, &other.id)
    }
}


impl cmp::Ord for DropBytes {
    #[inline]
    fn cmp(&self, other: &DropBytes) -> cmp::Ordering {
        Ord::cmp(&self.id, &other.id)
    }
}

