import { useCallback, useEffect, useRef } from "react";
import { useMarkRead } from "./usePostMutations";
import type { Post } from "../types/Post";
import type { PostId } from "../types/PostId";

// Posts whose delay ends within this window are sent in one request.
const BATCH_WINDOW_MS = 500;

/**
 * Returns the post list's `onScrolledPast` callback for the "On scroll" read mode.
 * Each unread post you scroll past is marked read after `delaySecs`, unless you
 * scroll back up to it first.
 */
export function useMarkReadOnScroll(delaySecs: number) {
  const { mutate: markRead } = useMarkRead();

  // Scrolled-past posts waiting out the delay, with their timers.
  const delayTimers = useRef(new Map<PostId, number>());
  // Posts whose delay has ended, waiting for the batch to be sent.
  const batch = useRef<PostId[]>([]);
  const batchTimer = useRef<number | null>(null);
  // Posts already in a batch, so they aren't marked twice before the cache shows them read.
  const markedIds = useRef(new Set<PostId>());

  const sendBatch = useCallback(() => {
    const ids = batch.current;
    batch.current = [];
    batchTimer.current = null;
    if (ids.length === 0) return;

    markRead(
      { posts: ids },
      // Let a failed batch be retried on the next scroll.
      { onError: () => ids.forEach((id) => markedIds.current.delete(id)) },
    );
  }, [markRead]);

  const addToBatch = useCallback(
    (postId: PostId) => {
      delayTimers.current.delete(postId);
      markedIds.current.add(postId);
      batch.current.push(postId);
      batchTimer.current ??= window.setTimeout(sendBatch, BATCH_WINDOW_MS);
    },
    [sendBatch],
  );

  const onScrolledPast = useCallback(
    (posts: Post[]) => {
      const unreadIds = new Set(
        posts.filter((post) => !post.is_read && !markedIds.current.has(post.id)).map((post) => post.id),
      );

      // You scrolled back up to these before their delay ended, so they stay unread.
      for (const [postId, timer] of delayTimers.current) {
        if (!unreadIds.has(postId)) {
          clearTimeout(timer);
          delayTimers.current.delete(postId);
        }
      }

      for (const postId of unreadIds) {
        if (!delayTimers.current.has(postId)) {
          const timer = window.setTimeout(() => addToBatch(postId), delaySecs * 1000);
          delayTimers.current.set(postId, timer);
        }
      }
    },
    [delaySecs, addToBatch],
  );

  // On leaving the list: drop posts still waiting out the delay, send the rest.
  useEffect(() => {
    const timers = delayTimers.current;
    return () => {
      timers.forEach(clearTimeout);
      timers.clear();
      if (batchTimer.current !== null) clearTimeout(batchTimer.current);
      sendBatch();
    };
  }, [sendBatch]);

  return onScrolledPast;
}
