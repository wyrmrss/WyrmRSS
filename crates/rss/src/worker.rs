use crate::{
    filter::CompiledFilters,
    http::{HttpClient, HttpConfig, HttpValidators},
    icon,
    url::FeedUrlResolver,
};
use chrono::Utc;
use database::{
    DatabasePool,
    models::{
        feed::{Feed, FeedUpdateForm},
        host_credential::HostCredential,
        post::{Post, PostInsertForm},
        webhook::Webhook,
    },
    utils::runtime_settings::RuntimeSettings,
    views,
};
use futures::future::join_all;
use std::sync::{Arc, RwLock};
use tokio::sync::{Semaphore, mpsc::Receiver};
use tracing::{debug, error, info, warn};
use wyrm_utils::{
    error::{DatabaseError, WyrmError},
    result::WyrmResult,
};

#[derive(Debug)]
pub enum WorkerCommand {
    /// Trigger an immediate feed poll.
    /// The sender is notified when polling completes.
    PollFeeds(tokio::sync::oneshot::Sender<()>),
    /// Reload runtime settings (e.g. HTTP client config)
    /// without restarting the worker.
    Reconfigure,
}

/// Polls due feeds every `feed_poll_interval_secs`, spawning a concurrent task per feed.
/// Accepts [`WorkerCommand`]s to trigger an immediate poll or apply updated runtime
/// settings without waiting for the current interval to expire.
pub struct FeedWorker {
    pub db_pool: DatabasePool,
    pub runtime_settings: Arc<RwLock<RuntimeSettings>>,
}

impl FeedWorker {
    pub fn new(pool: DatabasePool, runtime_settings: Arc<RwLock<RuntimeSettings>>) -> Self {
        Self {
            db_pool: pool,
            runtime_settings,
        }
    }

    /// Starts the feed polling loop.
    pub async fn run(&mut self, mut rx: Receiver<WorkerCommand>) -> WyrmResult<()> {
        loop {
            let interval = {
                let secs = self.runtime_settings.read()?.feed_poll_interval_secs;
                std::time::Duration::from_secs(secs as u64)
            };
            tokio::select! {
                _ = tokio::time::sleep(interval) => {
                    if let Err(e) = self.poll_feeds().await {
                        error!("Feed worker error: {e}");
                    }
                }
                Some(cmd) = rx.recv() => {
                    match cmd {
                        WorkerCommand::PollFeeds(reply) => {
                            if let Err(e) = self.poll_feeds().await {
                                error!("Feed worker error: {e}");
                            }
                            let _ = reply.send(());
                        }
                        WorkerCommand::Reconfigure => {}  // interrupts the sleep; interval re-read at loop top
                    }
                }
            }
        }
    }

    /// Queries the database for feeds due for polling and fetches new posts.
    async fn poll_feeds(&mut self) -> WyrmResult<()> {
        let feeds = Feed::get_all(&self.db_pool).await?;

        let resolver = FeedUrlResolver::from(&*self.runtime_settings.read()?);

        let due_feeds: Vec<Feed> = feeds
            .into_iter()
            .filter(|f| f.is_due() && !f.is_paused && resolver.is_pollable(&f.url))
            .collect();

        info!("Processing {} due feeds", due_feeds.len());

        // TODO(#372): every user's credentials share one client, matched by host;
        // a feed must only use its owner's credentials.
        let host_credentials = HostCredential::get_all(&self.db_pool).await?;

        // Build the http client from in-memory runtime settings
        // as settings may have been updated.
        let http = HttpClient::new(
            &HttpConfig::from(&*self.runtime_settings.read()?),
            host_credentials,
        )?;

        let feed_webhooks = views::webhook::all_by_feed(&self.db_pool).await?;
        let feed_folders = views::folder::all_by_feed(&self.db_pool).await?;

        // Each in-flight feed holds ~1 pool connection. Reserve headroom so polling
        // can never starve the API handlers of the shared pool.
        let permits = (self.db_pool.status().max_size / 2).max(1);
        let semaphore = Arc::new(Semaphore::new(permits));

        let tasks: Vec<_> = due_feeds
            .into_iter()
            .map(|feed| {
                let http = http.clone();
                let pool = self.db_pool.clone();
                let webhooks = feed_webhooks.get(&feed.id).cloned().unwrap_or_default();
                let folder = feed_folders.get(&feed.id).map(|f| f.name.clone());
                let semaphore = semaphore.clone();
                let resolver = resolver.clone();
                tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    let task = FeedTask {
                        feed,
                        webhooks,
                        folder,
                    };
                    process_feed(&pool, &http, &resolver, &task).await;
                })
            })
            .collect();

        join_all(tasks).await;

        Ok(())
    }
}

/// Per-feed data for one poll task
pub struct FeedTask {
    /// The feed
    pub feed: Feed,
    /// List of webhooks attached to the feed
    webhooks: Vec<Webhook>,
    /// Resolved folder name for webhook payloads; `None` = standalone feed.
    folder: Option<String>,
}

/// Fetch -> Parse -> Store -> Notify -> Update
async fn process_feed(
    pool: &DatabasePool,
    http: &HttpClient,
    resolver: &FeedUrlResolver,
    task: &FeedTask,
) {
    info!("Processing feed {}", task.feed.title);

    let result = fetch_and_store(pool, http, resolver, task).await;

    if let Err(e) = &result {
        error!("Failed to process feed {}: {e}", task.feed.url);
    }

    let mut form = FeedUpdateForm::from_poll_result(&task.feed, &result);

    if let Ok(Some(validators)) = &result {
        form.etag = Some(validators.etag.clone());
        form.last_modified = Some(validators.last_modified.clone());
    }

    let url = &task.feed.url;

    match Feed::update(pool, form).await {
        Ok(_) => {}
        // Feed edited or deleted mid-poll.
        Err(WyrmError::Database(DatabaseError::NotFound)) => {
            debug!("Discarded stale poll for {url}");
        }
        Err(e) => error!("Failed to save poll for {url}: {e}"),
    }
}

async fn fetch_and_store(
    pool: &DatabasePool,
    http: &HttpClient,
    resolver: &FeedUrlResolver,
    task: &FeedTask,
) -> WyrmResult<Option<HttpValidators>> {
    let fetch_target = resolver.resolve(&task.feed.url)?;

    let current = Feed::get(pool, task.feed.user_id, task.feed.id).await?;

    let needs_icon = icon::needs_feed_icon(pool, &task.feed).await;

    let validators = if needs_icon {
        HttpValidators::default() // Fetch the body so the icon can be resolved.
    } else {
        HttpValidators::from(&current)
    };

    let Some(fetched) = http.fetch_conditional(&fetch_target, &validators).await? else {
        return Ok(None); // 304: the stored validators stay as they are.
    };

    let parsed = feed_rs::parser::parse(&fetched.body[..])?;

    // Best-effort: resolves the icon for feeds that never got one (OPML
    // imports, pre-icon feeds) and retries week-old misses.
    if needs_icon {
        icon::ensure_feed_icon(pool, http, resolver, &task.feed, &parsed).await;
    }

    let filters = CompiledFilters::new(&task.feed.filters);

    let mut forms: Vec<PostInsertForm> = parsed
        .entries
        .into_iter()
        .filter(|entry| !filters.excludes(entry))
        .map(|entry| PostInsertForm::from_entry(entry, task.feed.id))
        .collect();

    // Insert oldest-first so serial ids ascend with publish date: post lists
    // sort by (created_at, id) and a whole batch shares one created_at, so
    // insertion order is the display order within the batch. Entries without
    // a publish date get NOW() from the column default, hence sort as newest.
    let now = Utc::now();
    forms.sort_by_key(|f| f.published_at.unwrap_or(now));

    // A feed's first fetch is backfill, not news: stamp created_at with the
    // publish date so the history interleaves into the river chronologically
    // instead of clumping on top as one just-arrived batch.
    if task.feed.last_fetched_at.is_none() {
        for form in &mut forms {
            form.created_at = form.published_at;
        }
    }

    let new_posts = Post::create_many(pool, forms).await?;

    info!(
        "Got {} new posts for feed {}",
        new_posts.len(),
        task.feed.title
    );

    if new_posts.is_empty() {
        return Ok(Some(fetched.validators));
    }

    for webhook in &task.webhooks {
        // A broken custom template must not abort the poll or send garbage:
        // log and skip this delivery.
        match wyrm_webhook::get_payload(webhook, &task.feed, task.folder.as_deref(), &new_posts) {
            Ok(payload) => {
                let http = http.clone();
                let webhook = webhook.clone();
                tokio::spawn(async move {
                    if let Err(e) = http.post_json(&webhook.url, &payload).await {
                        warn!("webhook {} delivery error: {e}", webhook.name);
                    }
                });
            }
            Err(e) => error!("skipping webhook for feed {}: {e}", task.feed.title),
        }
    }

    Ok(Some(fetched.validators))
}
