use nix::unistd::Pid;
use std::{fs::*, path::PathBuf};
pub fn pid_verifier(path: &PathBuf) {
    match read_to_string(path) {
        Ok(content) => {
            let _ = nix::sys::signal::kill(
                Pid::from_raw(content.trim().parse::<i32>().unwrap()),
                nix::sys::signal::SIGTERM,
            );
        }
        Err(_) => {}
    }
}
