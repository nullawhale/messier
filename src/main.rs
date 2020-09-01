extern crate gio;
extern crate gtk;
extern crate glib;

use std::{
    env,
    fs::{self},
    path::{Path, PathBuf},
    rc::Rc,
    time::SystemTime,
    sync::atomic::{AtomicBool, Ordering}
};

use gio::prelude::*;
use gtk::{Application, ApplicationWindow, Button, Entry, TreeView};
use gtk::prelude::*;
use glib::bitflags::_core::cell::RefCell;

// File struct
#[derive(Debug)]
struct File {
    name: String,
    size: String,
    o_type: String,
    modified: String
}

impl File {
    pub fn new(name: String, size: String, o_type: String, modified: String) -> Self {
        File { name, size, o_type, modified }
    }
}

// Dir struct
/*#[derive(Debug)]
struct Dir {
    name: String,
    size: String,
    o_type: String,
    modified: String
}

impl Dir {
    pub fn new(name: String, modified: String) -> Self {
        Dir { name, size: "0".to_string(), o_type: "File".to_string(), modified}
    }
}*/

#[repr(i32)]
enum Columns {
    Name,
    Size,
    Type,
    Modified,
}

static SHOW_HIDDEN: AtomicBool = AtomicBool::new(false);

fn main() {
    /*let mut files = get_files_and_dirs(path.as_path());

    files.sort_by(|a, b| a.name.cmp(&b.name));
    for file in &files {
        println!("{} ({}) ({}) ({})", file.name, file.size, file.o_type, file.modified);
    }*/

    let abs_pathbuf = fs::canonicalize(env::current_dir().unwrap()).unwrap();

    let application = Application::new(
        Some("com.github.gtk-rs.examples.basic"),
        Default::default(),
    ).expect("failed to initialize GTK application");

    application.connect_activate(move |app| {
        let window = ApplicationWindow::new(app);
        window.set_title("Messier files");
        window.set_position(gtk::WindowPosition::Center);
        window.set_default_size(800, 600);

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 8);
        window.add(&vbox);

        let mut label: Rc<RefCell<gtk::Label>> = Rc::new(RefCell::new(gtk::Label::new(abs_pathbuf.to_str())));
        // let mut rc_label: Rc<RefCell<gtk::Label>> = Rc::new(RefCell::new(label));
        vbox.add(&*label.borrow_mut());

        let back = Button::with_label("<-");
        vbox.add(&back);

        let sw = gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None:: <&gtk::Adjustment>);
        sw.set_shadow_type(gtk::ShadowType::EtchedIn);
        sw.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        vbox.add(&sw);

        // let mut path = env::current_dir().unwrap();
        // let model = Rc::new(create_model(get_files_and_dirs(path.as_path(), SHOW_HIDDEN.load(Ordering::SeqCst)).as_slice()));
        // let mut tree_view = gtk::TreeView::with_model(&*model);
        // tree_view.set_vexpand(true);
        let tree_view = create_and_setup_view();
        let model = create_model();
        tree_view.set_model(Some(&model));

        sw.add(&tree_view);

        let button = Button::with_label("Show hidden");
        vbox.add(&button);

        window.show_all();

        tree_view.connect_row_activated(
            move |tree, path, _col| {
                let model = tree.get_model().unwrap();
                let iter = model.get_iter(path).unwrap();
                let folder = model.get_value(&iter, 0).get::<String>().unwrap();
                let item_type = model.get_value(&iter, 2).get::<String>().unwrap();

                if item_type.unwrap().eq("Folder") {
                    let mut path = env::current_dir().unwrap();
                    path.push(folder.unwrap());
                    env::set_current_dir(path.as_path());
                    println!("{:?}", path.as_path());
                    (*label.borrow_mut()).set_label(path.to_str().unwrap());
                    update_tree_view_with_model(&tree);
                }
            }
        );

        back.connect_clicked(move |_| {
            println!("{:?}", env::current_dir().unwrap());
            let mut path = env::current_dir().unwrap();
            path.pop();
            env::set_current_dir(path.as_path());
            (*label.borrow_mut()).set_label(path.to_str().unwrap());
            update_tree_view_with_model(&tree_view);
        });

        button.connect_clicked(move |b| {
            let show_hidden: bool = SHOW_HIDDEN.load(Ordering::SeqCst);
            SHOW_HIDDEN.fetch_nand(show_hidden, Ordering::SeqCst);

            // update_tree_view_with_model(&tree_view);

            if SHOW_HIDDEN.load(Ordering::SeqCst) {
                b.set_label("Hide hidden");
            } else {
                b.set_label("Show hidden");
            }
        });

        // update_tree_view_with_model(&tree_view);

        let entry = Entry::new();
        entry.connect_key_press_event(|_, _| {
            println!("key pressed");
            Inhibit(false)
        });
    });

    application.run(&[]);
}

fn update_tree_view_with_model(tree: &TreeView) {
    let model = create_model();
    tree.set_model(Some(&model));
}

fn get_files_and_dirs(dir: &Path, show_hidden: bool) -> Vec<File> {
    let mut files: Vec<File> = Vec::new();

    let paths = fs::read_dir(dir).unwrap();

    for path in paths {
        let path = path.unwrap();
        let metadata = path.metadata().unwrap();
        let time: SystemTime = metadata.modified().unwrap();
        let name = path.file_name().into_string().unwrap();

        if metadata.is_dir() {
            files.push(File::new(
                name,
                get_files_count_in_dir(path.path().as_path(), show_hidden),
                "Folder".to_string(),
                format_systime(time)
            ));
        } else if metadata.is_file() {
            files.push(File::new(
                name,
                format_filesize(metadata.len()),
                "File".to_string(),
                format_systime(time)
            ));
        }
    }

    if show_hidden == true {
        files
    } else {
        files.into_iter().filter(|f| !f.name.starts_with('.')).collect()
    }
}

fn get_files_count_in_dir(dir: &Path, show_hidden: bool) -> String {
    let paths = fs::read_dir(dir).unwrap();
    let mut counter = 0;
    for path in paths {
        let path = path.unwrap();
        let metadata = path.metadata().unwrap();

        if metadata.is_dir() || metadata.is_file() {
            let name = path.file_name().into_string().unwrap();
            if !name.starts_with('.') {
                counter += 1;
            } else if show_hidden == true {
                counter += 1;
            }
        }
    }
    format!("{} items", counter)
}

fn format_systime(time: SystemTime) -> String {
    let datetime: chrono::DateTime<chrono::offset::Utc> = time.into();
    datetime.format("%d-%m-%Y %H:%M").to_string()
}

fn format_filesize(bytes: u64) -> String {
    const UNITS: [&str; 8] = ["Bytes", "kB", "MB", "GB", "TB", "PB", "EB", "ZB"];
    const THRESH: f64 = 1024.0;
    let mut bytes: f64 = bytes as f64;

    let mut index = 0;
    while bytes > THRESH {
        bytes /= THRESH;
        index += 1;
    }

    if bytes.fract() == 0.0 {
        format!("{:.0} {}", bytes, UNITS[index])
    } else {
        format!("{:.1} {}", bytes, UNITS[index])
    }
}

fn create_and_setup_view() -> TreeView {
    let tree = TreeView::new();
    tree.set_vexpand(true);
    add_columns(&tree);
    tree
}

fn create_model() -> gtk::ListStore {
    let col_types: [glib::Type; 4] = [
        glib::Type::String,
        glib::Type::String,
        glib::Type::String,
        glib::Type::String,
    ];
    let store = gtk::ListStore::new(&col_types);
    let col_indices: [u32; 4] = [0, 1, 2, 3];

    let mut path = env::current_dir().unwrap();
    let mut files = get_files_and_dirs(path.as_path(), SHOW_HIDDEN.load(Ordering::SeqCst));

    for (_, d) in files.as_slice().iter().enumerate() {
        let values: [&dyn ToValue; 4] = [
            &d.name,
            &d.size,
            &d.o_type,
            &d.modified,
        ];
        store.set(&store.append(), &col_indices, &values);
    }
    store
}

fn add_columns(tree_view: &gtk::TreeView) {
    // Column for Name
    {
        let renderer = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("Name");
        column.add_attribute(&renderer, "text", Columns::Name as i32);
        column.set_sort_column_id(Columns::Name as i32);
        column.set_expand(true);
        tree_view.append_column(&column);
    }

    // Column for Size
    {
        let renderer = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("Size");
        column.add_attribute(&renderer, "text", Columns::Size as i32);
        column.set_sort_column_id(Columns::Size as i32);
        tree_view.append_column(&column);
    }

    // Column for Type
    {
        let renderer = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("Type");
        column.add_attribute(&renderer, "text", Columns::Type as i32);
        column.set_sort_column_id(Columns::Type as i32);
        tree_view.append_column(&column);
    }

    // Column for Modified
    {
        let renderer = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("Modified");
        column.add_attribute(&renderer, "text", Columns::Modified as i32);
        column.set_sort_column_id(Columns::Modified as i32);
        tree_view.append_column(&column);
    }
}
