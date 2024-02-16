use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use notify_rust::Notification;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;

const IGNORED_FILES: [&str; 2] = [".DS_Store", ".gitignore"];
const WHITELISTED_FILES: [&str; 1] = ["png"];
const FILE_EXPIRATION_TIME_IN_SECONDS: u64 = 10;

fn get_args() -> Vec<String> {
    std::env::args().collect()
}

fn main() {
    // print!("{}[2J", 27 as char);
    let cleaner_thread = thread::spawn(|| {
        let args = get_args();
        if args.len() < 2 {
            println!("Error: No path provided");
            std::process::exit(1);
        }
        let path = args[1].clone();
        cleaner(&path);
    });

    let watcher_thread = thread::spawn(|| {
        let args = get_args();
        if args.len() < 2 {
            println!("Error: No path provided");
            std::process::exit(1);
        }
        let path = args[1].clone();
        if let Err(e) = watch(&path) {
            println!("error: {:?}", e)
        }
    });

    cleaner_thread.join().unwrap();
    watcher_thread.join().unwrap();
}

#[derive(Debug)]
struct File {
    name: String,
    path: String,
    created: SystemTime,
    modified: SystemTime,
    size: u64,
    file_type: String,
}

fn file_type_is_whitelisted(file: &File) -> bool {
    if WHITELISTED_FILES.contains(&file.file_type.as_str()) {
        return true;
    } else {
        return false;
    }
}

// fn file_is_whitelisted(file: &File) -> bool {
//     if WHITELISTED_FILES.contains(&file.name.as_str()) {
//         return true;
//     } else {
//         return false;
//     }
// }

fn delete_file(file: &File) {
    fs::remove_file(&file.path).unwrap()
}

fn cleaner(path: &str) {
    println!("Cleaning: {}", &path);
    loop {
        let files = get_files_in_dir(&path);
        match files {
            Ok(files) => {
                files.iter().for_each(|file| {
                    match file.modified.elapsed() {
                        Ok(elapsed) => {
                            if elapsed.as_secs() > FILE_EXPIRATION_TIME_IN_SECONDS && file_type_is_whitelisted(&file) {
                                delete_file(&file);
                            }
                        }
                        Err(e) => {
                            println!("Error: {}", e);
                        }
                    }
                });
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
        thread::sleep(Duration::from_secs(1));
    }
}

fn get_files_in_dir(path: &str) -> Result<Vec<File>, std::io::Error> {
    let dir_files = fs::read_dir(&path);
    match dir_files {
        Ok(files) => {
            let mut file_vec: Vec<File> = Vec::new();
            files.for_each(|file| match file {
                Ok(file) => {
                    if file.path().is_dir() {
                        return;
                    }
                    if IGNORED_FILES.contains(&file.path().file_name().unwrap().to_str().unwrap()) {
                        return;
                    }
                    let file = get_file_info(&file.path().to_str().unwrap());
                    file_vec.push(file);
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            });
            return Ok(file_vec);
        }
        Err(e) => {
            println!("Error: {}", e);
            return Err(e);
        }
    }
}

fn get_file_info(path: &str) -> File {
    let meta_data = fs::metadata(path).unwrap();
    let file_time_created = meta_data.created().unwrap();
    let file_time_modified = meta_data.modified().unwrap();
    let file_size = meta_data.len();
    let file_name = Path::new(path).file_name().unwrap().to_str().unwrap();
    File {
        name: file_name.to_string(),
        path: path.to_string(),
        created: file_time_created,
        modified: file_time_modified,
        size: file_size,
        file_type: file_name
            .split('.')
            .collect::<Vec<&str>>()
            .pop()
            .unwrap()
            .to_string(),
    }
}

fn handle_create_event(event: &notify::Event, path: &str) {
    let event_file_path = Path::new(&event.paths[0]);
    if event_file_path.is_dir() {
        return;
    }
    let file = get_mofified_file(&event, &path);
    let notification_string = format!("{} was created", file.name);
    
    println!("{}", notification_string);
}

fn get_mofified_file(event: &notify::Event, path: &str) -> File {
    let dir_files = fs::read_dir(&path);
    let mut modified_file: File = File {
        name: "".to_string(),
        path: "".to_string(),
        created: SystemTime::now(),
        modified: SystemTime::now(),
        size: 0,
        file_type: "".to_string(),
    };
    match dir_files {
        Ok(files) => {
            files.for_each(|file| match file {
                Ok(file) => {
                    let event_path_os_file_name: &OsStr = &event.paths[0].file_name().unwrap();
                    if let Some(event_path) = event_path_os_file_name.to_str() {
                        let pth = file.path();
                        let file_path = Path::new(pth.to_str().unwrap())
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap();

                        if &event_path.get(1..).unwrap() == &file_path {
                            modified_file = get_file_info(&file.path().to_str().unwrap());
                        };
                    }
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            });
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
    return modified_file;
}


fn handle_remove_event(path: &str) {
    let notification_string = format!("A file was removed from {}", &path);
    Notification::new()
        .summary("File Removed")
        .body(&notification_string)
        .show()
        .unwrap();
}

fn watch(path: &str) -> notify::Result<()> {
    println!("Watching: {}", &path);
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            Ok(event) => {
                match event.kind {
                    notify::EventKind::Access(_) => {}
                    notify::EventKind::Create(_) => {
                        handle_create_event(&event, &path);
                    }
                    notify::EventKind::Modify(_) => {
                        // handle_modify_event(&event, &path);
                    }
                    notify::EventKind::Remove(_) => {
                        println!("File Removed: {:?}", &path);
                        handle_remove_event(&path);
                    }
                    notify::EventKind::Other => {}
                    notify::EventKind::Any => {}
                };
                print!("")
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}
