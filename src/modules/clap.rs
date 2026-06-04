use clap::Parser;
use std::{path::PathBuf, str::FromStr};

#[derive(Debug, Clone)]

//creates the auxiliary structure for the args structure parameter
pub struct AuxTargetRule {
    pub extensions: Vec<String>,
    pub destination: PathBuf,
}

//treats the string it receives and splits the string infos between extensions and destinations
impl FromStr for AuxTargetRule {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.splitn(2, ':').collect();

        if parts.len() != 2 {
            return Err(format!(
                "Invalid format: expected 'ext1,ext2:path', received '{}'",
                s
            ));
        }
        let extensions = parts[0].split(',').map(|e| e.trim().to_string()).collect();

        let destination = PathBuf::from(parts[1].trim());

        let destination = if destination.starts_with("~") {
            dirs::home_dir()
                .unwrap()
                .join(destination.strip_prefix("~").unwrap())
        } else {
            destination
        };

        Ok(AuxTargetRule {
            extensions,
            destination,
        })
    }
}

#[derive(Parser, Debug, Clone)]
#[command(name = "Solaris-CLI", about = "Rust-based file and folder organizer", long_about= None)]
pub struct Args {
    #[arg(
        short = 'r',
        long = "rules",
        help = "These are the files that you will tell the program to use.",
        num_args = 1..
    )]
    pub rules: Vec<String>,
    #[arg(
        short = 'w',
        long = "watch",
        help = "These are the directories that the daemon will 'monitor' to ensure the proper functioning and organization of the files immediately.",
        num_args = 1..
    )]
    pub watch: Vec<String>,
    #[arg(
        short = 'p',
        long = "protected",
        help = "These will be the folder and file paths that, at the user's choice, should not be viewed or modified in any way.",
        num_args = 1..
    )]
    pub protected: Vec<String>,
    #[arg(
        short = 't',
        long = "target",
        help = "Map file extensions to destination folders. Format: 'ext1,ext2:/path'. Example: -t zip,tar:/home/user/Archives",
        value_parser = clap::value_parser!(AuxTargetRule)
    )]
    pub targets: Vec<AuxTargetRule>,
}
