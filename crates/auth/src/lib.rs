//! Authentication: password and OIDC sign-in, sessions and CSRF checks, the
//! desktop key, the setup token, and the middleware for protected routes.
pub mod authenticated_user;
mod desktop_key;
#[cfg(feature = "oidc")]
mod http;
pub mod middleware;
#[cfg(feature = "oidc")]
pub mod oidc;
pub mod password;
pub mod session;
pub mod username;

#[cfg(feature = "oidc")]
use crate::oidc::Oidc;
use crate::session::SessionManager;
use actix_web::HttpRequest;
use database::DatabasePool;
use serde::Serialize;
use sha2::{Digest, Sha256};
use wyrm_utils::{config::WyrmStartupConfig, error::AuthError, result::WyrmResult};

/// Sign-in methods enabled on this instance, as reported to the frontend.
#[derive(Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case", export)]
pub enum AuthMethod {
    Oidc,
    Password,
}

pub struct Auth {
    desktop_key: Option<String>,
    #[cfg(feature = "oidc")]
    oidc: Option<Oidc>,
    sessions: SessionManager,
    setup_token: Option<String>,
}

impl Auth {
    pub fn new(config: &WyrmStartupConfig) -> WyrmResult<Self> {
        let sessions = SessionManager::from_public_url(&config.public_url()?);

        Ok(Self {
            desktop_key: config.desktop_key.clone(),
            #[cfg(feature = "oidc")]
            oidc: Oidc::new(config)?,
            sessions,
            setup_token: None,
        })
    }

    pub fn methods(&self) -> Vec<AuthMethod> {
        let mut methods = Vec::new();
        if self.oidc_enabled() {
            methods.push(AuthMethod::Oidc);
        }
        methods.push(AuthMethod::Password);
        methods
    }

    /// True when OIDC is compiled in and configured.
    fn oidc_enabled(&self) -> bool {
        #[cfg(feature = "oidc")]
        let enabled = self.oidc.is_some();
        #[cfg(not(feature = "oidc"))]
        let enabled = false;
        enabled
    }

    /// The OIDC method, or `AuthMethodDisabled` (404) when it isn't configured.
    #[cfg(feature = "oidc")]
    pub fn oidc(&self) -> WyrmResult<&Oidc> {
        self.oidc
            .as_ref()
            .ok_or_else(|| AuthError::AuthMethodDisabled("oidc".into()).into())
    }

    pub fn sessions(&self) -> &SessionManager {
        &self.sessions
    }

    pub fn setup_token(&self) -> Option<&str> {
        self.setup_token.as_deref()
    }

    /// Generates a one-time setup token.
    pub fn with_setup_token(self) -> Self {
        let mut bytes = [0u8; 8];
        getrandom::fill(&mut bytes).expect("OS random source unavailable");
        let token = hex::encode(bytes);

        Self {
            setup_token: Some(token),
            ..self
        }
    }

    /// True if `provided` is this instance's setup token.
    pub fn setup_token_matches(&self, provided: &str) -> bool {
        self.setup_token
            .as_deref()
            .is_some_and(|token| Sha256::digest(provided.trim()) == Sha256::digest(token))
    }

    /// True when the request carries the desktop app's key.
    pub fn desktop_key_valid(&self, req: &HttpRequest) -> bool {
        desktop_key::valid(req, self)
    }

    /// True when the request carries the desktop key or an active session.
    /// For public routes, which the auth middleware doesn't run on.
    pub async fn is_signed_in(&self, pool: &DatabasePool, req: &HttpRequest) -> WyrmResult<bool> {
        if self.desktop_key_valid(req) {
            return Ok(true);
        }
        Ok(self.sessions.authenticate(pool, req).await?.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_public_url_is_an_error() {
        assert!(Auth::new(&WyrmStartupConfig::default()).is_err());
    }

    #[cfg(feature = "oidc")]
    fn oidc_config() -> WyrmStartupConfig {
        WyrmStartupConfig {
            public_url: "https://rss.example.com".into(),
            oidc_issuer_url: Some("https://sso.example.com/realms/wyrm".into()),
            oidc_client_id: Some("wyrm".into()),
            ..Default::default()
        }
    }

    #[cfg(feature = "oidc")]
    #[test]
    fn oidc_is_listed_with_password() {
        let auth = Auth::new(&oidc_config()).expect("valid config");
        assert!(matches!(
            auth.methods().as_slice(),
            [AuthMethod::Oidc, AuthMethod::Password]
        ));
        assert!(auth.oidc().is_ok());
    }
}
