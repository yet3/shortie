use core::panic;
use std::process::{Command, Stdio};

pub static PID_FILE_PATH: &str = "/tmp/shortiedapp.pid";

pub fn is_pid_running(pid: &str) -> bool {
    if pid.len() == 0 {
        return false;
    }

    Command::new("kill")
        .args(["-0", &pid])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn spawn_daemon() -> String {
    let child = Command::new("cargo")
        .args(["run", "-p", "shortie-daemon"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|err| {
            panic!("Error starting shortie-daemon: {}", err);
        });

    let pid = child.id().to_string();
    std::mem::forget(child);

    return pid;
}

pub fn kill_daemon(pid: &str) {
    Command::new("kill")
        .args(["-9", pid])
        .output()
        .map(|o| o.status.success())
        .unwrap_or_else(|err| {
            panic!("Error starting shortie-daemon: {}", err);
        });
}
