use clap::Parser;
use solaris::modules::{clap::Args, config::*, watcher::*};
use std::{collections::HashMap, sync::mpsc::channel};

fn main() {
    let args = Args::parse();
    let (tx, rx) = channel();

    //universal path for the config file
    let config_path = dirs::config_dir().unwrap().join("solaris/config.toml");
    create_config(&config_path).expect("It was not possible to create the configuration file.");
    //load the config contents for other modules to use
    let config = load_config(&config_path).expect("Error loading configuration file.");

    //converting the <AuxTargetRules> in a hashmap for the config file
    let mut targets = HashMap::new();
    for rule in &args.targets {
        for ext in &rule.extensions {
            targets.insert(ext.clone(), rule.destination.to_string_lossy().to_string());
        }
    }
    //It checks if all the args are empty so the updater is not called
    let content = TomlContent {
        rules: if args.rules.is_empty() {
            config.rules
        } else {
            args.rules
        },
        watch: if args.watch.is_empty() {
            config.watch.clone()
        } else {
            args.watch
        },
        protected: if args.protected.is_empty() {
            config.protected
        } else {
            args.protected
        },
        targets: if targets.is_empty() {
            config.targets
        } else {
            targets
        },
    };
    //updates the config if something new is added
    update_config(&config_path, content).expect("Failed to update config");

    //start the watcher
    std::thread::spawn(move || {
        watcher(config.watch, tx).expect("Watcher failed");
    });
    //captures the events that the watcher observed and prints
    for event in rx {
        match event {
            FsEvent::Created(path) => println!("Created: {:?}", path),
            FsEvent::Removed(path) => println!("Removed: {:?}", path),
        }
    }
}
