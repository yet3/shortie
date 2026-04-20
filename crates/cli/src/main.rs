mod cli;
mod daemon;
use crate::{
    cli::{print_daemon_status, start_dameon, stop_daemon},
    daemon::DaemonOpts,
};
use clap::{Args, Error, Parser, Subcommand};
use shortie_common::{config::parse_config, tokenizer::ShortTokenizer};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Cmds,
}

#[derive(Args)]
struct CmdArgs {
    /// Path to the directory containing .yaml config files
    #[arg(short, long, default_value = None)]
    config: Option<String>,

    /// Path to the directory containing temporary .pid file
    #[arg(short, long, default_value = None)]
    pid: Option<String>,

    /// Disable cli output
    #[arg(short, long, default_value_t = false)]
    silent: bool,
}

#[derive(Subcommand)]
enum Cmds {
    /// Start shortie-daemon
    Parse {
        #[command(flatten)]
        args: CmdArgs,
    },
    Start {
        #[command(flatten)]
        args: CmdArgs,
    },
    /// Stop shortie-daemon
    Stop {
        #[command(flatten)]
        args: CmdArgs,
    },
    /// Reload shortie-daemon
    Reload {
        #[command(flatten)]
        args: CmdArgs,
    },
    /// See status of shortie-daemon
    Status {
        #[command(flatten)]
        args: CmdArgs,
    },
}

fn make_opts(args: CmdArgs) -> DaemonOpts {
    let mut opts = DaemonOpts::new();

    if let Some(config) = args.config {
        opts.config = config;
    }

    if let Some(pid) = args.pid {
        opts.pid = pid;
    }

    opts.silent = args.silent;

    opts
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Cmds::Parse { args } => {
            let opts = &make_opts(args);

            // let mut t = ShortTokenizer::new("this {{embed ./tak}} is\na embed");
            // let mut t = ShortTokenizer::new("this {var tak} {embed ./tak} is\na embed");
            let mut t = ShortTokenizer::new("aa: {now} asdf -> {{now %d-%m}} this is\na embed");
            let tokens = t.tokenize();
            dbg!("TOKENS: {:?}", tokens);

            match parse_config(&opts.config) {
                Ok(config) => {
                    println!("{:?}", config);
                }
                Err(e) => {
                    panic!("{}", e);
                }
            }
        }
        Cmds::Start { args } => {
            start_dameon(&make_opts(args));
        }
        Cmds::Stop { args } => {
            stop_daemon(&make_opts(args));
        }
        Cmds::Reload { args } => {
            let opts = &make_opts(args);
            stop_daemon(&opts);
            start_dameon(&opts);
        }
        Cmds::Status { args } => {
            print_daemon_status(&make_opts(args));
        }
    };
}
