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
use gtk::{Application, ApplicationWindow, Button, Entry};
use gtk::prelude::*;

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

        let label = gtk::Label::new(abs_pathbuf.to_str());
        vbox.add(&label);

        let sw = gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None:: <&gtk::Adjustment>);
        sw.set_shadow_type(gtk::ShadowType::EtchedIn);
        sw.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        vbox.add(&sw);

        let path = env::current_dir().unwrap();
        let model = Rc::new(create_model(get_files_and_dirs(path.as_path(), SHOW_HIDDEN.load(Ordering::SeqCst)).as_slice()));
        let tree_view = gtk::TreeView::with_model(&*model);
        tree_view.set_vexpand(true);

        sw.add(&tree_view);
        add_columns(&tree_view);

        let button = Button::with_label("Show hidden");
        button.connect_clicked(move |b| {
            let show_hidden: bool = SHOW_HIDDEN.load(Ordering::SeqCst);
            SHOW_HIDDEN.fetch_nand(show_hidden, Ordering::SeqCst);

            let path = env::current_dir().unwrap();
            let model = Rc::new(create_model(get_files_and_dirs(path.as_path(), SHOW_HIDDEN.load(Ordering::SeqCst)).as_slice()));
            tree_view.set_model(Some(&*model));

            if SHOW_HIDDEN.load(Ordering::SeqCst) {
                b.set_label("Hide hidden");
            } else {
                b.set_label("Show hidden");
            }

            println!("{:?}", SHOW_HIDDEN.load(Ordering::SeqCst));

        });
        vbox.add(&button);

        window.show_all();

        let entry = Entry::new();
        entry.connect_key_press_event(|_, _| {
            println!("key pressed");
            Inhibit(false)
        });
    });

    application.run(&[]);
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
                "0".to_string(),
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

fn create_model(files: &[File]) -> gtk::ListStore {

    let col_types: [glib::Type; 4] = [
        glib::Type::String,
        glib::Type::String,
        glib::Type::String,
        glib::Type::String,
    ];

    let store = gtk::ListStore::new(&col_types);

    let col_indices: [u32; 4] = [0, 1, 2, 3];

    for (_, d) in files.iter().enumerate() {
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
