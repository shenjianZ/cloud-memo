use serde::Deserialize;
use config::{Config, ConfigError, Environment};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    #[serde(default = "default_jwt_expiration_days")]
    pub jwt_expiration_days: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RedisConfig {
    pub url: String,
    #[serde(default)]
    pub password: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub redis: RedisConfig,
}

fn default_max_connections() -> u32 {
    10
}

fn default_jwt_expiration_days() -> i64 {
    7
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let builder = Config::builder()
            // 1. 加载默认配置
            .add_source(config::File::from(
                PathBuf::from("config/default.toml")
            ));

        // 2. 加载环境特定配置（可选）
        #[cfg(debug_assertions)]
        let builder = builder.add_source(
            config::File::from(
                PathBuf::from("config/development.toml")
            ).required(false)
        );

        #[cfg(not(debug_assertions))]
        let builder = builder.add_source(
            config::File::from(
                PathBuf::from("config/production.toml")
            ).required(false)
        );

        // 3. 环境变量覆盖（APP_ 前缀）
        // 例如：APP_DATABASE_URL, APP_AUTH_JWT_SECRET
        let settings = builder
            .add_source(Environment::with_prefix("APP").separator("__"))
            .build()?;

        settings.try_deserialize()
    }
}
