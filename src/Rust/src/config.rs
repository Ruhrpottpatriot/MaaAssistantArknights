use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

lazy_static! {
    pub static ref CONFIG: Config = {
        let config_file = Path::new("./server_config.json");
        if !config_file.exists() {
            let default_config = include_str!("../server_config.json");
            fs::write(config_file,default_config).unwrap();
        }
        let s = fs::read(config_file).unwrap();
        let r: Config = serde_json::from_slice(&s).unwrap();
        r
    };
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub server: Server,

    pub database: Database,

    pub resource: Resource,
}

#[derive(Serialize, Deserialize)]
pub struct Database {
    pub path: String,

    pub drop_on_start_up: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Server {
    pub address: String,

    pub port: u16,
}

#[derive(Serialize, Deserialize)]
pub struct Resource {
    pub path: String,
}
