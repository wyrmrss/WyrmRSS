use database::{
    DatabasePool,
    models::host_credential::HostCredential,
    newtypes::UserId,
    utils::runtime_settings::RuntimeSettings,
};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc::Sender;
use wyrm_rss::{
    http::{HttpClient, HttpConfig},
    url::FeedUrlResolver,
    worker::WorkerCommand,
};
use wyrm_utils::result::WyrmResult;

#[derive(Clone)]
pub struct WyrmContext {
    /// Async database connection pool.
    pub db_pool: DatabasePool,
    /// Runtime settings loaded from the database. Shared between RSS worker
    /// and actix handlers threads so that setting updates take effect
    /// immediately without a restart.
    pub runtime_settings: Arc<RwLock<RuntimeSettings>>,
    /// Channel to send commands to the background RSS worker.
    pub worker_tx: Sender<WorkerCommand>,
    /// Whether or not this server is running in demo mode.
    /// Demo mode means certain APIs are restricted.
    /// Used for https://demo.wyrmrss.com
    pub run_as_demo: bool,
}

impl WyrmContext {
    /// Builds an HTTP client from the runtime settings and the user's
    /// host credentials, for feed discovery and icon fetches.
    pub async fn http(&self, user_id: UserId) -> WyrmResult<HttpClient> {
        let user_creds = HostCredential::get_all_for_user(&self.db_pool, user_id).await?;
        let http = HttpClient::new(&HttpConfig::from(&*self.runtime_settings.read()?))?;

        Ok(http.with_credentials(user_creds))
    }

    /// Builds a feed URL resolver from the runtime settings.
    pub fn url_resolver(&self) -> WyrmResult<FeedUrlResolver> {
        Ok(FeedUrlResolver::from(&*self.runtime_settings.read()?))
    }

    /// Deny request if server is running in demo mode.
    pub fn deny_in_demo(&self) -> WyrmResult<()> {
        if self.run_as_demo {
            return Err(wyrm_utils::error::WyrmError::DemoRestricted);
        }
        Ok(())
    }
}
