use notify::{self, EventKind, RecursiveMode, Watcher, recommended_watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;

pub enum FsEvent {
    Created(PathBuf),
    Removed(PathBuf),
}
pub fn watcher(paths: Vec<String>, tx: Sender<FsEvent>) -> anyhow::Result<()> {
    let mut watcher = recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(event) = res {
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

    for i in &paths {
        watcher.watch(Path::new(i), RecursiveMode::Recursive)?;
    }
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
