use database::newtypes::FeedId;
use std::{
    collections::HashSet,
    sync::{Arc, Mutex, PoisonError},
};

/// IDs of all feeds being polled right now. One set lives on `FeedWorker`, and
/// every worker clone and guard holds an `Arc` pointing at that same set.
pub(crate) type PollingFeeds = Arc<Mutex<HashSet<FeedId>>>;

/// Keeps one feed's ID in [`PollingFeeds`] for as long as the guard exists.
///
/// The worker moves a guard into each feed's poll task. When the task ends, by
/// finishing, returning early or panicking, the guard is dropped and removes the
/// ID, so the feed can be polled again.
pub(crate) struct FeedPollingGuard {
    /// The shared set, not a copy.
    polling_feeds: PollingFeeds,
    /// The entry this guard added and removes. Other feeds' entries stay.
    feed_id: FeedId,
}

impl FeedPollingGuard {
    /// Adds the feed's ID to the set and returns its guard. Returns `None` when the
    /// ID is already in the set, meaning another poll of the feed is still running.
    pub(crate) fn try_start(polling_feeds: &PollingFeeds, feed_id: FeedId) -> Option<Self> {
        let mut ids = polling_feeds.lock().unwrap_or_else(PoisonError::into_inner);
        if !ids.insert(feed_id) {
            return None;
        }
        Some(Self {
            polling_feeds: polling_feeds.clone(),
            feed_id,
        })
    }
}

impl Drop for FeedPollingGuard {
    /// Removes this guard's feed ID from the set. A poisoned lock is still used,
    /// since a single insert or remove can't leave the set half-updated.
    fn drop(&mut self) {
        let mut ids = self
            .polling_feeds
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        ids.remove(&self.feed_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn polling_feeds() -> PollingFeeds {
        Arc::new(Mutex::new(HashSet::new()))
    }

    #[test]
    fn second_start_of_the_same_feed_is_refused() {
        let polling_feeds = polling_feeds();
        let _first = FeedPollingGuard::try_start(&polling_feeds, FeedId(1))
            .expect("first start should succeed");

        assert!(
            FeedPollingGuard::try_start(&polling_feeds, FeedId(1)).is_none(),
            "same feed should be refused while its guard lives"
        );
    }

    #[test]
    fn other_feeds_can_start_while_one_is_polling() {
        let polling_feeds = polling_feeds();
        let _first = FeedPollingGuard::try_start(&polling_feeds, FeedId(1))
            .expect("first start should succeed");

        assert!(
            FeedPollingGuard::try_start(&polling_feeds, FeedId(2)).is_some(),
            "a different feed should start"
        );
    }

    #[test]
    fn dropping_the_guard_lets_the_feed_start_again() {
        let polling_feeds = polling_feeds();
        let first = FeedPollingGuard::try_start(&polling_feeds, FeedId(1))
            .expect("first start should succeed");
        drop(first);

        assert!(
            FeedPollingGuard::try_start(&polling_feeds, FeedId(1)).is_some(),
            "feed should start again after its guard is dropped"
        );
    }

    #[test]
    fn panic_in_the_task_still_removes_the_feed() {
        let polling_feeds = polling_feeds();
        let guard = FeedPollingGuard::try_start(&polling_feeds, FeedId(1))
            .expect("first start should succeed");

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            let _guard = guard;
            panic!("poll task panicked");
        }));
        assert!(result.is_err(), "closure should have panicked");

        assert!(
            polling_feeds.lock().expect("lock").is_empty(),
            "feed should be removed after the panic"
        );
    }
}
