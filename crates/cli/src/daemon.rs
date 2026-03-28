use core::panic;
use std::{
    env::current_exe,
    path::PathBuf,
    process::{Command, Stdio},
};

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

pub struct DaemonOpts {
    pub config: String,
    pub pid: String,
}

impl DaemonOpts {
    pub fn new() -> Self {
        DaemonOpts {
            config: String::from(
                PathBuf::from(dirs::home_dir().expect("Failed to get path from $HOME"))
                    .join(".config/shortie")
                    .to_string_lossy()
                    .to_string(),
            ),
            pid: String::from("/tmp/shortied.pid"),
        }
    }

    pub fn pid_file_path(&self) -> &str {
        &self.pid
    }
}

pub fn spawn_daemon(opts: &DaemonOpts) -> String {
    let mut cmd = "cargo".to_string();
    let mut args = vec!["run", "-p", "shortie-daemon", "--"];
    if !cfg!(debug_assertions) {
        let exe_dir = current_exe()
            .expect("Failed to get current exe")
            .parent()
            .expect("No parent dir")
            .to_owned();

        cmd = exe_dir.join("shortied").to_string_lossy().to_string();

        args.clear();
    }

    args.push("--config");
    args.push(&opts.config);

    let child = Command::new(cmd)
        .args(args)
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
