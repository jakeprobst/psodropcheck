extern crate time;
extern crate byteorder;
extern crate gtk;
extern crate gdk;
extern crate glib;
extern crate pango;
mod item;
mod itemdrop;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use gtk::prelude::*;
use gdk::{WindowTypeHint};
use gtk::{Window, WindowType, TextView, ScrolledWindow};
use std::thread;
use std::time::Duration;
use std::thread::sleep;

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
                    data.push_back(item);
                }
            }
            
            sleep(Duration::new(1,0));
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
    timeout_add_seconds(1, move || {
        let mut data = bnewitems.lock().unwrap();
        while let Some(item) = data.pop_front() {
            buffer.insert(&mut buffer.get_end_iter(), item.as_str());
            buffer.insert(&mut buffer.get_end_iter(), "\n");
        }
        textbox.scroll_to_iter(&mut buffer.get_end_iter(), 0.0, false, 0.0, 0.0);
        return Continue(true);
    });

    window.show_all();

    gtk::main();
    
}
