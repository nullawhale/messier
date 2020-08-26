use std::{
    fs::{self, DirEntry},
    path::Path,
};

// File struct
#[derive(Debug)]
struct File {
    name: String,
    size: u64,
}

impl File {
    pub fn new(name: String, size: u64) -> Self {
        File { name, size }
    }
}

// Dir struct
#[derive(Debug)]
struct Dir {
    name: String,
}

impl Dir {
    pub fn new(name: String) -> Self {
        Dir { name }
    }
}

fn main() {
    let (mut files, mut dirs) = get_files_and_dirs(Path::new("."));

    dirs.sort_by(|a, b| a.name.cmp(&b.name));
    for dir in dirs {
        println!("{:?}", dir);
    }

    files.sort_by(|a, b| a.name.cmp(&b.name));
    for file in files {
        println!("{:?}", file);
    }
}

fn get_files_and_dirs(dir: &Path) -> (Vec<File>, Vec<Dir>) {
    let mut files: Vec<File> = Vec::new();
    let mut dirs: Vec<Dir> = Vec::new();

    let paths = fs::read_dir(dir).unwrap();

    for path in paths {
        if let Ok(path) = path {
            if let Ok(metadata) = path.metadata() {
                println!("{:?}", metadata);
                if metadata.is_dir() {
                    dirs.push(Dir::new(path.file_name().into_string().unwrap()));
                } else if metadata.is_file() {
                    files.push(File::new(
                        path.file_name().into_string().unwrap(),
                        metadata.len(),
                    ));
                }
            }
        }
    }

    (files, dirs)
}
