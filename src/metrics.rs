use prometheus::{register_gauge_vec, GaugeVec};
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
