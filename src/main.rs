use clap::Parser;
use solaris::modules::{
    clap::Args,
    config::{create_config, load_config},
};

fn main() {
    let args = Args::parse();
    let config_path = dirs::config_dir().unwrap().join("solaris/config.toml");
    create_config(&config_path).expect("It was not possible to create the configuration file.");
    load_config(config_path).expect("Error loading configuration file.");
}
