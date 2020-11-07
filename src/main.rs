#![recursion_limit = "1024"]

mod logger;
mod service;

use std::env;
use std::fs;
use std::io::Read;

use slog::{o, Drain};

fn env_or(k: &str, default: &str) -> String {
    env::var(k).unwrap_or_else(|_| default.to_string())
}

lazy_static::lazy_static! {
    pub static ref CONFIG: Config = Config::load();

    // The "base" logger that all crates should branch off of
    pub static ref BASE_LOG: slog::Logger = {
        let level: slog::Level = CONFIG.log_level
                .parse()
                .expect("invalid log_level");
        if CONFIG.log_format == "pretty" {
            let decorator = slog_term::TermDecorator::new().build();
            let drain = slog_term::CompactFormat::new(decorator).build().fuse();
            let drain = slog_async::Async::new(drain).build().fuse();
            let drain = slog::LevelFilter::new(drain, level).fuse();
            slog::Logger::root(drain, o!())
        } else {
            let drain = slog_json::Json::default(std::io::stderr()).fuse();
            let drain = slog_async::Async::new(drain).build().fuse();
            let drain = slog::LevelFilter::new(drain, level).fuse();
            slog::Logger::root(drain, o!())
        }
    };

    // Base logger
    pub static ref LOG: slog::Logger = BASE_LOG.new(slog::o!("app" => "dev"));
}

#[derive(serde_derive::Deserialize)]
pub struct Config {
    pub version: String,
    pub host: String,
    pub port: u16,
    pub ssl_port: u16,
    pub log_format: String,
    pub log_level: String,
    pub this_host_name: String,
    pub dev_server_port: u16,
}
impl Config {
    pub fn load() -> Self {
        let version = fs::File::open("commit_hash.txt")
            .map(|mut f| {
                let mut s = String::new();
                f.read_to_string(&mut s).expect("Error reading commit_hasg");
                s
            })
            .unwrap_or_else(|_| "unknown".to_string());
        Self {
            version,
            host: env_or("HOST", "0.0.0.0"),
            port: env_or("PORT", "80").parse().expect("invalid port"),
            ssl_port: env_or("SSL_PORT", "443").parse().expect("invalid port"),
            log_format: env_or("LOG_FORMAT", "json")
                .to_lowercase()
                .trim()
                .to_string(),
            log_level: env_or("LOG_LEVEL", "INFO"),
            this_host_name: env_or("THIS_HOST_NAME", "jaemk.me"),
            dev_server_port: env_or("DEV_SERVER_PORT", "3003")
                .parse()
                .expect("invalid port"),
        }
    }
    pub fn initialize(&self) -> anyhow::Result<()> {
        slog::info!(
            LOG, "initialized config";
            "version" => &CONFIG.version,
            "host" => &CONFIG.host,
            "port" => &CONFIG.port,
            "ssl_port" => &CONFIG.ssl_port,
            "log_format" => &CONFIG.log_format,
            "log_level" => &CONFIG.log_level,
            "this_host_name" => &CONFIG.this_host_name,
            "dev_server_port" => &CONFIG.dev_server_port,
        );
        Ok(())
    }
}

async fn run() -> anyhow::Result<()> {
    CONFIG.initialize()?;
    service::start().await?;
    Ok(())
}

#[actix_web::main]
async fn main() {
    if let Err(e) = run().await {
        slog::error!(LOG, "Error: {:?}", e);
    }
}
