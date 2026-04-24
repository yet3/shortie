use chrono::{DateTime, Local};
use clap::Parser;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use rdev::{Event, EventType, Key as RKey, listen};
use shortie_common::{
    config::{Config, GroupedShorts, Short, parse_config},
    tokenizer::{FuncKind, ShortToken, ShortTokenizer},
};
use std::{
    collections::HashMap,
    fs,
    path::Path,
    sync::{Arc, Mutex},
};

fn backspace(enigo: &mut Enigo, times: usize) {
    for _ in 0..times {
        enigo
            .key(Key::Backspace, enigo::Direction::Press)
            .expect("to press backspace");
        enigo
            .key(Key::Backspace, enigo::Direction::Release)
            .expect("to release backspace");
    }
}

struct InputState {
    current_input: String,
    enigo: Enigo,
}

fn resolve_tokens(
    config: &Config,
    short: &Short,
    tokens: &Vec<ShortToken>,
    now: &DateTime<Local>,
    depth: usize,
) -> String {
    let mut output = String::new();

    if depth >= 10 {
        eprintln!("Reached content resolution limit: {}", depth);
        return output;
    }

    let tokenize_content = |str: &str| -> String {
        let mut tokenizer = ShortTokenizer::new(str);
        let tokens = tokenizer.tokenize().unwrap();
        resolve_tokens(config, short, &tokens, now, depth + 1)
    };

    for token in tokens {
        match token {
            ShortToken::Text { value, .. } => {
                output.push_str(value);
            }
            ShortToken::Func { func, .. } => match func {
                FuncKind::Embed { path } => {
                    let p = Path::new(&config.conf_paths[short.path_idx])
                        .parent()
                        .unwrap()
                        .join(path);
                    let str = fs::read_to_string(p).unwrap();
                    output.push_str(&tokenize_content(&str));
                }
                FuncKind::Now { format } => {
                    output.push_str(now.format(format.as_str()).to_string().as_str());
                }
                FuncKind::Var { name } => match short.vars.get(name).or(config.vars.get(name)) {
                    Some(var) => {
                        output.push_str(&tokenize_content(&var.value));
                    }
                    None => {
                        println!("Variable missing: \"{}\"", name)
                    }
                },
            },
            ShortToken::NewLine { .. } => {
                output.push('\n');
            }
        }
    }

    output
}

fn event_callback(event: Event, config: &Config, state: &Mutex<InputState>) {
    let mut s = state.lock().unwrap();

    match event.event_type {
        EventType::KeyPress(key) => match key {
            RKey::Return => {
                s.current_input.clear();
            }
            RKey::Escape => {
                s.current_input.clear();
            }
            RKey::Backspace => {
                if s.current_input.len() > 0 {
                    s.current_input.pop().unwrap();
                }
            }
            _ => {
                let key = event.name.unwrap();

                if key.len() == 1 {
                    let mut group: Option<&GroupedShorts> = None;
                    let groups = &config.groups;
                    s.current_input.push_str(&key);

                    let first_char = &s.current_input[..1];

                    if groups.contains_key(&key) {
                        s.current_input.clear();
                        s.current_input.push_str(&key);
                    } else if groups.contains_key(first_char) {
                        group = groups.get(first_char);
                    }

                    if s.current_input.len() > config.max_len {
                        s.current_input.remove(0);
                    }

                    if let Some(g) = group {
                        for short in &g.shorts {
                            if s.current_input == short.name {
                                s.current_input.clear();
                                backspace(&mut s.enigo, short.name.len());

                                let now: DateTime<Local> = Local::now();
                                let output = resolve_tokens(config, short, &short.tokens, &now, 0);

                                s.enigo.text(&output).unwrap();

                                if short.enter {
                                    s.enigo.key(Key::Return, Direction::Click).unwrap();
                                }

                                break;
                            }
                        }
                    }
                }
            }
        },
        _ => {}
    }
}

#[derive(Parser)]
struct Cli {
    /// Path to the directory containing .yaml config files
    #[arg(short, long)]
    config: String,
}

fn main() {
    let cli = Cli::parse();

    let config = parse_config(&cli.config).unwrap_or_else(|err| {
        println!("{err}");
        return Config {
            max_len: 0,
            groups: HashMap::new(),
            vars: HashMap::new(),
            conf_paths: vec![],
        };
    });

    let state = Arc::new(Mutex::new(InputState {
        current_input: String::with_capacity(config.max_len),
        enigo: Enigo::new(&Settings::default()).unwrap(),
    }));
    let state_ref = Arc::clone(&state);

    if let Err(e) = listen(move |event| event_callback(event, &config, &state_ref)) {
        println!("Listen error: {:?}", e);
    }
}
