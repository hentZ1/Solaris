use clap::Parser;
use daemonize::Daemonize;
use solaris::modules::{
    clap::Args,
    config::*,
    pid_verifier::*,
    rules::{Action, apply_rules},
    watcher::*,
};
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
    let mut config = load_config(&config_path)?;

    //converting the <AuxTargetRules> in a hashmap for the config file
    let mut targets = HashMap::new();
    for rule in &args.targets {
        for ext in &rule.extensions {
            targets.insert(ext.clone(), rule.destination.to_string_lossy().to_string());
        }
    }

    let args_content = TomlContent {
        rules: args.rules,
        watch: args.watch,
        protected: args.protected,
        targets: targets,
    };

    config.merge(args_content);

    //updates the config if something new is added
    update_config(&config_path, &config)?;

    let daemonize = Daemonize::new()
        .pid_file(&pid_path)
        .working_directory("/tmp/")
        .stdout(stdout)
        .stderr(stderr);

    match daemonize.start() {
        Ok(_) => {}
        Err(_) => {}
    }
    //start the watcher
    std::thread::spawn(move || {
        watcher(config.watch, tx).expect("");
    });
    //
    for event in rx {
        match &event {
            FsEvent::Created(path) => {
                if let Some(Action::Move { destination }) = apply_rules(&event, &config.targets) {
                    std::fs::rename(path, destination.join(path.file_name().unwrap())).ok();
                }
            }
            FsEvent::Removed(_) => {}
        }
    }
    anyhow::Ok(())
}
