use std::ffi::OsStr;
use std::path::Path;
use std::time::SystemTime;
use std::{env, fs};

fn get_dir_from_env(key: &str) -> String {
    match env::var(key) {
        Ok(val) => val,
        Err(err) => panic!("{}", err),
    }
}

fn get_extension_from_filename(filename: &str) -> String {
    match Path::new(filename).extension().and_then(OsStr::to_str) {
        Some(extension) => format!(".{}", extension),
        _ => String::from("no extension"),
    }
}

fn main() {
    let directory: &str = &get_dir_from_env("DIR");
    const IGNORE_FILES: [&str; 2] = [".DS_Store", ".localized"];

    let paths = fs::read_dir(directory);
    match paths {
        Ok(paths) => {
            paths.for_each(|path| {
                let path = path.unwrap().path();
                let file_name = path.file_name().unwrap().to_str().unwrap();
                let meta_data = fs::metadata(&path).unwrap();
                let file_type = get_extension_from_filename(&file_name);
                let current_time = SystemTime::now();
                let file_time_created = meta_data.created().unwrap();
                let time_diff = current_time
                    .duration_since(file_time_created)
                    .unwrap()
                    .as_secs()
                    / 60
                    / 60
                    / 24;
                if !IGNORE_FILES.contains(&file_name) {
                    println!("Filename: {}", file_name);
                    println!("File type: {:#?}", file_type);
                    println!("Created {:#?} days ago", time_diff);
                }
            });
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
