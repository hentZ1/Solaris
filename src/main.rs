use anyhow::Context;
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
    let config_path = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("XDG directory not found"))?
        .join("solaris/config.toml");

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
    let stdout = File::create("/tmp/solaris.out").context("failed to create /tmp/solaris.out")?;
    let stderr = File::create("/tmp/solaris.err").context("failed to create /tmp/solaris.err")?;

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
        if let Err(e) = watcher(config.watch, tx) {
            eprintln!("solaris: watcher error: {:#}", e);
        }
    });

    applyrules_runner(rx, &config.targets);

    anyhow::Ok(())
}

//function to run the daemon
fn daemon_runner(pid_path: &PathBuf, stdout: File, stderr: File) -> anyhow::Result<()> {
    let daemonize = Daemonize::new()
        .pid_file(pid_path)
        .working_directory("/tmp/")
        .stdout(stdout)
        .stderr(stderr);

    daemonize.start().map_err(|e| anyhow::anyhow!(e))?;

    anyhow::Ok(())
}
//function to run the apply_rules
fn applyrules_runner(rx: Receiver<FsEvent>, targets: &HashMap<String, String>) {
    let mut notification = notify_rust::Notification::new()
        .appname("Solaris")
        .timeout(100)
        .clone();

    for event in rx {
        match &event {
            FsEvent::Created(path) => {
                if let Some(Action::Move { destination }) = apply_rules(&event, targets) {
                    let filename = match path.file_name() {
                        Some(name) => name,
                        None => {
                            eprintln!("solaris: cannot determine filename for path {:?}", path);
                            continue;
                        }
                    };
                    let dest_path = destination.join(filename);
                    if let Err(e) = std::fs::rename(path, &dest_path) {
                        eprintln!(
                            "solaris: failed to move {:?} to {:?}: {}",
                            path, dest_path, e
                        );
                        continue;
                    }
                    if let Err(e) = notification.summary("Solaris moved a file").show() {
                        eprintln!("solaris: notification error: {}", e);
                    }
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
        Some(Commands::List { field }) => Some(handle_list(config_path, field)),
        Some(Commands::Remove { field, value }) => Some(handle_remove(config_path, field, value)),
        None => None,
    }
}

fn handle_list(config_path: &PathBuf, field: &Option<ConfigField>) -> anyhow::Result<()> {
    let config = load_config(config_path)?;
    match field {
        Some(f) => print_field(&config, f, false),
        None => {
            for f in [ConfigField::Rules, ConfigField::Watch, ConfigField::Protected] {
                print_field(&config, &f, true);
            }
            print_field(&config, &ConfigField::Targets, true);
        }
    }
    Ok(())
}

fn handle_remove(config_path: &PathBuf, field: &ConfigField, value: &str) -> anyhow::Result<()> {
    let mut config = load_config(config_path)?;
    remove_field_value(&mut config, field, value);
    update_config(config_path, &config)?;
    Ok(())
}

fn print_field(config: &TomlContent, field: &ConfigField, header: bool) {
    let prefix = if header { "  " } else { "" };
    match field {
        ConfigField::Rules => {
            if header { println!("Rules:"); }
            for item in &config.rules { println!("{prefix}{item}"); }
        }
        ConfigField::Watch => {
            if header { println!("Watch:"); }
            for item in &config.watch { println!("{prefix}{item}"); }
        }
        ConfigField::Protected => {
            if header { println!("Protected:"); }
            for item in &config.protected { println!("{prefix}{item}"); }
        }
        ConfigField::Targets => {
            if header { println!("Targets:"); }
            for (ext, dest) in &config.targets { println!("{prefix}{ext} -> {dest}"); }
        }
    }
}

fn remove_field_value(config: &mut TomlContent, field: &ConfigField, value: &str) {
    match field {
        ConfigField::Watch => config.watch.retain(|item| item != value),
        ConfigField::Rules => config.rules.retain(|item| item != value),
        ConfigField::Protected => config.protected.retain(|item| item != value),
        ConfigField::Targets => { config.targets.remove(value); }
    }
}
