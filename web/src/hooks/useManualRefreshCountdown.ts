import { useEffect, useState } from "react";
import { useAccount } from "./useAccount";
import { useLocalInstance } from "./useLocalInstance";

/** Seconds until the user can refresh their feeds again, 0 when they can now. */
export function useManualRefreshCountdown(): number {
  const { data: account } = useAccount();
  const { data: instance } = useLocalInstance();
  const [now, setNow] = useState(() => Date.now());

  const lastRefreshAt = account?.last_manual_poll_at;
  const nextRefreshAt =
    lastRefreshAt && instance
      ? new Date(lastRefreshAt).getTime() + instance.manual_poll_interval_secs * 1000
      : 0;

  useEffect(() => {
    if (nextRefreshAt === 0) return;
    // First tick right away, as `now` may be from before the last refresh.
    const tick = () => {
      setNow(Date.now());
      if (Date.now() >= nextRefreshAt) clearInterval(interval);
    };
    const first = setTimeout(tick, 0);
    const interval = setInterval(tick, 1000);
    return () => {
      clearTimeout(first);
      clearInterval(interval);
    };
  }, [nextRefreshAt]);

  return Math.max(0, Math.ceil((nextRefreshAt - now) / 1000));
}

/** Formats seconds as m:ss, e.g. 42 as "0:42". */
export function formatCountdown(secs: number): string {
  const mins = Math.floor(secs / 60);
  return `${mins}:${String(secs % 60).padStart(2, "0")}`;
}
