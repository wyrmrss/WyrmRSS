import { useEffect, useEffectEvent } from "react";
import type { Virtualizer } from "@tanstack/react-virtual";
import type { PostListRow } from "../utils/posts";

/**
 * Reports which posts have been scrolled past, for the mark-read-on-scroll mode.
 *
 * @param virtualizer The list's virtualizer: knows the scroll element and every row's position.
 * @param rows The rows the virtualizer renders, in the same order.
 * @param onScrolledPast Called on each scroll with every item fully above the visible area,
 *   or with all items once the end is reached with no more pages to load.
 *   Undefined disables tracking.
 */
export function useScrolledPastPosts<T>(
  virtualizer: Virtualizer<HTMLDivElement, Element>,
  rows: PostListRow<T>[],
  onScrolledPast: ((items: T[]) => void) | undefined,
) {
  // Uses the virtualizer's positions, so rows that were never rendered count too.
  const report = useEffectEvent(() => {
    const scrollElement = virtualizer.scrollElement;
    if (!scrollElement || !onScrolledPast) return;

    const { scrollTop, clientHeight, scrollHeight } = scrollElement;
    const hasNextPage = rows.at(-1)?.kind === "load-more";
    const reachedEnd = !hasNextPage && scrollTop + clientHeight >= scrollHeight - 1;
    const firstVisible = virtualizer.measurementsCache.findIndex((row) => row.end > scrollTop);
    const passedRows = reachedEnd || firstVisible === -1 ? rows : rows.slice(0, firstVisible);

    onScrolledPast(passedRows.flatMap((row) => (row.kind === "item" ? [row.item] : [])));
  });

  const enabled = onScrolledPast !== undefined;
  useEffect(() => {
    const scrollElement = virtualizer.scrollElement;
    if (!enabled || !scrollElement) return;
    scrollElement.addEventListener("scroll", report, { passive: true });
    return () => scrollElement.removeEventListener("scroll", report);
  }, [enabled, virtualizer]);
}
