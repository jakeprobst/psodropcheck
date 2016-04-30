use std::fs::File;
use std::io::{Read, BufReader, BufRead, Seek, SeekFrom};
use std::process::Command;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;
//use byteorder::{ByteOrder, BigEndian};

const MAGICDROPVALUE: [u8; 7] = [0xE6, 0x01, 0x00, 0x55, 0x53, 0x45, 0x00];
const MAGICDROPOFFSET: u64 = 24;
const DROPSTEP: u64 = 0x24;
const AREASTEP: u64 =  0x1B00;
const AREACOUNT: u64 = 18;
const MINSTARTADDR: u64 = 0x6900000;
const MAXITEMS: u64 = 64;

const WEPFILE: &'static str = "items.txt";
const SPECFILE: &'static str = "specials.txt";
const TECHFILE: &'static str = "techs.txt";


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
    seen: BTreeSet<[u8; 12]>,
    //seen: Vec<[u8; 12]>,
    pub dropoffset: u64,
}

impl ItemDrop {
    pub fn new() -> ItemDrop {
        ItemDrop {
            pid: ItemDrop::psopid(),
            weapons: ItemDrop::parsefile(WEPFILE),
            specials: ItemDrop::parsefile(SPECFILE),
            techs: ItemDrop::parsefile(TECHFILE),
            seen: BTreeSet::new(),
            //seen: Vec::new(),
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
    
    // slow as fuckkkkk
    fn findmagicinrange(&self, start: u64, end: u64) -> Option<u64> {
        let mut dropoffset: u64 = 0;
        let mut dvindex = 0;

        let mut f = File::open(format!("/proc/{}/mem", self.pid)).unwrap();
        f.seek(SeekFrom::Start(start)).unwrap();
        for i in start..end+1 {
            let mut buf: [u8; 1] = [0];
            match f.read(&mut buf) {
                Ok(_) => {
                }
                Err(_) => {
                    break;
                }
            }
            let val = buf[0];

            if val == MAGICDROPVALUE[dvindex] {
                dvindex += 1;
            }
            else {
                dvindex = 0;
            }

            if dvindex == MAGICDROPVALUE.len() {
                dropoffset = i - MAGICDROPVALUE.len() as u64;
                dvindex = 0
            }
        }

        if dropoffset != 0 {
            return Some(dropoffset);
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

    fn printweapon(&self, item: &[u8; 12]) -> Option<String> {
        let id = val2str(&item[0..3]);
        let grind = &item[3];
        let special = val2str(&[item[4] & 0x3F]);

        let mut attr = BTreeMap::new();
        for i in 0..3 {
            if item[6+i*2] != 0 {
                attr.insert(item[6 + i*2], item[7 + i*2]);
            }
        }

        let name = match self.weapons.get(&id) {
            Some(v) => v.as_str(),
            None => return None
        };
        
        let mut output = String::new();
        if special != "00" {
            match self.specials.get(&special) {
                Some(spec) => {
                    write!(output, "{} ", spec).unwrap();
                },
                None => {}
            }
        }
        write!(output, "{}", name).unwrap();
        if *grind != 0 {
            write!(output, " +{}", *grind).unwrap();
        }

        let mut attrnum: Vec<String> = vec![String::new(); 5]; //= Vec::new(); //Vec<u8>;
        for i in 1..6 {
            match attr.get(&i) {
                Some(v) => {
                    attrnum[(i-1) as usize] = v.to_string();
                },
                None => {
                    attrnum[(i-1) as usize] = String::from("0");
                }
            };
        }
        write!(output, " {}", attrnum.join("/")).unwrap();
        return Some(output);
    }
    
    fn printarmor(&self, item: &[u8; 12]) ->String {
        let id = val2str(&item[0..3]);

        let name = self.weapons.get(&id).unwrap();
        //let slots = BigEndian::read_i16(&item[4..6]);
        let slots = item[5];
        let dfp = item[7];
        let evp= item[10];
        //let dfp = BigEndian::read_i16(&item[6..8]);
        //let evp = BigEndian::read_i16(&item[9..11]);

        let mut output = String::new();
        write!(output, "{} [{}s +{}d +{}e]", name ,slots, dfp, evp).unwrap();
        return output;
    }
    
    fn printshield(&self, item: &[u8; 12]) ->String {
        let id = val2str(&item[0..3]);

        let name = self.weapons.get(&id).unwrap();
        //let dfp = BigEndian::read_i16(&item[6..8]);
        //let evp = BigEndian::read_i16(&item[9..11]);
        let dfp = item[7];
        let evp= item[10];
        
        let mut output = String::new();
        write!(output, "{} [+{}d +{}e]", name , dfp, evp).unwrap();
        return output;
    }
    
    fn printmisc(&self, item: &[u8; 12]) -> String {
        let id = val2str(&item[0..3]);

        //let name = self.weapons.get(&id).unwrap();
        let name = match self.weapons.get(&id) {
            Some(n) => n.as_str(),
            None => "Unknown"
        };
        let count = item[5];
        
        let mut output = String::new();
        if count != 0 {
            write!(output, "{} x{}", name , count).unwrap();
        }
        else {
            write!(output, "{}", name).unwrap();
        }
        return output;
    }
    
    fn printmag(&self, item: &[u8; 12]) -> String {
        let id = val2str(&item[0..3]);

        let name = self.weapons.get(&id).unwrap();
        let mut output = String::new();
        write!(output, "{}", name).unwrap();
        return output;
    }
    
    fn printtech(&self, item: &[u8; 12]) -> String {
        let level = item[2]+1;
        let id = val2str(&[item[4]]);

        let name = self.techs.get(&id).unwrap();
        let mut output = String::new();
        write!(output, "{} {}", name, level).unwrap();
        return output;
    }

    fn item2string(&self, item: &[u8; 12]) -> Option<String> {
        match item[0] {
            0x00 => self.printweapon(item),
            0x01 => match item[1] {
                0x01 => Some(self.printarmor(item)),
                0x02 => Some(self.printshield(item)),
                0x03 =>Some(self.printmisc(item)),
                _ => None
            },

            0x02 => Some(self.printmag(item)),
            0x03 => match item[1] {
                0x02 => Some(self.printtech(item)),
                _ => Some(self.printmisc(item)),
            },
            _ => None,
        }
    }

    pub fn getchanges(&mut self) -> Vec<String> {
        let mut f = File::open(format!("/proc/{}/mem", self.pid)).unwrap();
        let mut newdrops = BTreeSet::new();
        //let mut newdrops = Vec::new();
        for area in 0..AREACOUNT {
            for item in 0..MAXITEMS {
                let offset = self.dropoffset + AREASTEP * area + DROPSTEP * item;
                f.seek(SeekFrom::Start(offset)).unwrap();
                let mut buf:[u8; 12] = [0; 12];
                f.read(&mut buf).unwrap();

                newdrops.insert(buf);
            }
        }

        let mut out: Vec<String> = Vec::new();
        for &i in newdrops.difference(&self.seen) {
            match self.item2string(&i) {
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
