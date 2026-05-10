use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(name = "Solaris-CLI", about = "Rust-based file and folder organizer", long_about= None)]
pub struct Args {
    #[arg(
        short = 'r',
        long = "rules",
        help = "These are the files that you will tell the program to use.",
        num_args = 1..
    )]
    rules: Vec<String>,
    #[arg(
        short = 'w',
        long = "watch",
        help = "These are the directories that the daemon will 'monitor' to ensure the proper functioning and organization of the files immediately.",
        num_args = 1..
    )]
    watch: Vec<String>,
    #[arg(
        short = 'p',
        long = "protected",
        help = "These will be the folder and file paths that, at the user's choice, should not be viewed or modified in any way.",
        num_args = 1..
    )]
    protected: Vec<String>,
}
