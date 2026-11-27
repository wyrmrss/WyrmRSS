use crate::models::instance::LocalInstance;

pub struct RuntimeSettings {
    pub feed_poll_interval_secs: i32,
    pub http_timeout_secs: i32,
    pub http_connect_timeout_secs: i32,
    pub http_retries: i32,
    pub http_user_agent: Option<String>,
    pub manual_poll_interval_secs: i32,
    pub rsshub_instance: Option<String>,
}

impl From<&LocalInstance> for RuntimeSettings {
    fn from(s: &LocalInstance) -> Self {
        RuntimeSettings {
            feed_poll_interval_secs: s.feed_poll_interval_secs,
            http_timeout_secs: s.http_timeout_secs,
            http_connect_timeout_secs: s.http_connect_timeout_secs,
            http_retries: s.http_retries,
            http_user_agent: s.http_user_agent.clone(),
            manual_poll_interval_secs: s.manual_poll_interval_secs,
            rsshub_instance: s.rsshub_instance.clone(),
        }
    }
}
