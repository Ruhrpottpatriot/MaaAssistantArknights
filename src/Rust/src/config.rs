use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::IpAddr;
use std::path::Path;

lazy_static! {
    pub static ref CONFIG: Config = {
        let config_file = Path::new("./server_config.json");
        if !config_file.exists() {
            let default_config = include_str!("../server_config.json");
            fs::write(config_file, default_config).unwrap();
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
    pub address: IpAddr,

    pub port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Resource {
    pub path: String
}


#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn deserialize_resource() {
        let data = r#"{
            "path": "./"
        }"#;
        let r: Resource = serde_json::from_str(data).unwrap();
        assert_eq!(r.path, "./");
    }

    #[test]
    fn serialize_resource() {
        let r = Resource {
            path: "./".to_string(),
        };
        let data = serde_json::to_string(&r).unwrap();
        assert_eq!(data, r#"{"path":"./"}"#);
    }

    #[test]
    fn deserialize_server() {
        let data = r#"{
            "address": "127.0.0.1",
            "port": 8080
        }"#;
        let s: Server = serde_json::from_str(data).unwrap();
        assert_eq!(Ok(s.address), "127.0.0.1".parse());
        assert_eq!(s.port, 8080);
    }

    #[test]
    fn serialize_server() {
        let s = Server {
            address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            port: 8080,
        };
        let data = serde_json::to_string(&s).unwrap();
        let actual = r#"{"address":"127.0.0.1","port":8080}"#;
        assert_eq!(data, actual);
    }

    #[test]
    fn deserialize_database() {
        let data = r#"{
            "path": "./db.sqlite",
            "drop_on_start_up": true
        }"#;

        let d: Database = serde_json::from_str(data).unwrap();
        assert_eq!(d.path, "./db.sqlite");
        assert_eq!(d.drop_on_start_up, true);
    }

    #[test]
    fn serialize_database() {
        let d = Database {
            path: "./db.sqlite".to_string(),
            drop_on_start_up: true,
        };
        let data = serde_json::to_string(&d).unwrap();
        let actual = r#"{"path":"./db.sqlite","drop_on_start_up":true}"#;
        assert_eq!(data, actual);
    }

    #[test]
    fn deserialize_config() {
        let data = r#"{
            "server": {
                "address": "127.0.0.1",
                "port": 8080
            },
            "database": {
                "path": "./db.sqlite",
                "drop_on_start_up": true
            },
            "resource": {
                "path": "./"
            }
        }"#;

        let c: Config = serde_json::from_str(data).unwrap();
        assert_eq!(Ok(c.server.address), "127.0.0.1".parse());
        assert_eq!(c.server.port, 8080);
        assert_eq!(c.database.path, "./db.sqlite");
        assert_eq!(c.database.drop_on_start_up, true);
        assert_eq!(c.resource.path, "./");
    }

    #[test]
    fn serialize_config() {
        let c = Config {
            server: Server {
                address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                port: 8080,
            },
            database: Database {
                path: "./db.sqlite".to_string(),
                drop_on_start_up: true,
            },
            resource: Resource {
                path: "./".to_string(),
            },
        };
        let data = serde_json::to_string(&c).unwrap();
        let actual = r#"{
            "server": {
                "address": "127.0.0.1",
                "port": 8080
            },
            "database": {
                "path": "./db.sqlite",
                "drop_on_start_up": true
            },
            "resource": {
                "path": "./"
            }
        }"#;
        let actual = actual.replace(['\n', ' '], "");
        assert_eq!(data, actual);
    }
}
