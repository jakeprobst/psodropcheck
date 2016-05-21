//use std;
//use std::cmp::{Ordering};
//use std::fmt::Display;
//use std::fmt::Formatter;

//#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]

pub struct Weapon {
    pub name: String,
    pub grind: u8,
    pub special: String,
    pub native: u8,
    pub abeast: u8,
    pub machine: u8,
    pub dark: u8,
    pub hit: u8,
}

pub struct Armor {
    pub name: String,
    pub slots: u8,
    pub dfp: u8,
    pub evp: u8,
}

pub struct Shield {
    pub name: String,
    pub dfp: u8,
    pub evp: u8,
}

pub struct Misc {
    pub name: String,
    pub count: u8,
}

pub struct Mag {
    pub name: String,
}

pub struct Tech {
    pub name: String,
    pub level: u8,
}

/*impl std::fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        //let x = self.data[2];
        return write!(f, "itemz");
    }
}*/

/*impl std::cmp::Ord for Item {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.timestamp > other.timestamp {
            return Ordering::Greater;
        }
        else if self.timestamp < other.timestamp {
            return Ordering::Less;
        }
        else {
            return Ordering::Equal;
        }
    }
}

impl std::cmp::PartialOrd for Item {
    fn partial_cmp(&self, other: &Item) -> Option<Ordering> {
        if self.timestamp > other.timestamp {
            return Some(Ordering::Greater);
        }
        else if self.timestamp < other.timestamp {
            return Some(Ordering::Less);
        }
        return Some(Ordering::Equal);
    }
}

impl std::cmp::Eq for Item {
}

impl std::cmp::PartialEq for Item {
    fn eq(&self, other: &Item) -> bool {
        return self.data == other.data;
    }
}*/
