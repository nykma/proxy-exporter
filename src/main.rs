mod collector;
mod config;
mod metrics;

use std::net::SocketAddr;

use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use prometheus::{Encoder, TextEncoder};
use tokio::net::TcpListener;

const DEFAULT_LISTEN_ADDR: &str = "0.0.0.0:9898";
const DEFAULT_CONFIG_PATH: &str = "config.toml";

type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

async fn handle(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody>, hyper::Error> {
    match req.uri().path() {
        "/metrics" => {
            let encoder = TextEncoder::new();
            let metric_families = prometheus::gather();
            let mut buffer = Vec::new();
            encoder.encode(&metric_families, &mut buffer).unwrap();
            Ok(Response::builder()
                .header("Content-Type", encoder.format_type())
                .body(full(buffer))
                .unwrap())
        }
        "/" => Ok(Response::new(full(
            "<html><body><a href='/metrics'>Metrics</a></body></html>",
        ))),
        _ => Ok(Response::builder()
            .status(404)
            .body(full("not found"))
            .unwrap()),
    }
}

#[tokio::main]
async fn main() {
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| DEFAULT_CONFIG_PATH.to_string());
    let config = config::Config::load(&config_path).unwrap_or_else(|e| {
        eprintln!("failed to load config from {}: {}", config_path, e);
        std::process::exit(1);
    });

    for upstream in &config.upstream {
        let labels = &[upstream.name.as_str()];
        metrics::UP_TOTAL.with_label_values(labels).set(0.0);
        metrics::DOWN_TOTAL.with_label_values(labels).set(0.0);
    }

    for upstream in config.upstream {
        eprintln!(
            "[{}] configured: {}://{}:{}",
            upstream.name,
            if upstream.ssl { "wss" } else { "ws" },
            upstream.url,
            upstream.port
        );
        tokio::spawn(collector::traffic::run(upstream));
    }

    let listen_addr: SocketAddr = std::env::var("LISTEN_ADDRESS")
        .unwrap_or_else(|_| DEFAULT_LISTEN_ADDR.to_string())
        .parse()
        .expect("invalid LISTEN_ADDRESS");

    let listener = TcpListener::bind(listen_addr)
        .await
        .unwrap_or_else(|e| {
            eprintln!("failed to bind {}: {}", listen_addr, e);
            std::process::exit(1);
        });

    eprintln!("listening on {}", listen_addr);

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let stream = TokioIo::new(stream);
        let svc = hyper::service::service_fn(handle);
        tokio::spawn(async move {
            if let Err(e) = http1::Builder::new().serve_connection(stream, svc).await {
                eprintln!("connection error: {}", e);
            }
        });
    }
}