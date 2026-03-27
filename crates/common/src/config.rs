use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, Read},
};

use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct Short {
    pub name: String,
    pub output: String,
}

#[derive(Debug)]
pub struct GroupedShorts {
    pub shorts: Vec<Short>,
}

#[derive(Debug, Deserialize)]
pub struct ConfigFile {
    prefix: Option<String>,
    shorts: Vec<Short>,
}

pub struct Config {
    pub max_len: usize,
    pub groups: HashMap<String, GroupedShorts>,
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

pub fn parse_config_file(path: &str) -> Result<ConfigFile, ConfigError> {
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
                configs.push(config);
            }
        }
    }

    Ok(configs)
}

pub fn parse_config(config_path: &str) -> Result<Config, ConfigError> {
    let configs = read_configs_in_dir(config_path)?;

    let mut groups: HashMap<String, GroupedShorts> = HashMap::new();
    let mut max_len: usize = 0;

    for conf in configs {
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

            if groups.contains_key(&prefix) {
                let group = groups.get_mut(&prefix).unwrap();
                group.shorts.push(Short {
                    name: short_name,
                    output: short.output,
                });
            } else {
                groups.insert(
                    prefix.clone(),
                    GroupedShorts {
                        shorts: vec![Short {
                            name: short_name,
                            output: short.output,
                        }],
                    },
                );
            }
        }
    }

    Ok(Config { groups, max_len })
}
