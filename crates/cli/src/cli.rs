use crate::daemon::{DaemonOpts, is_pid_running, kill_daemon, spawn_daemon};
use shortie_common::config::parse_config;
use std::{
    fs::{self, File},
    io::{Read, Seek, SeekFrom, Write},
};

fn log_not_running() {
    println!("The shortie-daemon is not currently running");
}

fn log_running() {
    println!("The shortie-daemon is already running");
}

pub fn start_dameon(opts: &DaemonOpts) {
    if let Err(e) = parse_config(&opts.config) {
        panic!("{}", e);
    }
    let mut file = File::options()
        .write(true)
        .read(true)
        .create(true)
        .open(&opts.pid_file_path())
        .unwrap();

    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();

    if is_pid_running(&buf) {
        if !opts.silent {
            log_running();
        }
        return;
    }

    let proccess_pid = spawn_daemon(opts);

    file.seek(SeekFrom::Start(0)).unwrap();
    file.set_len(0).unwrap();
    file.write_all(proccess_pid.as_bytes()).unwrap();

    if !opts.silent {
        println!("Started shortie-daemon (PID: {})", proccess_pid);
    }
}

pub fn stop_daemon(opts: &DaemonOpts) {
    let pid = fs::read_to_string(opts.pid_file_path()).unwrap_or(String::new());

    if pid.len() == 0 {
        if !opts.silent {
            log_not_running();
        }
        return;
    }

    kill_daemon(&pid);
    fs::remove_file(opts.pid_file_path()).unwrap_or_else(|err| {
        panic!("Error deleting shortie-daemon pid file: {}", err);
    })
}

pub fn print_daemon_status(opts: &DaemonOpts) {
    let pid = fs::read_to_string(opts.pid_file_path()).unwrap_or(String::new());
    println!("========= Shortie =========");
    println!(
        "Status: {}",
        if is_pid_running(&pid) {
            "running"
        } else {
            "not running"
        }
    );
    println!("PID: {}", if is_pid_running(&pid) { &pid } else { "-" });
    println!("===========================");
}
