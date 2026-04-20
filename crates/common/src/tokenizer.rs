use std::fmt::Write;
use std::{fs, vec};
use thiserror::Error;

#[derive(Debug)]
pub struct ShortTokenDebug {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug)]
pub enum FuncKind {
    Embed { path: String },
    Now { format: String },
    Var { name: String },
}

#[derive(Debug)]
pub enum ShortToken {
    Text {
        debug: ShortTokenDebug,
        value: String,
    },
    Func {
        debug: ShortTokenDebug,
        func: FuncKind,
    },
    NewLine {
        debug: ShortTokenDebug,
    },
}

pub struct ShortTokenizer {
    caret: usize,
    chars: Vec<char>,
}

#[derive(Error, Debug)]
pub enum TokenizerError {
    #[error("Tokenizer Error: Missing arguments in function: \"{func_name}\"")]
    MissingArgs {
        func_name: String,
        chars: Vec<char>,
        debug: ShortTokenDebug,
        msg: Vec<String>,
    },

    #[error("Tokenizer Error: Unknown function: \"{func_name}\"")]
    UnknownFunc {
        func_name: String,
        chars: Vec<char>,
        debug: ShortTokenDebug,
        msg: Vec<String>,
    },
}

impl TokenizerError {
    pub fn render_missing_args(
        &self,
        conf_path: impl Into<String>,
        short_name: impl Into<String>,
    ) -> String {
        if let Self::MissingArgs {
            msg, chars, debug, ..
        } = self
        {
            let snippet: String = chars[debug.start.saturating_sub(5).max(2)..debug.end]
                .iter()
                .collect();

            let mut output = String::new();
            write!(output, "Error in file: \"{}\"\n", conf_path.into()).unwrap();
            write!(output, "shorts:\n").unwrap();
            write!(output, "  - name: \"{}\"\n", short_name.into()).unwrap();

            let prefix = "    content: \"..";
            let pointer = " ".repeat(prefix.len() + snippet.len() - 2);
            write!(
                output,
                "{prefix}{snippet}..\"\n{pointer}^ {}",
                msg.iter()
                    .map(|s| s.as_ref())
                    .collect::<Vec<&str>>()
                    .join(format!("\n  {pointer}").as_str())
            )
            .unwrap();

            output
        } else {
            panic!("called on wrong variant")
        }
    }
}

impl ShortTokenizer {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            caret: 0,
            chars: content.into().chars().collect(),
        }
    }

    fn get_char(&self) -> Option<char> {
        if self.caret < self.chars.len() - 1 {
            return Some(self.chars[self.caret]);
        }
        None
    }

    fn next_char(&mut self) {
        self.caret += 1;
    }

    fn next_token(&mut self) -> Result<Option<ShortToken>, TokenizerError> {
        let Some(char) = self.get_char() else {
            return Ok(None);
        };

        match char {
            '{' => {
                let mut debug = ShortTokenDebug {
                    start: self.caret,
                    end: 0,
                };

                let mut caret = self.caret + 1;
                let mut arg = String::new();
                let mut args: Vec<String> = vec![];

                let mut only_open = 1;
                while caret < self.chars.len() {
                    let char = self.chars[caret];
                    if char == '{' || only_open > 1 {
                        if char != '{' {
                            break;
                        }
                        only_open += 1;
                    } else if char == '}' {
                        caret += 1;
                        break;
                    } else if char == ' ' {
                        args.push(arg.clone());
                        arg.clear();
                    } else {
                        arg.push(char);
                    }
                    caret += 1;
                }

                self.caret = caret;
                debug.end = caret;

                if only_open > 1 {
                    return Ok(Some(ShortToken::Text {
                        debug,
                        value: "{".repeat(only_open),
                    }));
                }

                if arg.len() > 0 {
                    args.push(arg.clone());
                }

                let func_name = args[0].as_str();

                match func_name {
                    "var" => {
                        if args.len() < 2 {
                            return Err(TokenizerError::MissingArgs {
                                chars: self.chars.clone(),
                                debug: debug,
                                func_name: func_name.into(),
                                msg: vec![
                                    "Missing variable name!".into(),
                                    "(example: {var first_name})".into(),
                                ],
                            });
                        }

                        Ok(Some(ShortToken::Func {
                            debug,
                            func: FuncKind::Var {
                                name: args[1..].join(" ").clone(),
                            },
                        }))
                    }
                    "embed" => {
                        if args.len() < 2 {
                            return Err(TokenizerError::MissingArgs {
                                chars: self.chars.clone(),
                                debug: debug,
                                func_name: func_name.into(),
                                msg: vec![
                                    "Missing file path!".into(),
                                    "(example: {embed ./templates/email_1.txt})".into(),
                                ],
                            });
                        }

                        Ok(Some(ShortToken::Func {
                            debug,
                            func: FuncKind::Embed {
                                path: args[1..].join(" ").clone(),
                            },
                        }))
                    }
                    "now" => {
                        let mut format = String::from("%H:%M:%S %d-%m-%Y");

                        if args.len() >= 2 {
                            format = args[1..].join(" ").clone();
                        }

                        Ok(Some(ShortToken::Func {
                            debug,
                            func: FuncKind::Now { format },
                        }))
                    }
                    _ => Err(TokenizerError::UnknownFunc {
                        chars: self.chars.clone(),
                        debug: debug,
                        func_name: func_name.into(),
                        msg: vec![
                            "Unknown function!".into(),
                            "(function: now; embed; var)".into(),
                        ],
                    }),
                }
            }
            '\n' => {
                self.next_char();
                Ok(Some(ShortToken::NewLine {
                    debug: ShortTokenDebug {
                        start: self.caret - 1,
                        end: self.caret,
                    },
                }))
            }
            '\r' => {
                self.next_char();
                if let Some(c) = self.get_char()
                    && c == '\n'
                {
                    self.next_char();
                    return Ok(Some(ShortToken::NewLine {
                        debug: ShortTokenDebug {
                            start: self.caret - 2,
                            end: self.caret,
                        },
                    }));
                }

                self.next_token()
            }
            _ => {
                let mut str = String::new();
                let start_caret = self.caret;
                let mut caret = start_caret;

                while caret < self.chars.len() {
                    let char = self.chars[caret];
                    if char == '{' || char == '\0' || char == '\n' {
                        break;
                    }
                    str.push(char);
                    caret += 1;
                }

                self.caret = caret;

                Ok(Some(ShortToken::Text {
                    debug: ShortTokenDebug {
                        start: start_caret,
                        end: caret,
                    },
                    value: str,
                }))
            }
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<ShortToken>, TokenizerError> {
        let mut tokens: Vec<ShortToken> = vec![];

        while let Some(token) = self.next_token()? {
            tokens.push(token);
        }

        Ok(tokens)
    }
}
