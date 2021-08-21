use std::env;
use std::fs;
use std::io::Read;
use tracing_subscriber::fmt::format::FmtSpan;
use warp::Filter;

fn env_or(k: &str, default: &str) -> String {
    env::var(k).unwrap_or_else(|_| default.to_string())
}

lazy_static::lazy_static! {
    pub static ref CONFIG: Config = Config::load();
}

#[derive(serde_derive::Deserialize)]
pub struct Config {
    pub version: String,
    pub host: String,
    pub port: u16,
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

    pub fn initialize(&self) {
        tracing::info!(
            target: "server",
            version = %CONFIG.version,
            host = %CONFIG.host,
            port = %CONFIG.port,
            log_format = %CONFIG.log_format,
            log_level = %CONFIG.log_level,
            this_host_name = %CONFIG.this_host_name,
            dev_server_port = %CONFIG.dev_server_port,
            "initialized config",
        );
    }
}

#[tokio::main]
async fn main() {
    // Filter traces based on the RUST_LOG env var, or, if it's not set,
    // default to show the output of the example.
    let filter = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "tracing=debug,warp=debug,server=debug".to_owned());

    // Configure the default `tracing` subscriber.
    // The `fmt` subscriber from the `tracing-subscriber` crate logs `tracing`
    // events to stdout. Other subscribers are available for integrating with
    // distributed tracing systems such as OpenTelemetry.
    tracing_subscriber::fmt()
        // Use the filter we built above to determine which traces to record.
        .with_env_filter(filter)
        // Record an event when each span closes. This can be used to time our
        // routes' durations!
        .with_span_events(FmtSpan::CLOSE)
        .init();

    async fn status() -> Result<impl warp::Reply, std::convert::Infallible> {
        tracing::info!(target: "server", "checking server status");
        let james = reqwest::get("https://james.kominick.com/status");
        let transfer = reqwest::get("https://transfer.kominick.com/status");
        let badge = reqwest::get("https://badge-cache.kominick.com/status");
        let paste = reqwest::get("https://paste.kominick.com/status");
        let soundlog = reqwest::get("https://soundlog.co/status");
        let slackat = reqwest::get("https://slackat.com/status");
        let (james, transfer, badge, paste, soundlog, slackat) =
            futures::try_join!(james, transfer, badge, paste, soundlog, slackat)
                .expect("status requests failed");
        let james_status = james.status().as_u16();
        let transfer_status = transfer.status().as_u16();
        let badge_status = badge.status().as_u16();
        let paste_status = paste.status().as_u16();
        let soundlog_status = soundlog.status().as_u16();
        let slackat_status = slackat.status().as_u16();

        Ok(warp::reply::json(&serde_json::json!({
            "james": james_status,
            "transfer": transfer_status,
            "badge": badge_status,
            "paste": paste_status,
            "soundlog": soundlog_status,
            "slackat": slackat_status,
        })))
    }

    CONFIG.initialize();
    let status = warp::get().and_then(status);
    let host = format!("{}:{}", CONFIG.host, CONFIG.port)
        .parse::<std::net::SocketAddr>()
        .expect("invalid host");
    warp::serve(status.with(warp::filters::log::log("server")))
        .run(host)
        .await;
}
