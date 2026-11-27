use crate::{
    filter::CompiledFilters,
    guard::{FeedPollingGuard, PollingFeeds},
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
    newtypes::UserId,
    utils::runtime_settings::RuntimeSettings,
    views,
};
use futures::future::join_all;
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex, RwLock},
    time::Duration,
};
use tokio::{
    sync::{Semaphore, mpsc::Receiver},
    time::{Instant, MissedTickBehavior},
};
use tracing::{debug, error, info, warn};
use wyrm_utils::{
    error::{DatabaseError, WyrmError},
    result::WyrmResult,
};

#[derive(Debug)]
pub enum WorkerCommand {
    /// Trigger an immediate poll for user's feeds.
    /// The caller is notified when polling completes.
    PollUserFeeds {
        reply: tokio::sync::oneshot::Sender<()>,
        user_id: UserId,
    },
    /// Polls one feed now, e.g. right after it's added. Not rate limited.
    /// The caller is notified when polling completes.
    PollFeed {
        reply: tokio::sync::oneshot::Sender<()>,
        feed: Feed,
    },
    /// Reload runtime settings (e.g. HTTP client config) without restarting the
    /// worker. The poll schedule only restarts if `feed_poll_interval_secs` changed.
    Reconfigure,
}

/// Polls due feeds every `feed_poll_interval_secs`, a user's feeds on
/// [`WorkerCommand::PollUserFeeds`] and one feed on [`WorkerCommand::PollFeed`].
/// Each poll runs in its own task on a clone,
/// and all polls share `semaphore` and `polling_feeds`.
/// `polling_feeds` stops two polls, manual or periodic, from fetching the same
/// feed at once.
#[derive(Clone)]
pub struct FeedWorker {
    pub db_pool: DatabasePool,
    pub runtime_settings: Arc<RwLock<RuntimeSettings>>,
    /// Max feeds fetched at once, across all polls.
    semaphore: Arc<Semaphore>,
    /// Feeds currently being polled.
    polling_feeds: PollingFeeds,
}

impl FeedWorker {
    pub fn new(pool: DatabasePool, runtime_settings: Arc<RwLock<RuntimeSettings>>) -> Self {
        // Each in-flight feed holds ~1 pool connection. Reserve headroom so parallel
        // polling can never starve the API handlers of the shared pool.
        let permits = (pool.status().max_size / 2).max(1);
        let semaphore = Arc::new(Semaphore::new(permits));
        Self {
            db_pool: pool,
            runtime_settings,
            semaphore,
            polling_feeds: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Starts the feed polling loop.
    pub async fn run(&mut self, mut rx: Receiver<WorkerCommand>) -> WyrmResult<()> {
        let mut interval = self.poll_interval()?;
        let mut ticker = tokio::time::interval_at(Instant::now() + interval, interval);
        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    let worker = self.clone();
                    tokio::spawn(async move {
                        if let Err(e) = worker.poll_all_feeds().await {
                            error!("Feed worker error: {e}");
                        }
                    });
                }
                Some(cmd) = rx.recv() => {
                    match cmd {
                        WorkerCommand::PollUserFeeds {reply, user_id} => {
                            let worker = self.clone();
                            tokio::spawn(async move {
                                if let Err(e) = worker.poll_user_feeds(user_id).await {
                                    error!("Feed worker error: {e}");
                                }
                                let _ = reply.send(());
                            });
                        }
                        WorkerCommand::PollFeed {reply, feed} => {
                            let worker = self.clone();
                            tokio::spawn(async move {
                                if let Err(e) = worker.poll_feed(feed).await {
                                    error!("Feed worker error: {e}");
                                }
                                let _ = reply.send(());
                            });
                        }
                        // Restarts the schedule only if the poll interval changed. A change to
                        // any other runtime setting, like the HTTP timeout, keeps the next tick.
                        WorkerCommand::Reconfigure => {
                            let new_interval = self.poll_interval()?;
                            if new_interval != interval {
                                interval = new_interval;
                                ticker = tokio::time::interval_at(Instant::now() + interval, interval);
                                ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Polls all of a user's feeds, on a manual refresh or after an OPML import.
    /// Filters out paused and non-pollable feeds (e.g. rsshub feeds when integration is disabled).
    async fn poll_user_feeds(&self, user_id: UserId) -> WyrmResult<()> {
        let feeds = Feed::get_all_for_user(&self.db_pool, user_id).await?;

        let resolver = FeedUrlResolver::from(&*self.runtime_settings.read()?);

        let pollable: Vec<Feed> = feeds
            .into_iter()
            .filter(|f| !f.is_paused && resolver.is_pollable(&f.url))
            .collect();

        self.run_polls(pollable, Some(user_id)).await
    }

    /// Polls one feed, sent by the create-feed handler so the user sees its posts right away.
    /// Skips the paused and pollable checks, since a new feed is always both.
    async fn poll_feed(&self, feed: Feed) -> WyrmResult<()> {
        let user_id = feed.user_id;
        self.run_polls(vec![feed], Some(user_id)).await
    }

    /// Queries the database for feeds due for polling and fetches new posts.
    /// Filters out paused and non-pollable feeds (e.g. rsshub feeds when integration is disabled).
    async fn poll_all_feeds(&self) -> WyrmResult<()> {
        let feeds = Feed::get_all(&self.db_pool).await?;

        let resolver = FeedUrlResolver::from(&*self.runtime_settings.read()?);

        let due_feeds: Vec<Feed> = feeds
            .into_iter()
            .filter(|f| f.is_due() && !f.is_paused && resolver.is_pollable(&f.url))
            .collect();

        info!("Processing {} due feeds", due_feeds.len());

        self.run_polls(due_feeds, None).await
    }

    /// Polls `feeds` concurrently, one task per feed, and returns when all are done.
    ///
    /// Feeds already in [`PollingFeeds`] are skipped, since another poll is still
    /// running them. Each remaining feed gets a [`PollingFeedGuard`] that keeps it
    /// marked until its task ends.
    ///
    /// `user_id` limits the credential lookup to that user when all feeds are theirs,
    /// `None` loads every user's.
    async fn run_polls(&self, feeds: Vec<Feed>, user_id: Option<UserId>) -> WyrmResult<()> {
        // Marked before any query, so an early return drops the guards and unmarks them.
        let feeds: Vec<(Feed, FeedPollingGuard)> = feeds
            .into_iter()
            .filter_map(|feed| {
                FeedPollingGuard::try_start(&self.polling_feeds, feed.id).map(|guard| (feed, guard))
            })
            .collect();

        if feeds.is_empty() {
            return Ok(());
        }

        let resolver = FeedUrlResolver::from(&*self.runtime_settings.read()?);

        let credentials = match user_id {
            Some(user_id) => HostCredential::get_all_for_user(&self.db_pool, user_id).await?,
            None => HostCredential::get_all(&self.db_pool).await?,
        };
        let mut creds_by_user: HashMap<UserId, Vec<HostCredential>> = HashMap::new();
        for credential in credentials {
            creds_by_user
                .entry(credential.user_id)
                .or_default()
                .push(credential);
        }

        // One client per feed owner, holding only that owner's credentials.
        // Built from in-memory runtime settings, as settings may have been updated.
        let http_base = HttpClient::new(&HttpConfig::from(&*self.runtime_settings.read()?))?;
        let mut clients: HashMap<UserId, HttpClient> = HashMap::new();
        for owner in feeds.iter().map(|(f, _)| f.user_id) {
            clients.entry(owner).or_insert_with(|| {
                http_base.with_credentials(creds_by_user.remove(&owner).unwrap_or_default())
            });
        }

        let feed_webhooks = views::webhook::all_by_feed(&self.db_pool).await?;
        let feed_folders = views::folder::all_by_feed(&self.db_pool).await?;

        let tasks: Vec<_> = feeds
            .into_iter()
            .map(|(feed, guard)| {
                let http = clients[&feed.user_id].clone();
                let pool = self.db_pool.clone();
                let webhooks = feed_webhooks.get(&feed.id).cloned().unwrap_or_default();
                let folder = feed_folders.get(&feed.id).map(|f| f.name.clone());
                let semaphore = self.semaphore.clone();
                let resolver = resolver.clone();
                tokio::spawn(async move {
                    let _guard = guard; // unmarks the feed when the task ends
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

    fn poll_interval(&self) -> WyrmResult<Duration> {
        let interval = self.runtime_settings.read()?.feed_poll_interval_secs;
        Ok(Duration::from_secs(interval as u64))
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
