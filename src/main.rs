use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("Argument 1 needs to be a path");
    println!("watching {}", path);
    if let Err(e) = watch(path) {
        println!("error: {:?}", e)
    }
}

fn watch(path: String) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            Ok(event) => {
                match event.kind {
                    notify::EventKind::Access(_) => {}
                    notify::EventKind::Create(_) => {
                        println!("File Created");
                        println!("===============================");

                        let dir_files = fs::read_dir(&path);
                        match dir_files {
                            Ok(files) => {
                                files.for_each(|file| match file {
                                    Ok(file) => {
                                        let event_path_os_file_name: &OsStr =
                                            &event.paths[0].file_name().unwrap();
                                        if let Some(event_path) = event_path_os_file_name.to_str() {
                                            let pth = file.path();
                                            let file_path = Path::new(pth.to_str().unwrap())
                                                .file_name()
                                                .unwrap()
                                                .to_str()
                                                .unwrap();

                                            if &event_path.get(1..).unwrap() == &file_path {
                                                let meta_data = fs::metadata(&file.path()).unwrap();
                                                let current_time = SystemTime::now();
                                                let file_time_created =
                                                    meta_data.created().unwrap();
                                                let time_diff = current_time
                                                    .duration_since(file_time_created)
                                                    .unwrap()
                                                    .as_secs()
                                                    / 60
                                                    / 60
                                                    / 24;
                                                println!("File: {:?}", file.file_name());
                                                println!("Path: {:?}", file.path());
                                                println!("Created {:#?} days ago", time_diff);
                                                println!(
                                                    "Metadata: {:?}",
                                                    fs::metadata(&file.path()).unwrap()
                                                );
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
                    }
                    notify::EventKind::Modify(_) => {}
                    notify::EventKind::Remove(_) => {}
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
