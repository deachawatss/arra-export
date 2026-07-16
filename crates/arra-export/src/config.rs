use std::{env, path::PathBuf};

use thiserror::Error;
use url::Url;

const DEFAULT_ORACLE_URL: &str = "http://localhost:47778";
const DEFAULT_PORT: u16 = 4778;
const DEFAULT_FRONTEND_DIST: &str = "frontend/build";

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub oracle_url: Url,
    pub port: u16,
    pub frontend_dist: PathBuf,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error(
        "ORACLE_URL must be an absolute http or https URL without credentials, query parameters, or fragments"
    )]
    InvalidOracleUrl,
    #[error("ARRA_EXPORT_PORT must be an integer from 1 through 65535")]
    InvalidPort,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let oracle_url = env::var("ORACLE_URL").unwrap_or_else(|_| DEFAULT_ORACLE_URL.to_owned());
        let port = match env::var("ARRA_EXPORT_PORT") {
            Ok(value) => value.parse::<u16>().map_err(|_| ConfigError::InvalidPort)?,
            Err(_) => DEFAULT_PORT,
        };

        if port == 0 {
            return Err(ConfigError::InvalidPort);
        }

        let frontend_dist = env::var("ARRA_EXPORT_FRONTEND_DIST")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(DEFAULT_FRONTEND_DIST));

        Ok(Self {
            oracle_url: parse_oracle_url(&oracle_url)?,
            port,
            frontend_dist,
        })
    }
}

pub fn parse_oracle_url(value: &str) -> Result<Url, ConfigError> {
    let mut url = Url::parse(value).map_err(|_| ConfigError::InvalidOracleUrl)?;
    let valid_scheme = matches!(url.scheme(), "http" | "https");
    let valid_host = url.host().is_some();
    let path_is_root = url.path().is_empty() || url.path() == "/";

    if !valid_scheme
        || !valid_host
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || !path_is_root
    {
        return Err(ConfigError::InvalidOracleUrl);
    }

    url.set_path("/");
    Ok(url)
}
