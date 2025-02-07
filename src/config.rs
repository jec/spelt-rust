use std::path::PathBuf;
use serde::Serialize;
use twelf::{config, Layer};

#[config]
#[derive(Debug, Default, Clone, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub jwt: JwtConfig,
    pub database: DatabaseConfig,
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
    pub uri: String,
    pub username: String,
    pub password: String,
}

pub fn load(path: PathBuf) -> Result<Config, twelf::Error> {
    let path = path.into();
    let conf = Config::with_layers(&[
        Layer::Toml(path),
    ])?;

    Ok(conf)
}
