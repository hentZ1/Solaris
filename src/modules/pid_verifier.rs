use nix::unistd::Pid;
use std::{fs::*, path::PathBuf};
pub fn pid_verifier(path: &PathBuf) {
    if let Ok(content) = read_to_string(path) {
        match content.trim().parse::<i32>() {
            Ok(pid) => {
                let _ = nix::sys::signal::kill(
                    Pid::from_raw(pid),
                    nix::sys::signal::SIGTERM,
                );
            }
            Err(_) => {
                eprintln!("solaris: invalid PID file content at {:?}: '{}'", path, content.trim());
            }
        }
    }
}
