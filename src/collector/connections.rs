use crate::config::Upstream;
use crate::metrics;
use futures_util::StreamExt;
use serde::Deserialize;
use std::time::Duration;
use tokio_tungstenite::tungstenite::Message;

#[derive(Deserialize)]
struct Metadata {
    #[serde(default)]
    network: String,
    #[serde(default)]
    r#type: String,
    #[serde(default)]
    #[serde(rename = "sourceIP")]
    source_ip: String,
    #[serde(default)]
    #[serde(rename = "destinationIP")]
    destination_ip: String,
    #[serde(default)]
    #[serde(rename = "sourceGeoIP")]
    source_geo_ip: Option<String>,
    #[serde(default)]
    #[serde(rename = "destinationGeoIP")]
    destination_geo_ip: Option<String>,
    #[serde(default)]
    #[serde(rename = "sourceIPASN")]
    source_ip_asn: String,
    #[serde(default)]
    #[serde(rename = "destinationIPASN")]
    destination_ip_asn: String,
    #[serde(default)]
    #[serde(rename = "sourcePort")]
    source_port: String,
    #[serde(default)]
    #[serde(rename = "destinationPort")]
    destination_port: String,
    #[serde(default)]
    #[serde(rename = "inboundIP")]
    inbound_ip: String,
    #[serde(default)]
    #[serde(rename = "inboundPort")]
    inbound_port: String,
    #[serde(default)]
    #[serde(rename = "inboundName")]
    inbound_name: String,
    #[serde(default)]
    #[serde(rename = "inboundUser")]
    inbound_user: String,
    #[serde(default)]
    host: String,
    #[serde(default)]
    #[serde(rename = "dnsMode")]
    dns_mode: String,
    #[serde(default)]
    uid: u32,
    #[serde(default)]
    process: String,
    #[serde(default)]
    #[serde(rename = "processPath")]
    process_path: String,
    #[serde(default)]
    #[serde(rename = "specialProxy")]
    special_proxy: String,
    #[serde(default)]
    #[serde(rename = "specialRules")]
    special_rules: String,
    #[serde(default)]
    #[serde(rename = "remoteDestination")]
    remote_destination: String,
    #[serde(default)]
    dscp: u32,
    #[serde(default)]
    #[serde(rename = "sniffHost")]
    sniff_host: String,
}

#[derive(Deserialize)]
struct Connection {
    id: String,
    metadata: Metadata,
    upload: u64,
    download: u64,
    #[serde(default)]
    chains: Vec<String>,
    #[serde(default)]
    #[serde(rename = "providerChains")]
    provider_chains: Vec<String>,
    #[serde(default)]
    rule: String,
    #[serde(default)]
    #[serde(rename = "rulePayload")]
    rule_payload: String,
}

#[derive(Deserialize)]
struct Payload {
    #[serde(rename = "downloadTotal")]
    _download_total: u64,
    #[serde(rename = "uploadTotal")]
    _upload_total: u64,
    #[serde(default)]
    _memory: u64,
    #[serde(default)]
    connections: Vec<Connection>,
}

pub async fn run(config: Upstream) {
    loop {
        if let Err(e) = collect_once(&config).await {
            eprintln!(
                "[{}] connections error: {}, retrying in 5s...",
                config.name, e
            );
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }
}

async fn collect_once(config: &Upstream) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = config.ws_url("connections");
    eprintln!(
        "[{}] connecting to connections endpoint: {}",
        config.name, url
    );
    let (mut stream, _) = tokio_tungstenite::connect_async(&url).await?;

    while let Some(msg) = stream.next().await {
        let msg = msg?;
        if let Message::Text(text) = msg {
            let payload: Payload = match serde_json::from_str(&text) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("[{}] connections parse error: {}", config.name, e);
                    continue;
                }
            };

            for conn in payload.connections {
                // Build label values
                let upstream_name = config.name.as_str();
                let network = &conn.metadata.network;
                let conn_type = &conn.metadata.r#type;
                let source_ip = &conn.metadata.source_ip;
                let destination_ip = &conn.metadata.destination_ip;
                let source_port = &conn.metadata.source_port;
                let destination_port = &conn.metadata.destination_port;
                let inbound_ip = &conn.metadata.inbound_ip;
                let inbound_port = &conn.metadata.inbound_port;
                let inbound_name = &conn.metadata.inbound_name;
                let inbound_user = &conn.metadata.inbound_user;
                let host = &conn.metadata.host;
                let dns_mode = &conn.metadata.dns_mode;
                let uid_str = conn.metadata.uid.to_string();
                let process = &conn.metadata.process;
                let process_path = &conn.metadata.process_path;
                let special_proxy = &conn.metadata.special_proxy;
                let special_rules = &conn.metadata.special_rules;
                let remote_destination = &conn.metadata.remote_destination;
                let dscp_str = conn.metadata.dscp.to_string();
                let sniff_host = &conn.metadata.sniff_host;
                let rule = &conn.rule;
                let rule_payload = &conn.rule_payload;
                let conn_id = &conn.id;
                let source_geo_ip = conn.metadata.source_geo_ip.as_deref().unwrap_or("");
                let destination_geo_ip = conn.metadata.destination_geo_ip.as_deref().unwrap_or("");
                let source_ip_asn = &conn.metadata.source_ip_asn;
                let destination_ip_asn = &conn.metadata.destination_ip_asn;

                for chain in &conn.chains {
                    if chain.is_empty() {
                        continue;
                    }
                    for provider_chain in &conn.provider_chains {
                        if provider_chain.is_empty() {
                            continue;
                        }
                        // Create labels array (must match the order in metrics.rs)
                        let labels = &[
                            upstream_name,
                            conn_id,
                            network,
                            conn_type,
                            source_ip,
                            destination_ip,
                            source_port,
                            destination_port,
                            source_geo_ip,
                            destination_geo_ip,
                            source_ip_asn,
                            destination_ip_asn,
                            inbound_ip,
                            inbound_port,
                            inbound_name,
                            inbound_user,
                            host,
                            dns_mode,
                            &uid_str,
                            process,
                            process_path,
                            special_proxy,
                            special_rules,
                            remote_destination,
                            &dscp_str,
                            sniff_host,
                            rule,
                            rule_payload,
                            chain,
                            provider_chain,
                        ];

                        // Record upload bytes
                        metrics::CONNECTION_UPLOAD
                            .with_label_values(labels)
                            .set(conn.upload as f64);

                        // Record download bytes
                        metrics::CONNECTION_DOWNLOAD
                            .with_label_values(labels)
                            .set(conn.download as f64);
                    }
                }
            }
        }
    }

    eprintln!("[{}] connections disconnected", config.name);
    Err("disconnected".into())
}
