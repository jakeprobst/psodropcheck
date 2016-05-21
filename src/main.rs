extern crate time;
extern crate byteorder;
extern crate gtk;
extern crate gdk;
extern crate glib;
extern crate pango;
mod itemdrop;
mod item;
use std::fmt::Write;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use gtk::prelude::*;
use gdk::{WindowTypeHint};
use gtk::{Window, WindowType, TextView, ScrolledWindow};
use std::thread;
use std::time::Duration;
use std::thread::sleep;


fn weaponstring(item: item::Weapon) -> String {
    let mut output = String::new();


    if item.special != "" {
        write!(output, "{} ", item.special).unwrap();
    }
    write!(output, "{}", item.name).unwrap();
    if item.grind > 0 {
        write!(output, " +{}", item.grind).unwrap();
        
    }

    write!(output, " {}/{}/{}/{}", item.native, item.abeast, item.machine, item.dark).unwrap();

    if item.hit > 0 {
        write!(output, "|<span foreground=\"red\">{}</span>", item.hit).unwrap();
        output = format!("<span weight=\"bold\">{}</span>", output);
    }
    else {
        write!(output, "|0").unwrap();
    }

    return output;
}

fn armorstring(item: item::Armor) -> String {
    let mut output = String::new();
    write!(output, "{} [{}s +{}d +{}e]", item.name, item.slots, item.dfp, item.evp).unwrap();
    return output;
}

fn shieldstring(item: item::Shield) -> String {
    let mut output = String::new();
    write!(output, "{} [+{}d +{}e]", item.name, item.dfp, item.evp).unwrap();
    return output;
}

fn miscstring(item: item::Misc) -> String {
    let mut output = String::new();
    write!(output, "{}", item.name).unwrap();
    if item.count > 0 {
        write!(output, " x{}", item.count).unwrap();
    }
    return output;
}

fn magstring(item: item::Mag) -> String {
    let mut output = String::new();
    write!(output, "{}", item.name).unwrap();
    return output;
}

fn techstring(item: item::Tech) -> String {
    let mut output = String::new();
    write!(output, "{} {}", item.name, item.level).unwrap();
    return output;
}

fn item2string(item: itemdrop::ItemType) -> String {
    match item {
        itemdrop::ItemType::Weapon(item) => weaponstring(item),
        itemdrop::ItemType::Armor(item) => armorstring(item),
        itemdrop::ItemType::Shield(item) => shieldstring(item),
        itemdrop::ItemType::Misc(item) => miscstring(item),
        itemdrop::ItemType::Mag(item) => magstring(item),
        itemdrop::ItemType::Tech(item) => techstring(item)
    }
}

fn main() {
    let newitems = Arc::new(Mutex::new(VecDeque::new()));

    let mut itemdrop = itemdrop::ItemDrop::new();
    let anewitems = newitems.clone();
    thread::spawn(move || {
        {
            let mut data = anewitems.lock().unwrap();
            data.push_back(String::from("finding offsets..."));
        }
        itemdrop.findoffsets();
        println!("found 0x{:X}", itemdrop.dropoffset);
        {
            let mut data = anewitems.lock().unwrap();
            let s = format!("found: 0x{:X}\n", itemdrop.dropoffset);
            data.push_back(s);
        }
        loop {
            {
                let mut data = anewitems.lock().unwrap();
                let items = itemdrop.getchanges();
                for item in items {
                    data.push_back(item2string(item));
                }
            }
            
            sleep(Duration::new(0,25000000));
        }
    });
    
    gtk::init().unwrap();
    
    let window = Window::new(WindowType::Toplevel);
    window.set_type_hint(WindowTypeHint::Dock);
    window.set_size_request(500,230);

    let textbox = TextView::new();
    textbox.override_font(&pango::FontDescription::from_string("Deja Vu Sans Mono 12"));


    let scrollb = ScrolledWindow::new(None, None);
    scrollb.add(&textbox);
    
    window.add(&scrollb);

    let buffer = textbox.get_buffer().unwrap();
    //buffer.insert(&mut buffer.get_end_iter(), "finding memory offset...");

    let bnewitems = newitems.clone();
    timeout_add(250, move || {
        let mut data = bnewitems.lock().unwrap();
        while let Some(item) = data.pop_front() {
            buffer.insert_markup(&mut buffer.get_end_iter(), item.as_str());
            buffer.insert(&mut buffer.get_end_iter(), "\n");
        }
        textbox.scroll_to_iter(&mut buffer.get_end_iter(), 0.0, false, 0.0, 0.0);
        return Continue(true);
    });

    window.show_all();

    gtk::main();
    
}
