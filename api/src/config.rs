use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenv::dotenv().ok();
        Ok(Self {
            port: env::var("API_PORT").unwrap_or("3000".into()).parse()?,
        })
    }
}
