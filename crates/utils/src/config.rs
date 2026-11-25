use crate::{error::WyrmError, result::WyrmResult};
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use smart_default::SmartDefault;
use std::net::{IpAddr, Ipv4Addr};
use url::Url;

#[derive(Debug, Clone, Deserialize, SmartDefault)]
#[serde(default)]
/// Built from wyrm.toml or env vars
pub struct WyrmStartupConfig {
    /// Database connection URI
    #[default("postgres://wyrm:wyrm@localhost/wyrm")]
    pub database_connection: String,
    /// Max number of active database connections
    #[default(30)]
    pub database_pool_size: usize,
    /// Bind to IP addr
    #[default(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)))]
    pub bind: IpAddr,
    /// Port to listen on
    #[default(3001)]
    pub port: u16,
    /// Externally reachable URL of this instance, e.g. `https://rss.example.com`.
    pub public_url: String,
    /// The key set by the desktop build. Used to bypass authentication entirely.
    #[serde(skip)]
    pub desktop_key: Option<String>,
    /// OIDC provider's issuer URL, e.g. `https://sso.example.com/realms/wyrm`.
    pub oidc_issuer_url: Option<String>,
    /// Client ID registered for Wyrm at the OIDC provider.
    pub oidc_client_id: Option<String>,
    /// Client secret, for confidential clients. Omit for public clients.
    pub oidc_client_secret: Option<String>,
    /// Comma-separated subject allowlist. Omit to allow any user
    /// authenticated by the configured OIDC provider.
    pub oidc_allowed_subjects: Option<String>,
    /// Whether or not this server is running in demo mode.
    /// Demo mode means certain APIs are restricted.
    /// Used for https://demo.wyrmrss.com
    pub run_as_demo: bool,
}

impl WyrmStartupConfig {
    pub fn load() -> WyrmResult<Self> {
        let config: Self = Config::builder()
            .add_source(File::with_name("config/wyrm.toml").required(false))
            .add_source(Environment::with_prefix("WYRM"))
            .build()?
            .try_deserialize()?;
        // Fail fast on a missing or malformed public_url.
        config.public_url()?;
        Ok(config)
    }

    /// `public_url` parsed and validated: an http(s) origin with nothing after host and port.
    pub fn public_url(&self) -> WyrmResult<Url> {
        if self.public_url.trim().is_empty() {
            return Err(WyrmError::StartupConfigError(ConfigError::Message(
                "public_url is required".into(),
            )));
        }

        let invalid = |reason: &str| {
            WyrmError::StartupConfigError(ConfigError::Message(format!("public_url {reason}")))
        };

        let url = Url::parse(self.public_url.trim()).map_err(|_| invalid("is not a valid URL"))?;

        if !matches!(url.scheme(), "http" | "https") {
            return Err(invalid("must use http or https"));
        }
        if url.path() != "/" || url.query().is_some() || url.fragment().is_some() {
            return Err(invalid("must not have a path, query or fragment"));
        }
        if !url.username().is_empty() || url.password().is_some() {
            return Err(invalid("must not contain credentials"));
        }
        Ok(url)
    }
}
