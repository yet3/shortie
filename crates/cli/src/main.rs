mod cli;
mod daemon;
use crate::cli::{print_daemon_status, start_dameon, stop_daemon};
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Cmds,
}

#[derive(Subcommand)]
enum Cmds {
    Start,
    Stop,
    Reload,
    Status,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Cmds::Start => start_dameon(),
        Cmds::Stop => stop_daemon(),
        Cmds::Reload => {
            stop_daemon();
            start_dameon();
        }
        Cmds::Status => {
            print_daemon_status();
        }
    };
}
