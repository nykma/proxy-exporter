use crate::config::Upstream;
use crate::metrics;
use futures_util::StreamExt;
use serde::Deserialize;
use std::time::Duration;
use tokio_tungstenite::tungstenite::Message;

#[derive(Deserialize)]
struct Payload {
    #[serde(rename = "upTotal")]
    up_total: u64,
    #[serde(rename = "downTotal")]
    down_total: u64,
}

pub async fn run(config: Upstream) {
    loop {
        if let Err(e) = collect_once(&config).await {
            eprintln!("[{}] error: {}, retrying in 5s...", config.name, e);
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }
}

async fn collect_once(
    config: &Upstream,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = config.ws_url("traffic");
    eprintln!("[{}] connecting to {}", config.name, url);
    let (mut stream, _) = tokio_tungstenite::connect_async(&url).await?;

    while let Some(msg) = stream.next().await {
        let msg = msg?;
        if let Message::Text(text) = msg {
            let payload: Payload = match serde_json::from_str(&text) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("[{}] parse error: {}", config.name, e);
                    continue;
                }
            };
            let labels = &[config.name.as_str()];
            metrics::UP_TOTAL.with_label_values(labels).set(payload.up_total as f64);
            metrics::DOWN_TOTAL.with_label_values(labels).set(payload.down_total as f64);
        }
    }

    eprintln!("[{}] disconnected", config.name);
    Err("disconnected".into())
}