use shortie_common::config::parse_config;
use std::{
    fs::{self, File},
    io::{Read, Seek, SeekFrom, Write},
};

use crate::daemon::{PID_FILE_PATH, is_pid_running, kill_daemon, spawn_daemon};

fn log_not_running() {
    println!("The shortie-daemon is not currently running");
}

fn log_running() {
    println!("The shortie-daemon is already running");
}

pub fn start_dameon() {
    if let Err(e) = parse_config() {
        panic!("{}", e);
    }

    let mut file = File::options()
        .write(true)
        .read(true)
        .create(true)
        .open(PID_FILE_PATH)
        .unwrap();

    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();

    if is_pid_running(&buf) {
        log_running();
        return;
    }

    let proccess_pid = spawn_daemon();

    file.seek(SeekFrom::Start(0)).unwrap();
    file.set_len(0).unwrap();
    file.write_all(proccess_pid.as_bytes()).unwrap();

    println!("Started shortie-daemon (PID: {})", proccess_pid);
}

pub fn stop_daemon() {
    let pid = fs::read_to_string(PID_FILE_PATH).unwrap_or(String::new());

    if pid.len() == 0 {
        log_not_running();
        return;
    }

    kill_daemon(&pid);
    fs::remove_file(PID_FILE_PATH).unwrap_or_else(|err| {
        panic!("Error deleting shortie-daemon pid file: {}", err);
    })
}

pub fn print_daemon_status() {
    let pid = fs::read_to_string(PID_FILE_PATH).unwrap_or(String::new());
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
