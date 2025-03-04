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
                homeserver_name: rng.gen::<Domain>().to_string(),
                identity_server: format!("https://id.{}/", rng.gen::<Domain>().to_string()),
                bind_address: String::from("localhost"),
                port: rng.gen_range(1024..=65535),
            },
            jwt: JwtConfig {
                issuer: format!("https://{}/base", rng.gen::<Domain>().to_string()),
            },
            database: DatabaseConfig {
                dev: DevDatabaseConfig {
                    hostname: format!("{}.{}", rng.gen::<FirstName>().to_string(), rng.gen::<Domain>().to_string()),
                    port: rng.gen_range(1024..=65535),
                    username: rng.gen::<FirstName>().to_string(),
                    password: rng.gen::<FirstName>().to_string(),
                    namespace: rng.gen::<FirstName>().to_string(),
                    database_name: rng.gen::<FirstName>().to_string(),
                },
                test: TestDatabaseConfig {
                    hostname: format!("{}.{}", rng.gen::<FirstName>().to_string(), rng.gen::<Domain>().to_string()),
                    port: rng.gen_range(1024..=65535),
                    username: rng.gen::<FirstName>().to_string(),
                    password: rng.gen::<FirstName>().to_string(),
                    namespace: rng.gen::<FirstName>().to_string(),
                }
            },
        }
    }
}

#[config]
#[derive(Debug, Default, Clone, Serialize)]
pub struct ServerConfig {
    pub homeserver_name: String,
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
    pub dev: DevDatabaseConfig,
    pub test: TestDatabaseConfig,
}

#[config]
#[derive(Debug, Default, Clone, Serialize)]
pub struct DevDatabaseConfig {
    pub hostname: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub namespace: String,
    pub database_name: String,
}

#[config]
#[derive(Debug, Default, Clone, Serialize)]
pub struct TestDatabaseConfig {
    pub hostname: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub namespace: String,
}

pub fn load(path: PathBuf) -> Result<Config, twelf::Error> {
    let path = path.into();
    let conf = Config::with_layers(&[
        Layer::Toml(path),
    ])?;

    Ok(conf)
}
