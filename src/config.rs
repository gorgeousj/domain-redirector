use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub listen: String,

    pub mobile_prefix: String,
    pub desktop_prefix: String,

    pub redirect_code: u16,

    pub cookie_name: String,

    pub mobile_cookie_value: String,
    pub desktop_cookie_value: String,
}

impl Config {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;

        let cfg: Config = toml::from_str(&content)?;

        Ok(cfg)
    }
}
