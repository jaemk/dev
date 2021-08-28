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

const TT: &str = "devserver";

#[derive(serde_derive::Deserialize)]
pub struct Config {
    pub version: String,
    pub host: String,
    pub port: u16,
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
        }
    }

    pub fn initialize(&self) {
        tracing::info!(
            target: TT,
            version = %CONFIG.version,
            host = %CONFIG.host,
            port = %CONFIG.port,
            "initialized config",
        );
    }
}

#[tokio::main]
async fn main() {
    let filter = std::env::var("LOG")
        .unwrap_or_else(|_| format!("tracing=debug,warp=debug,{tt}=debug", tt = TT));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(FmtSpan::CLOSE)
        .init();

    CONFIG.initialize();

    async fn status() -> Result<impl warp::Reply, std::convert::Infallible> {
        tracing::info!(target: TT, "checking server status");
        let james = reqwest::get("https://james.kominick.com/status");
        let transfer = reqwest::get("https://transfer.kominick.com/status");
        let badge = reqwest::get("https://badge-cache.kominick.com/status");
        let paste = reqwest::get("https://paste.kominick.com/status");
        let soundlog = reqwest::get("https://soundlog.co/status");
        let slackat = reqwest::get("https://slackat.com/status");
        let ritide = reqwest::get("https://ritide.kominick.com/status");
        let (james, transfer, badge, paste, soundlog, slackat, ritide) =
            futures::try_join!(james, transfer, badge, paste, soundlog, slackat, ritide)
                .expect("status requests failed");
        let james_status = james.status().as_u16();
        let transfer_status = transfer.status().as_u16();
        let badge_status = badge.status().as_u16();
        let paste_status = paste.status().as_u16();
        let soundlog_status = soundlog.status().as_u16();
        let slackat_status = slackat.status().as_u16();
        let ritide_status = ritide.status().as_u16();

        Ok(warp::reply::json(&serde_json::json!({
            "james": james_status,
            "transfer": transfer_status,
            "badge": badge_status,
            "paste": paste_status,
            "soundlog": soundlog_status,
            "slackat": slackat_status,
            "ritide": ritide_status,
        })))
    }

    let status = warp::get().and_then(status);
    let host = format!("{}:{}", CONFIG.host, CONFIG.port)
        .parse::<std::net::SocketAddr>()
        .expect("invalid host");
    warp::serve(status.with(warp::filters::log::log(TT)))
        .run(host)
        .await;
}
