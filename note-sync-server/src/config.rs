use serde::Deserialize;
use config::{Config, ConfigError, Environment, File};
use std::path::PathBuf;
use std::env;

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

/// 获取可执行文件所在目录
fn get_exe_dir() -> PathBuf {
    env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

/// 查找配置文件，按优先级返回
/// 1. 命令行参数指定的路径
/// 2. 环境变量 CLOUDMEMO_CONFIG_PATH 指定的路径
/// 3. 可执行文件同目录下的 config/{environment}.toml
/// 4. 当前工作目录下的 config/{environment}.toml
/// 5. 可执行文件同目录下的 config/default.toml
/// 6. 当前工作目录下的 config/default.toml
fn find_config_file(cli_path: Option<String>, environment: &str) -> PathBuf {
    // 1. 命令行参数优先级最高
    if let Some(path) = cli_path {
        return PathBuf::from(path);
    }

    // 2. 环境变量 CLOUDMEMO_CONFIG_PATH
    if let Ok(config_path) = env::var("CLOUDMEMO_CONFIG_PATH") {
        return PathBuf::from(config_path);
    }

    // 3-6. 尝试多个默认位置
    let config_name = format!("{}.toml", environment);
    let default_name = "default.toml";

    let exe_dir = get_exe_dir();
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // 按优先级尝试的位置
    let candidates = [
        // 环境特定配置
        exe_dir.join("config").join(&config_name),
        cwd.join("config").join(&config_name),
        // 默认配置
        exe_dir.join("config").join(default_name),
        cwd.join("config").join(default_name),
    ];

    for candidate in candidates {
        if candidate.exists() {
            tracing::info!("Found configuration file: {}", candidate.display());
            return candidate;
        }
    }

    // 如果都找不到，返回默认路径（会在后续加载时报错）
    exe_dir.join("config").join(&config_name)
}

impl AppConfig {
    /// 加载配置
    ///
    /// # 配置文件查找顺序
    /// 1. 命令行参数 `--config <path>`
    /// 2. 环境变量 `CLOUDMEMO_CONFIG_PATH`
    /// 3. 可执行文件同目录 `config/{environment}.toml`
    /// 4. 当前目录 `config/{environment}.toml`
    /// 5. 可执行文件同目录 `config/default.toml`
    /// 6. 当前目录 `config/default.toml`
    ///
    /// # 环境变量
    /// - `CLOUDMEMO_ENV`: 指定环境 (development/production)，默认为 development
    /// - `CLOUDMEMO_CONFIG_PATH`: 指定配置文件路径
    /// - `CLOUDMEMO_*`: 环境变量覆盖，如 `CLOUDMEMO_SERVER__PORT=8000`
    pub fn load(cli_config_path: Option<String>) -> Result<Self, ConfigError> {
        // 获取环境
        let environment = env::var("CLOUDMEMO_ENV")
            .unwrap_or_else(|_| "development".to_string())
            .to_lowercase();

        tracing::info!("Environment: {}", environment);

        // 查找配置文件
        let config_path = find_config_file(cli_config_path, &environment);

        if !config_path.exists() {
            tracing::error!("Configuration file not found: {}", config_path.display());
            tracing::error!("Please create a configuration file at one of these locations:");
            let exe_dir = get_exe_dir();
            tracing::error!("  - {}/config/{}.toml", exe_dir.display(), environment);
            tracing::error!("  - {}/config/default.toml", exe_dir.display());
            tracing::error!("Or specify it with:");
            tracing::error!("  - Environment variable: CLOUDMEMO_CONFIG_PATH=/path/to/config.toml");
            tracing::error!("  - Command line argument: --config /path/to/config.toml");
            return Err(ConfigError::NotFound(
                config_path.to_string_lossy().to_string()
            ));
        }

        tracing::info!("Loading configuration from: {}", config_path.display());

        let settings = Config::builder()
            // 加载配置文件
            .add_source(File::from(config_path))
            // 环境变量覆盖（CLOUDMEMO_ 前缀）
            // 例如：CLOUDMEMO_SERVER__PORT=8000, CLOUDMEMO_DATABASE__URL=mysql://...
            .add_source(Environment::with_prefix("CLOUDMEMO").separator("__"))
            .build()?;

        settings.try_deserialize()
    }

    /// 从指定路径加载配置
    pub fn load_from_path(path: &str) -> Result<Self, ConfigError> {
        tracing::info!("Loading configuration from: {}", path);

        let settings = Config::builder()
            .add_source(File::from(PathBuf::from(path)))
            .add_source(Environment::with_prefix("CLOUDMEMO").separator("__"))
            .build()?;

        settings.try_deserialize()
    }
}
