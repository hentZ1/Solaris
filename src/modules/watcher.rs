use notify::{self, EventKind, RecursiveMode, Watcher, recommended_watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;

//define the two type of event that the watcher can return
pub enum FsEvent {
    Created(PathBuf),
    Removed(PathBuf),
}

//this function receives the paths from the main and the sender(tx) from the channel in the main
pub fn watcher(paths: Vec<String>, tx: Sender<FsEvent>) -> anyhow::Result<()> {
    //create the notify watcher and the closure is called everytime that an event occurs in the
    //filesystem
    let mut watcher = recommended_watcher(move |res: notify::Result<notify::Event>| {
        match res {
            Ok(event) => {
                match event.kind {
                    EventKind::Create(_) => {
                        if let Some(path) = event.paths.first()
                            && let Err(e) = tx.send(FsEvent::Created(path.clone()))
                        {
                            eprintln!("solaris: channel send error: {}", e);
                        }
                    }

                    EventKind::Remove(_) => {
                        if let Some(path) = event.paths.first()
                            && let Err(e) = tx.send(FsEvent::Removed(path.clone()))
                        {
                            eprintln!("solaris: channel send error: {}", e);
                        }
                    }

                    _ => {}
                }
            }
            Err(e) => {
                eprintln!("solaris: notify error: {:?}", e);
            }
        }
    })?;
    //watches each directorial that was passed recursively
    for i in &paths {
        watcher.watch(Path::new(i), RecursiveMode::Recursive)?;
    }
    loop {
        //holds the function alive
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
