extern crate glob;
extern crate walkdir;
use glob::glob;
use std::env;
use walkdir::{DirEntry, WalkDir};

fn is_not_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map_or(false, |s| entry.depth() == 0 || !s.starts_with('.'))
}

fn run_walkdir(path: &str) -> usize {
    WalkDir::new(path)
        .into_iter()
        .filter_entry(is_not_hidden)
        .filter_map(Result::ok)
        .count()
}

#[allow(unused)]
fn run_glob(path: &str) {
    for e in glob(path).expect("Failed to find files").flatten() {
        println!("{}", e.display());
        println!("{:?} + {:?}", e.file_stem(), e.extension());
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let target = args.get(1).map_or(".", |a| a.as_str());
    println!("{} files in \"{target}\"", run_walkdir(target));
}
