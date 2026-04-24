use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, Read},
    vec,
};

use serde::Deserialize;
use thiserror::Error;

use crate::tokenizer::{ShortToken, ShortTokenizer, TokenizerError};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShortKind {
    Text,
    File,
}

#[derive(Debug)]
pub struct Short {
    pub path_idx: usize,
    pub name: String,
    pub tokens: Vec<ShortToken>,
    pub kind: ShortKind,
    pub vars: HashMap<String, Var>,
    pub enter: bool
}

#[derive(Debug, Deserialize)]
pub struct Var {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct ConfigShort {
    pub name: String,
    pub content: String,
    pub kind: Option<ShortKind>,
    pub vars: Option<Vec<Var>>,
    pub enter: Option<bool>
}

#[derive(Debug)]
pub struct GroupedShorts {
    pub shorts: Vec<Short>,
}

#[derive(Debug)]
pub struct ConfigFile {
    pub path: String,
    pub vars: Option<Vec<Var>>,
    pub prefix: Option<String>,
    pub shorts: Vec<ConfigShort>,
}

#[derive(Debug, Deserialize)]
pub struct SerdeConfigFile {
    pub vars: Option<Vec<Var>>,
    pub prefix: Option<String>,
    pub shorts: Vec<ConfigShort>,
}

#[derive(Debug)]
pub struct Config {
    pub max_len: usize,
    pub groups: HashMap<String, GroupedShorts>,
    pub vars: HashMap<String, Var>,
    pub conf_paths: Vec<String>,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to open config file \"{path}\"")]
    OpenConfig {
        path: String,
        #[source]
        source: io::Error,
    },

    #[error("Failed to read config file \"{path}\"")]
    ReadConfig {
        path: String,
        #[source]
        source: io::Error,
    },

    #[error("Error in config file \"{path}\"\n{render}")]
    DeserializeConfig {
        path: String,
        render: String,
        #[source]
        source: serde_saphyr::Error,
    },

    #[error("Error reading directory  \"{path}\"")]
    ReadConfigsDir {
        path: String,
        #[source]
        source: io::Error,
    },
}

pub fn parse_config_file(path: &str) -> Result<SerdeConfigFile, ConfigError> {
    let mut file = File::open(path).map_err(|source| ConfigError::OpenConfig {
        path: path.to_string(),
        source,
    })?;

    let mut buff = String::new();
    file.read_to_string(&mut buff)
        .map_err(|source| ConfigError::ReadConfig {
            path: path.to_string(),
            source,
        })?;

    let config =
        serde_saphyr::from_str(&buff).map_err(|source| ConfigError::DeserializeConfig {
            path: path.to_string(),
            render: source.render(),
            source,
        })?;

    Ok(config)
}

pub fn read_configs_in_dir(path: &str) -> Result<Vec<ConfigFile>, ConfigError> {
    let mut configs: Vec<ConfigFile> = vec![];

    let dir = fs::read_dir(path).map_err(|source| ConfigError::ReadConfigsDir {
        path: path.to_string(),
        source,
    })?;

    for entry in dir.flatten() {
        let file_type = entry.file_type().unwrap();
        let file_path = entry.path();

        if file_type.is_dir() {
            configs.extend(read_configs_in_dir(file_path.to_str().unwrap())?);
        } else if file_type.is_file() {
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();

            if file_name.ends_with(".yaml") || file_name.ends_with(".yml") {
                let config = parse_config_file(file_path.to_str().unwrap())?;
                configs.push(ConfigFile {
                    path: file_path.to_string_lossy().to_string(),
                    vars: config.vars,
                    prefix: config.prefix,
                    shorts: config.shorts,
                });
            }
        }
    }

    Ok(configs)
}

pub fn parse_config(config_path: &str) -> Result<Config, ConfigError> {
    let configs = read_configs_in_dir(config_path)?;

    let mut conf_paths: Vec<String> = vec![];
    let mut global_vars: HashMap<String, Var> = HashMap::new();
    let mut groups: HashMap<String, GroupedShorts> = HashMap::new();
    let mut max_len: usize = 0;

    for conf in configs {
        conf_paths.push(conf.path);
        if let Some(v) = conf.vars {
            for var in v {
                global_vars.insert(var.name.clone(), var);
            }
        }

        for short in conf.shorts {
            let (prefix, is_prefix_custom) = if let Some(p) = &conf.prefix {
                (p.to_string(), true)
            } else {
                (short.name[..1].to_string(), false)
            };

            let mut short_name = short.name.clone();
            if is_prefix_custom {
                short_name.insert_str(0, &prefix);
            }

            if short_name.len() > max_len {
                max_len = short_name.len();
            }

            let kind = if let Some(k) = short.kind {
                k
            } else {
                ShortKind::Text
            };

            let mut vars: HashMap<String, Var> = HashMap::new();
            if let Some(v) = short.vars {
                for var in v {
                    vars.insert(var.name.clone(), var);
                }
            }

            let mut tokenizer = ShortTokenizer::new(short.content);

            match tokenizer.tokenize() {
                Ok(tokens) => {
                    let short = Short {
                        path_idx: conf_paths.len() - 1,
                        name: short_name,
                        kind,
                        vars,
                        tokens,
                        enter: short.enter.unwrap_or(false),
                    };

                    if groups.contains_key(&prefix) {
                        let group = groups.get_mut(&prefix).unwrap();
                        group.shorts.push(short);
                    } else {
                        groups.insert(
                            prefix.clone(),
                            GroupedShorts {
                                shorts: vec![short],
                            },
                        );
                    }
                }
                Err(e) => match e {
                    TokenizerError::MissingArgs { .. } => {
                        eprintln!(
                            "{}",
                            e.render_missing_args(conf_paths.last().unwrap(), short.name)
                        );
                    }
                    _ => {
                        eprintln!("{e}");
                    }
                },
            }
        }
    }

    Ok(Config {
        groups,
        max_len,
        conf_paths,
        vars: global_vars,
    })
}
