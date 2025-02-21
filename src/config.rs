use faker_rand::en_us::internet::Domain;
use faker_rand::en_us::names::FirstName;
use rand::Rng;
use serde::Serialize;
use std::path::PathBuf;
use twelf::{config, Layer};

#[config]
#[derive(Debug, Default, Clone, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub jwt: JwtConfig,
    pub database: DatabaseConfig,
}

impl Config {
    pub fn test() -> Self {
        let mut rng = rand::thread_rng();

        Config {
            server: ServerConfig {
                base_url: format!("https://{}/", rng.gen::<Domain>().to_string()),
                identity_server: format!("https://id.{}/", rng.gen::<Domain>().to_string()),
                bind_address: String::from("localhost"),
                port: rng.gen_range(1024..=65535),
            },
            jwt: JwtConfig {
                issuer: format!("https://{}/base", rng.gen::<Domain>().to_string()),
            },
            database: DatabaseConfig {
                dev_uri: Some(format!("https://{}:{}@{}/prod", rng.gen::<FirstName>().to_string(), rng.gen::<FirstName>().to_string(), rng.gen::<Domain>().to_string())),
                test_uri: Some(format!("https://{}:{}@{}/prod", rng.gen::<FirstName>().to_string(), rng.gen::<FirstName>().to_string(), rng.gen::<Domain>().to_string())),
            },
        }
    }
}

#[config]
#[derive(Debug, Default, Clone, Serialize)]
pub struct ServerConfig {
    pub base_url: String,
    pub identity_server: String,
    pub bind_address: String,
    pub port: u16,
}

#[config]
#[derive(Debug, Default, Clone, Serialize)]
pub struct JwtConfig {
    pub issuer: String,
}

#[config]
#[derive(Debug, Default, Clone, Serialize)]
pub struct DatabaseConfig {
    pub dev_uri: Option<String>,
    pub test_uri: Option<String>,
}

pub fn load(path: PathBuf) -> Result<Config, twelf::Error> {
    let path = path.into();
    let conf = Config::with_layers(&[
        Layer::Toml(path),
    ])?;

    Ok(conf)
}
