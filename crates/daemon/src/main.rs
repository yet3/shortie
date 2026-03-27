use enigo::{Enigo, Key, Keyboard, Settings};
use rdev::{Event, EventType, Key as RKey, listen};
use shortie_common::config::{Config, GroupedShorts, parse_config};
use std::{
    collections::HashMap,
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

                    if s.current_input.len() >= config.max_len {
                        s.current_input.remove(0);
                    }

                    if let Some(g) = group {
                        for short in &g.shorts {
                            if s.current_input == short.name {
                                s.current_input.clear();
                                backspace(&mut s.enigo, short.name.len());
                                s.enigo.text(&short.output).unwrap();
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

fn main() {
    let config = parse_config().unwrap_or_else(|err| {
        println!("{err}");
        return Config {
            max_len: 0,
            groups: HashMap::new(),
        }
        // panic!("{err}");
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
