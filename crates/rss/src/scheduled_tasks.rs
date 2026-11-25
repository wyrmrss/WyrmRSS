use database::{
    DatabasePool,
    models::{post::Post, session::Session, settings::Settings},
};
use std::time::Duration;
use tokio::time::{MissedTickBehavior, interval};
use tracing::{error, info};

const HOURLY: Duration = Duration::from_secs(60 * 60);

pub fn spawn(pool: DatabasePool) {
    spawn_every(HOURLY, pool.clone(), expire_posts);
    spawn_every(HOURLY, pool.clone(), expire_sessions);
}

/// Runs `job` every `period` in its own task, first right away at startup.
fn spawn_every<F, Fut>(period: Duration, pool: DatabasePool, job: F)
where
    F: Fn(DatabasePool) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    tokio::spawn(async move {
        let mut ticker = interval(period);
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            ticker.tick().await;
            job(pool.clone()).await;
        }
    });
}

/// Deletes each user's read and unread posts older than their thresholds.
async fn expire_posts(pool: DatabasePool) {
    let all_settings = match Settings::get_all(&pool).await {
        Ok(all_settings) => all_settings,
        Err(e) => return error!("failed to load settings for expiry: {e}"),
    };

    for settings in all_settings {
        if let Some(days) = settings.expire_read_after_days {
            match Post::expire_read(&pool, settings.user_id, days).await {
                Ok(0) => {}
                Ok(n) => info!("expired {n} read posts for user {}", settings.user_id),
                Err(e) => error!(
                    "failed to expire read posts for user {}: {e}",
                    settings.user_id
                ),
            }
        }
        if let Some(days) = settings.expire_unread_after_days {
            match Post::expire_unread(&pool, settings.user_id, days).await {
                Ok(0) => {}
                Ok(n) => info!("expired {n} unread posts for user {}", settings.user_id),
                Err(e) => error!(
                    "failed to expire unread posts for user {}: {e}",
                    settings.user_id
                ),
            }
        }
    }
}

/// Deletes expired login sessions.
async fn expire_sessions(pool: DatabasePool) {
    match Session::delete_expired(&pool).await {
        Ok(0) => {}
        Ok(n) => info!("expired {n} sessions"),
        Err(e) => error!("failed to delete expired sessions: {e}"),
    }
}
