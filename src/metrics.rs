use prometheus::{GaugeVec, register_gauge_vec};
use std::sync::LazyLock;

pub static UP_TOTAL: LazyLock<GaugeVec> = LazyLock::new(|| {
    register_gauge_vec!("proxy_up_total_bytes", "Total uploaded bytes", &["name"]).unwrap()
});

pub static DOWN_TOTAL: LazyLock<GaugeVec> = LazyLock::new(|| {
    register_gauge_vec!(
        "proxy_down_total_bytes",
        "Total downloaded bytes",
        &["name"]
    )
    .unwrap()
});

pub static CONNECTION_UPLOAD: LazyLock<GaugeVec> = LazyLock::new(|| {
    register_gauge_vec!(
        "proxy_connection_upload_bytes",
        "Upload bytes per connection",
        &[
            "name",
            "connection_id",
            "network",
            "type",
            "source_ip",
            "destination_ip",
            "source_port",
            "destination_port",
            "source_geo_ip",
            "destination_geo_ip",
            "source_ip_asn",
            "destination_ip_asn",
            "inbound_ip",
            "inbound_port",
            "inbound_name",
            "inbound_user",
            "host",
            "dns_mode",
            "uid",
            "process",
            "process_path",
            "special_proxy",
            "special_rules",
            "remote_destination",
            "dscp",
            "sniff_host",
            "rule",
            "rule_payload",
            "chain",
            "provider_chain",
        ]
    )
    .unwrap()
});

pub static CONNECTION_DOWNLOAD: LazyLock<GaugeVec> = LazyLock::new(|| {
    register_gauge_vec!(
        "proxy_connection_download_bytes",
        "Download bytes per connection",
        &[
            "name",
            "connection_id",
            "network",
            "type",
            "source_ip",
            "destination_ip",
            "source_port",
            "destination_port",
            "source_geo_ip",
            "destination_geo_ip",
            "source_ip_asn",
            "destination_ip_asn",
            "inbound_ip",
            "inbound_port",
            "inbound_name",
            "inbound_user",
            "host",
            "dns_mode",
            "uid",
            "process",
            "process_path",
            "special_proxy",
            "special_rules",
            "remote_destination",
            "dscp",
            "sniff_host",
            "rule",
            "rule_payload",
            "chain",
            "provider_chain",
        ]
    )
    .unwrap()
});
