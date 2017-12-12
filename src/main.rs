extern crate time;
extern crate byteorder;
extern crate gtk;
extern crate gdk;
extern crate glib;
extern crate pango;
extern crate memmem;

mod itemdrop;
mod item;
use std::fmt::Write;
use std::collections::{VecDeque, HashMap};
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
    let s = match item {
        itemdrop::ItemType::Weapon(item) => weaponstring(item),
        itemdrop::ItemType::Armor(item) => armorstring(item),
        itemdrop::ItemType::Shield(item) => shieldstring(item),
        itemdrop::ItemType::Misc(item) => miscstring(item),
        itemdrop::ItemType::Mag(item) => magstring(item),
        itemdrop::ItemType::Tech(item) => techstring(item)
    };

    format!("<span color=\"black\">{}</span>", s)
}

fn main() {
    let newitems = Arc::new(Mutex::new(VecDeque::new()));

    let mut itemdrop = itemdrop::ItemDrop::new();
    let anewitems = newitems.clone();
    thread::spawn(move || {
        {
            let mut data = anewitems.lock().unwrap();
            //data.push_back(String::from("finding offsets..."));
        }
        itemdrop.findoffsets();
        {
            let mut data = anewitems.lock().unwrap();
            let s = format!("found: 0x{:X}\n", itemdrop.dropoffset);
            //data.push_back(s);
        }
        loop {
            {
                let mut data = anewitems.lock().unwrap();
                let items = itemdrop.getchanges();
                for item in items {
                    //data.push_back(item2string(item));
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

    //let textbox = TextView::new();
    //gtk::WidgetExt::override_font(&textbox, &pango::FontDescription::from_string("Deja Vu Sans Mono 12"));


    /*let scrollb = ScrolledWindow::new(None, None);
    scrollb.add(&textbox);
    
    window.add(&scrollb);*/

    //let buffer = textbox.get_buffer().unwrap();
    //buffer.insert(&mut buffer.get_end_iter(), "finding memory offset...");

    let tree = gtk::TreeView::new();
    tree.set_headers_visible(false);

    let selection = tree.get_selection();
    selection.set_mode(gtk::SelectionMode::None);

    /*for &(i, vis) in &[(0, false), (1, true)] {
        let col = gtk::TreeViewColumn::new();
        let cell = gtk::CellRendererText::new();
        col.pack_start(&cell, true);
        col.set_spacing(0);

        col.add_attribute(&cell, "markup", i);
        cell.set_visible(vis);
        //col.add_attribute(&cell, "ypad", 0);
        //col.set_padding(&cell, 0, 0);
        cell.set_padding(0, 0);
        tree.append_column(&col);
}*/
    let col = gtk::TreeViewColumn::new();
    let cell = gtk::CellRendererText::new();
    col.pack_start(&cell, true);
    col.set_spacing(0);

    col.add_attribute(&cell, "markup", 0);
    //cell.set_visible(vis);
    //col.add_attribute(&cell, "ypad", 0);
    //col.set_padding(&cell, 0, 0);
    cell.set_padding(0, 0);
    tree.append_column(&col);
        
    gtk::WidgetExt::override_font(&tree, &pango::FontDescription::from_string("Deja Vu Sans Mono 12"));

    //let model = gtk::ListStore::new(&[u32::static_type(), String::static_type(), String::static_type()]);
    let mut rowlookup = HashMap::new();
    
    //let model = gtk::ListStore::new(&[u32::static_type(), String::static_type()]);
    let model = gtk::ListStore::new(&[String::static_type()]);

    tree.set_model(Some(&model));

    let scrollb = ScrolledWindow::new(None, None);
    scrollb.add(&tree);
    
    window.add(&scrollb);

    
    //window.add(&tree);

    let bnewitems = newitems.clone();
    timeout_add_seconds(1, move || {
        let mut data = bnewitems.lock().unwrap();
        let mut last_iter = None;
        while let Some(item) = data.pop_front() {

            match item {
                itemdrop::DropChange::Add(id, drop) => {
                    //let iter = model.insert_with_values(None, &[0, 1], &[&id, &item2string(drop)]);
                    let iter = model.insert_with_values(None, &[0], &[&item2string(drop)]);
                    last_iter = Some(iter.clone());
                    rowlookup.insert(id, iter);
                }
                itemdrop::DropChange::Remove(id) => {
                    if let Some(iter) = rowlookup.get(&id) {
                        let txt = model.get_value(iter, 0).downcast::<String>().unwrap().get().unwrap();
                        model.set_value(iter, 0, &format!("<span fgalpha=\"50%\" style=\"italic\">{}</span>", txt).to_value());
                    }
                }
            }
            //buffer.insert_markup(&mut buffer.get_end_iter(), item.as_str());
            //buffer.insert(&mut buffer.get_end_iter(), "\n");
            //println!("item: {:#?}", item);
        }
        //textbox.scroll_to_iter(&mut buffer.get_end_iter(), 0.0, false, 0.0, 0.0);
        //let path = gtk::TreePath::new_from_string(&format!("{}", model.iter_n_children(None)-1));
        //let path
        if let Some(iter) = last_iter {
            if let Some(path) = model.get_path(&iter) { // Some(path) -> Some(&path)
                tree.scroll_to_cell(Some(&path), None, false, 0.0, 0.0); 
            }
        }
        return Continue(true);
    });

    window.show_all();

    gtk::main();
    
}
