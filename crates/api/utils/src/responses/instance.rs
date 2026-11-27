use database::models::instance::{LocalInstance, RegistrationMode};
use serde::Serialize;

/// Instance-wide settings. Readable without signing in, admins change them.
#[derive(Serialize, ts_rs::TS)]
#[ts(export)]
pub struct LocalInstanceResponse {
    /// Whether the instance has been set up yet. Always true on desktop.
    pub is_setup: bool,
    /// Who can create an account.
    pub registration_mode: RegistrationMode,
    /// How often feeds are polled for new posts, in seconds.
    pub feed_poll_interval_secs: i32,
    /// Minimum time between a user's manual refreshes, in seconds.
    pub manual_poll_interval_secs: i32,
    /// Total timeout for a feed HTTP request, in seconds.
    pub http_timeout_secs: i32,
    /// Timeout for connecting to a feed host, in seconds.
    pub http_connect_timeout_secs: i32,
    /// Number of times a failed feed request is retried.
    pub http_retries: i32,
    /// Custom `User-Agent` header sent when fetching feeds, `null` uses the default.
    pub http_user_agent: Option<String>,
    /// RSSHub instance base URL, `null` disables rsshub:// feeds.
    pub rsshub_instance: Option<String>,
}

impl LocalInstanceResponse {
    pub fn new(instance: LocalInstance, signed_in: bool) -> Self {
        Self {
            is_setup: instance.is_setup,
            registration_mode: instance.registration_mode,
            feed_poll_interval_secs: instance.feed_poll_interval_secs,
            manual_poll_interval_secs: instance.manual_poll_interval_secs,
            http_timeout_secs: instance.http_timeout_secs,
            http_connect_timeout_secs: instance.http_connect_timeout_secs,
            http_retries: instance.http_retries,
            http_user_agent: instance.http_user_agent,
            rsshub_instance: instance.rsshub_instance.filter(|_| signed_in),
        }
    }
}
