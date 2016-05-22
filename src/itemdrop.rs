use std::fs::File;
use std::io::{Read, BufReader, BufRead, Seek, SeekFrom};
use std::process::Command;
use std::collections::{BTreeMap};
use memmem::{Searcher, TwoWaySearcher};
use std::vec;
use item;

const MAGICDROPVALUE: [u8; 7] = [0xE6, 0x01, 0x00, 0x55, 0x53, 0x45, 0x00];
const MAGICDROPOFFSET: u64 = 24;
const DROPSTEP: u64 = 0x24;
const AREASTEP: u64 =  0x1B00;
const AREACOUNT: u64 = 18;
//const MINSTARTADDR: u64 = 0x6900000;
const MINSTARTADDR: u64 = 0x0;
const MAXITEMS: u64 = 150;

const WEPFILE: &'static str = "items.txt";
const SPECFILE: &'static str = "specials.txt";
const TECHFILE: &'static str = "techs.txt";

pub enum ItemType {
    Weapon(item::Weapon),
    Armor(item::Armor),
    Shield(item::Shield),
    Misc(item::Misc),
    Mag(item::Mag),
    Tech(item::Tech),
}

fn val2str(data: &[u8]) -> String {
    let mut out = String::new();
    for d in data {
        out.push_str(&format!("{:02X}", d));
    }
    return out;
}

pub struct ItemDrop {
    pid: String,
    weapons: BTreeMap<String, String>,
    specials: BTreeMap<String, String>,
    techs: BTreeMap<String, String>,
    seen: BTreeMap<[u8; 12], u32>,
    pub dropoffset: u64,
}


// TODO: implement difference for BTreeMap?
fn finddiff<T: Ord + Clone>(a: &BTreeMap<T, u32>, b: &BTreeMap<T, u32>) -> Vec<T> {
    let mut out:Vec<T> = Vec::new();

    for (key, value) in a {
        if let Some(count) = b.get(key) {
            if value > count {
                out.push(key.clone());
            }
        }
        else {
            out.push(key.clone());
        }
    }

    return out;
}

impl ItemDrop {
    pub fn new() -> ItemDrop {
        ItemDrop {
            pid: ItemDrop::psopid(),
            weapons: ItemDrop::parsefile(WEPFILE),
            specials: ItemDrop::parsefile(SPECFILE),
            techs: ItemDrop::parsefile(TECHFILE),
            seen: BTreeMap::new(),
            dropoffset: 0,
        }
    }
    
    fn psopid() -> String {
        let cmd = Command::new("pgrep").arg("psobb.exe").output().ok().expect("aaa");
        let s = String::from_utf8_lossy(cmd.stdout.as_slice());
        return String::from(s.trim());
    }

    fn parsefile(path: &str) -> BTreeMap<String, String> {
        let mut out = BTreeMap::new();
        let f = File::open(path).unwrap();
        let br = BufReader::new(f);
        for line in br.lines() {
            match line {
                Ok(l) => {
                    let spl:Vec<_> = l.split(" ").collect();
                    let key = String::from(spl[0]);
                    let val = spl[1..].join(" ");
                    out.insert(key, val);
                }
                Err(_) => {;}
            }
        }
        return out;
    }
    
    fn findmagicinrange(&self, start: u64, end: u64) -> Option<u64> {
        let mut f = File::open(format!("/proc/{}/mem", self.pid)).unwrap();
        f.seek(SeekFrom::Start(start)).unwrap();

        let mut buf = vec::from_elem(0, end as usize - start as usize);
        f.read(&mut buf).unwrap();

        let magicdropvalue = MAGICDROPVALUE; // rust is dumb, consts dont live long enough I guess?
        let search = TwoWaySearcher::new(&magicdropvalue);

        let dropoffset = match search.search_in(buf.as_slice()) {
            Some(off) => off as u64,
            None => 0
        };
        
        if dropoffset != 0 {
            return Some(dropoffset + start - 1);
        }
        else {
            return None;
        }
    }
    
    pub fn findoffsets(&mut self) {
        let f = File::open(format!("/proc/{}/maps", self.pid)).unwrap();
        let br = BufReader::new(f);

        for line in br.lines() {
            match line {
                Ok(l) => {
                    let spl: Vec<&str> = l.split(" ").collect();
                    if !spl[1].contains("r") || spl.last().unwrap().contains("stack") || spl[4] != "0" {
                        continue;
                    }
                    let range: Vec<&str> = spl[0].split("-").collect();
                    let start = u64::from_str_radix(range[0], 16).unwrap();
                    let end = u64::from_str_radix(range[1], 16).unwrap();
                    if start < MINSTARTADDR {
                        continue;
                    }
                    match self.findmagicinrange(start, end) {
                        Some(doff) => {
                            self.dropoffset = doff + MAGICDROPOFFSET;
                            break;
                        }
                        None => {
                        }
                    }
                }
                Err(_) => {;}
            }
        }
    }

    fn parseweapon(&self, item: &[u8; 12]) -> Option<ItemType> {
        let id = val2str(&item[0..3]);
        let grind = item[3];
        let special = val2str(&[item[4] & 0x3F]);

        let mut attr = BTreeMap::new();
        for i in 0..3 {
            if item[6+i*2] != 0 {
                attr.insert(item[6 + i*2], item[7 + i*2]);
            }
        }

        let name = match self.weapons.get(&id) {
            Some(v) => v.clone(),
            None => return None
        };
        
        let specialtext = match self.specials.get(&special) {
            Some(spec) => spec.clone(),
            None => String::new()
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
        let id = val2str(&item[0..3]);

        let name = self.weapons.get(&id).unwrap().clone();
        let slots = item[5];
        let dfp = item[6];
        let evp= item[8];
        
        return Some(ItemType::Armor(item::Armor {
            name: name,
            slots: slots,
            dfp: dfp,
            evp: evp
        }));
    }
    
    fn parseshield(&self, item: &[u8; 12]) -> Option<ItemType> {
        let id = val2str(&item[0..3]);

        let name = self.weapons.get(&id).unwrap().clone();
        let dfp = item[6];
        let evp= item[8];
        
        return Some(ItemType::Shield(item::Shield {
            name: name,
            dfp: dfp,
            evp: evp
        }));
    }
    
    fn parsemisc(&self, item: &[u8; 12]) -> Option<ItemType> {
        let id = val2str(&item[0..3]);

        let name = match self.weapons.get(&id) {
            Some(n) => n.clone(),
            None => format!("[{}]", id)
        };

        return Some(ItemType::Misc(item::Misc {
            name: name,
            count: item[5]
        }));
    }
    
    fn parsemag(&self, item: &[u8; 12]) -> Option<ItemType> {
        let id = val2str(&item[0..3]);

        let name = self.weapons.get(&id).unwrap().clone();
        return Some(ItemType::Mag(item::Mag {
            name: name,
        }));
    }
    
    fn parsetech(&self, item: &[u8; 12]) -> Option<ItemType> {
        let level = item[2]+1;
        let id = val2str(&[item[4]]);

        let name = self.techs.get(&id).unwrap().clone();

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

    pub fn getchanges(&mut self) -> Vec<ItemType> {
        let mut f = File::open(format!("/proc/{}/mem", self.pid)).unwrap();
        let mut newdrops:BTreeMap<[u8; 12], u32> = BTreeMap::new();
        //let mut newdrops = Vec::new();
        for area in 0..AREACOUNT {
            for item in 0..MAXITEMS {
                let offset = self.dropoffset + AREASTEP * area + DROPSTEP * item;
                f.seek(SeekFrom::Start(offset)).unwrap();
                let mut buf:[u8; 12] = [0; 12];
                f.read(&mut buf).unwrap();

                //newdrops.insert(buf);

                if !newdrops.contains_key(&buf) {
                    newdrops.insert(buf, 0);
                }

                match newdrops.get_mut(&buf) {
                    Some(a) => {
                        *a = *a + 1;
                    },
                    None => {}
                }
            }
        }

        let mut out: Vec<ItemType> = Vec::new();
        for i in finddiff(&newdrops, &self.seen) {
            //match self.item2string(&i) {
            match self.parseitem(&i) {
                Some(s) => {
                    out.push(s);
                },
                None => {}
            }
        }

        self.seen = newdrops;
        
        return out;
    }
}
