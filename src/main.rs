use clap::Parser;
use daemonize::Daemonize;
use solaris::modules::{
    clap::{Args, Commands, ConfigField},
    config::*,
    pid_verifier::*,
    rules::{Action, apply_rules},
    watcher::*,
};
use std::{
    collections::HashMap,
    fs::File,
    path::PathBuf,
    sync::mpsc::{Receiver, channel},
};

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config_path = dirs::config_dir().unwrap().join("solaris/config.toml");

    //verify if the config file exists
    if !config_path.try_exists()? {
        create_config(&config_path)?;
    }

    if let Some(result) = subcommand_handler(&args.command, &config_path) {
        return result;
    }
    //channel
    let (tx, rx) = channel();
    //paths
    let pid_path = PathBuf::from("/tmp/solaris.pid");
    let stdout = File::create("/tmp/solaris.out")?;
    let stderr = File::create("/tmp/solaris.err")?;

    pid_verifier(&pid_path);

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
        targets,
    };

    config.merge(args_content);

    //updates the config if something new is added
    update_config(&config_path, &config)?;

    //runs the daemon
    daemon_runner(&pid_path, stdout, stderr)?;

    std::thread::spawn(move || {
        watcher(config.watch, tx).ok();
    });

    //apply the rules
    applyrules_runner(rx, &config.targets);

    anyhow::Ok(())
}

//function to run the daemon
fn daemon_runner(pid_path: &PathBuf, stdout: File, stderr: File) -> anyhow::Result<()> {
    let daemonize = Daemonize::new()
        .pid_file(&pid_path)
        .working_directory("/tmp/")
        .stdout(stdout)
        .stderr(stderr);

    daemonize.start().map_err(|e| {
        eprintln!("{}", e);
        anyhow::anyhow!(e)
    })?;

    anyhow::Ok(())
}
//function to run the apply_rules
fn applyrules_runner(rx: Receiver<FsEvent>, targets: &HashMap<String, String>) {
    for event in rx {
        match &event {
            FsEvent::Created(path) => {
                if let Some(Action::Move { destination }) = apply_rules(&event, targets) {
                    std::fs::rename(path, destination.join(path.file_name().unwrap())).ok();
                }
            }
            FsEvent::Removed(_) => {}
        }
    }
}

fn subcommand_handler(
    command: &Option<Commands>,
    config_path: &PathBuf,
) -> Option<anyhow::Result<()>> {
    match command {
        //LIST
        Some(Commands::List { field }) => Some((|| -> anyhow::Result<()> {
            let config = load_config(config_path)?;
            match field {
                Some(ConfigField::Watch) => {
                    for item in &config.watch {
                        println!("{}", item);
                    }
                }
                Some(ConfigField::Rules) => {
                    for item in &config.rules {
                        println!("{}", item);
                    }
                }
                Some(ConfigField::Protected) => {
                    for item in &config.protected {
                        println!("{}", item);
                    }
                }
                Some(ConfigField::Targets) => {
                    for (ext, dest) in &config.targets {
                        println!("{ext} -> {dest}");
                    }
                }
                None => {
                    println!("Rules:");
                    for item in &config.rules {
                        println!("  {}", item);
                    }
                    println!("Watch:");
                    for item in &config.watch {
                        println!("  {}", item);
                    }
                    println!("Protected:");
                    for item in &config.protected {
                        println!("  {}", item);
                    }
                    println!("Targets:");
                    for (ext, dest) in &config.targets {
                        println!("  {ext} -> {dest}");
                    }
                }
            }
            Ok(())
        })()),
        //REMOVE
        Some(Commands::Remove { field, value }) => Some((|| -> anyhow::Result<()> {
            let mut config = load_config(config_path)?;
            match field {
                ConfigField::Watch => {
                    config.watch.retain(|item| item != value.as_str());
                }
                ConfigField::Rules => {
                    config.rules.retain(|item| item != value.as_str());
                }
                ConfigField::Protected => {
                    config.protected.retain(|item| item != value.as_str());
                }
                ConfigField::Targets => {
                    config.targets.remove(value);
                }
            }
            update_config(config_path, &config)?;
            Ok(())
        })()),

        None => None,
    }
}
