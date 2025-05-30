use fake::faker::internet::en::{DomainSuffix, Password, Username};
use fake::faker::lorem::en::Word;
use fake::faker::name::en::FirstName;
use fake::Fake;
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
        let mut rng = rand::rng();

        Config {
            server: ServerConfig {
                homeserver_name: format!("{}.{}", Word().fake::<String>(), DomainSuffix().fake::<String>()),
                identity_server: format!("https://id.{}.{}/", Word().fake::<String>(), DomainSuffix().fake::<String>()),
                bind_address: String::from("localhost"),
                port: rng.random_range(1024..=65535),
            },
            jwt: JwtConfig {
                issuer: format!("https://{}.{}/base", Word().fake::<String>(), DomainSuffix().fake::<String>()),
            },
            database: DatabaseConfig {
                dev: DevDatabaseConfig {
                    hostname: format!("{}.{}.{}", FirstName().fake::<String>(), Word().fake::<String>(), DomainSuffix().fake::<String>()),
                    port: rng.random_range(1024..=65535),
                    username: Username().fake(),
                    password: Password(8..20).fake(),
                    namespace: Word().fake(),
                    database_name: Word().fake(),
                },
                test: TestDatabaseConfig {
                    hostname: format!("{}.{}.{}", FirstName().fake::<String>(), Word().fake::<String>(), DomainSuffix().fake::<String>()),
                    port: rng.random_range(1024..=65535),
                    username: Username().fake(),
                    password: Password(8..20).fake(),
                    namespace: Word().fake(),
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
