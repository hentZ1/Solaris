use clap::Parser;
use solaris::modules::{clap::Args, config::*, watcher::*};
use std::sync::mpsc::channel;

fn main() {
    let args = Args::parse();
    let (tx, rx) = channel();
    let config_path = dirs::config_dir().unwrap().join("solaris/config.toml");
    create_config(&config_path).expect("It was not possible to create the configuration file.");
    let config = load_config(&config_path).expect("Error loading configuration file.");
    if args.watch.is_empty() {
    } else {
        let content = TomlContent {
            rules: args.rules,
            watch: args.watch,
            protected: args.protected,
        };

        update_config(&config_path, content).expect("Failed to update config");
    }
    std::thread::spawn(move || {
        watcher(config.watch, tx).expect("Watcher failed");
    });
    for event in rx {
        match event {
            FsEvent::Created(path) => println!("Created: {:?}", path),
            FsEvent::Removed(path) => println!("Removed: {:?}", path),
        }
    }
}
