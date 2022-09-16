use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub patterns: Vec<String>,
    pub eps: u128,
}

fn read_config_file(path: PathBuf) -> String {
    fs::read_to_string(path).expect("Unable to read config file")
}

pub fn get_config_yaml(path: PathBuf) -> Config {
    match serde_yaml::from_str(&read_config_file(path)) {
        Ok(config) => config,
        Err(..) => panic!("Unable to read config file as yaml!"),
    }
}
