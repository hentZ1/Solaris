use std::{collections::HashMap, path::PathBuf};
use crate::modules::watcher::FsEvent::{self};

pub enum Action { Move { destination: PathBuf } }

pub fn apply_rules(event: &FsEvent, targets: &HashMap<String, String>) -> Option<Action> 
{
    let path = match event {
        FsEvent::Created(p) => p
        FsEvent::Removed(_) => return None,
    };

    // get file extension
    let ext = path.extension()?.to_str()?;

    // Query the HashMap using the extension as the key.
    let destination = targets.get(ext)?;

    Some((Action::Move { destination: PathBuf::from(destination) }))
}

