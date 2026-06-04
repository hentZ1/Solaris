use clap::Parser;
use daemonize::Daemonize;
use solaris::modules::{clap::Args, config::*, pid_verifier::*, watcher::*};
use std::{collections::HashMap, fs::File, path::PathBuf, sync::mpsc::channel};
fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let (tx, rx) = channel();

    let config_path = dirs::config_dir().unwrap().join("solaris/config.toml"); //universal path for the config file
    let pid_path = PathBuf::from("/tmp/solaris.pid"); //hardcoded path to the pid
    let stdout = File::create("/tmp/solaris.out")?;
    let stderr = File::create("/tmp/solaris.err")?;
    pid_verifier(&pid_path);

    //verify if config file exists
    if config_path.try_exists()? {
    } else {
        create_config(&config_path)?;
    }

    //load the config contents for other modules to use
    let config = load_config(&config_path)?;

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
    update_config(&config_path, content)?;

    let daemonize = Daemonize::new()
        .pid_file(&pid_path)
        .working_directory("/tmp/")
        .stdout(stdout)
        .stderr(stderr);

    match daemonize.start() {
        Ok(_) => {}
        Err(e) => println!("{} ", e),
    }
    //start the watcher
    std::thread::spawn(move || {
        watcher(config.watch, tx).expect("");
    });

    //captures the events that the watcher observed and prints
    for event in rx {
        match event {
            FsEvent::Created(path) => println!("Created: {:?}", path),
            FsEvent::Removed(path) => println!("Removed: {:?}", path),
        }
    }
    anyhow::Ok(())
}
