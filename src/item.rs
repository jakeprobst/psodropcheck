//use std;
//use std::cmp::{Ordering};
//use std::fmt::Display;
//use std::fmt::Formatter;

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Item {
    pub data: [u8; 12],
    //pub timestamp: u64,
}

impl Item {
    pub fn new(data: [u8; 12]) -> Item {
        return Item {data: data};
    }
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
