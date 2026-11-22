import { useEffect, useEffectEvent } from "react";
import type { Virtualizer } from "@tanstack/react-virtual";
import type { PostListRow } from "../utils/posts";

/**
 * Keeps the post you're reading visible in the list. When you open another post
 * (j/k or a link) and its entry in the list is out of view, scrolls the list to it.
 *
 * @param virtualizer The list's virtualizer, which does the scrolling.
 * @param rows The rows the virtualizer renders, in the same order.
 * @param openPostId Id of the post open in the reader, or null/undefined if none.
 */
export function useScrollToOpenPost<T extends { id: number }>(
  virtualizer: Virtualizer<HTMLDivElement, Element>,
  rows: PostListRow<T>[],
  openPostId: number | null | undefined,
) {
  // Finds the post's row and scrolls to it. "auto" doesn't scroll if the row is already visible.
  const scrollToPost = useEffectEvent((postId: number) => {
    const rowIndex = rows.findIndex((row) => row.kind === "item" && row.item.id === postId);
    if (rowIndex !== -1) virtualizer.scrollToIndex(rowIndex, { align: "auto" });
  });

  // Runs only when you open another post, not when the list refreshes.
  useEffect(() => {
    if (openPostId != null) scrollToPost(openPostId);
  }, [openPostId]);
}
