use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Upstream {
    pub name: String,
    pub url: String,
    pub port: u16,
    pub token: String,
    #[serde(default)]
    pub ssl: bool,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub upstream: Vec<Upstream>,
}

impl Upstream {
    pub fn ws_url(&self, endpoint: &str) -> String {
        let scheme = if self.ssl { "wss" } else { "ws" };
        format!(
            "{}://{}:{}/{}?token={}",
            scheme, self.url, self.port, endpoint, self.token
        )
    }
}

impl Config {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}
