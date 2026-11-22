import { useMemo, useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { TbArrowDown, TbRefresh } from "react-icons/tb";
import { usePullToRefresh } from "../../hooks/usePullToRefresh";
import { useScrollToOpenPost } from "../../hooks/useScrollToOpenPost";
import { buildPostListRows, type PostListRow } from "../../utils/posts";

interface Props<T> {
  items: T[] | undefined;
  isLoading: boolean;
  hasNextPage: boolean | undefined;
  fetchNextPage: () => void;
  isFetchingNextPage: boolean;
  renderItem: (item: T) => React.ReactNode;
  // Date used for group headers; must be the field the list is sorted by.
  groupDate: (item: T) => string;
  emptyMessage: string;
  /** Enables pull-to-refresh. */
  onRefresh?: () => void;
  refreshing?: boolean;
  /** Id of the post open in the reader; the list keeps it in view. */
  openPostId?: number | null;
}

export function VirtualPostList<T extends { id: number }>({
  items,
  isLoading,
  hasNextPage,
  fetchNextPage,
  isFetchingNextPage,
  renderItem,
  groupDate,
  emptyMessage,
  onRefresh,
  refreshing = false,
  openPostId,
}: Props<T>) {
  const rows = useMemo(
    () => buildPostListRows(items ?? [], groupDate, hasNextPage ?? false),
    [items, groupDate, hasNextPage],
  );

  const scrollRef = useRef<HTMLDivElement>(null);
  // eslint-disable-next-line react-hooks/incompatible-library
  const virtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => 41,
    overscan: 10,
  });

  useScrollToOpenPost(virtualizer, rows, openPostId);

  const contentRef = useRef<HTMLDivElement>(null);
  const indicatorRef = useRef<HTMLDivElement>(null);
  usePullToRefresh(scrollRef, contentRef, indicatorRef, onRefresh, refreshing);

  const isEmpty = !isLoading && rows.length === 0;

  function renderRow(row: PostListRow<T>) {
    switch (row.kind) {
      case "header":
        return <GroupHeader label={row.label} count={row.count} />;
      case "item":
        return renderItem(row.item);
      case "load-more":
        return <LoadMoreButton onClick={fetchNextPage} loading={isFetchingNextPage} />;
    }
  }

  return (
    <div className="pane-scroll" ref={scrollRef}>
      {onRefresh && (
        // Revealed as the content below slides down; the hook sets data-state.
        <div className={`pull-refresh${refreshing ? " refreshing" : ""}`} ref={indicatorRef}>
          {refreshing ? <TbRefresh className="spinning" /> : <TbArrowDown />}
        </div>
      )}
      <div ref={contentRef}>
        {isLoading && <div className="pane-empty">Loading…</div>}
        {isEmpty && <div className="pane-empty">{emptyMessage}</div>}
        {!isLoading && rows.length > 0 && (
          <div style={{ height: virtualizer.getTotalSize(), position: "relative" }}>
            {virtualizer.getVirtualItems().map((virtualRow) => {
              const row = rows[virtualRow.index];
              return (
                <div
                  key={virtualRow.key}
                  data-index={virtualRow.index}
                  ref={virtualizer.measureElement}
                  className={row.kind === "item" ? itemRowClass(row) : undefined}
                  style={{ position: "absolute", top: virtualRow.start, left: 0, right: 0 }}
                >
                  {renderRow(row)}
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}

function GroupHeader({ label, count }: { label: string; count: number }) {
  return (
    <div className="posts-group">
      <div className="posts-group-label">
        <span>{label}</span>
        <span className="posts-group-count">{count}</span>
      </div>
    </div>
  );
}

// The first and last posts of a date group get rounded corners.
function itemRowClass(row: { isGroupFirst: boolean; isGroupLast: boolean }): string {
  const classes = ["post-card-item"];
  if (row.isGroupFirst) classes.push("post-card-item--first");
  if (row.isGroupLast) classes.push("post-card-item--last");
  return classes.join(" ");
}

function LoadMoreButton({ onClick, loading }: { onClick: () => void; loading: boolean }) {
  return (
    <div className="load-more">
      <button className="load-more-btn" onClick={() => onClick()} disabled={loading}>
        {loading ? "Loading…" : "Load more"}
      </button>
    </div>
  );
}
