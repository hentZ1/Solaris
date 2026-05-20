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
        //discart events that are errors
        if let Ok(event) = res {
            //this match verifies the event type, sends them to the channel and discart everyother
            //event that the watcher is not interested
            match event.kind {
                EventKind::Create(_) => {
                    tx.send(FsEvent::Created(event.paths[0].clone())).ok();
                }

                EventKind::Remove(_) => {
                    tx.send(FsEvent::Removed(event.paths[0].clone())).ok();
                }

                _ => {}
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
