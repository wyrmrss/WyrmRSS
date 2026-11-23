import { ENDPOINTS } from "./api";
import { AUTH } from "./constants";
import type { AuthStatusResponse } from "../types/AuthStatusResponse";

// The desktop app's per-launch key. Kept in memory only, since a new one is made every launch.
let desktopKey: string | null = null;

export function setDesktopKey(key: string): void {
  desktopKey = key;
}

// A reverse proxy answers with these when the backend behind it is down.
// Not 504: the server itself sends that when a manual poll outlasts the HTTP timeout.
const GATEWAY_ERRORS = new Set([502, 503]);

// Raises "wyrm:unreachable" when the server can't be reached and
// "wyrm:reachable" once it answers again (see ServerStatusBanner).
export async function fetchWithAuth(url: string, init: RequestInit = {}): Promise<Response> {
  const headers = new Headers(init.headers);
  if (desktopKey) headers.set(AUTH.desktopKeyHeader, desktopKey);
  if (!["GET", "HEAD", "OPTIONS"].includes((init.method ?? "GET").toUpperCase())) {
    headers.set(AUTH.csrfHeader, AUTH.csrfValue);
  }
  let res: Response;
  try {
    res = await fetch(url, { ...init, headers });
  } catch (e) {
    // An aborted request was cancelled on purpose, not lost.
    if (!(e instanceof DOMException && e.name === "AbortError")) {
      window.dispatchEvent(new Event("wyrm:unreachable"));
    }
    throw e;
  }
  const reachable = !GATEWAY_ERRORS.has(res.status);
  window.dispatchEvent(new Event(reachable ? "wyrm:reachable" : "wyrm:unreachable"));
  return res;
}

export function handleUnauthorized(): void {
  window.dispatchEvent(new Event(AUTH.unauthorizedEvent));
}

export async function fetchAuthStatus(): Promise<AuthStatusResponse | null> {
  try {
    const response = await fetchWithAuth(ENDPOINTS.auth.status());
    return response.ok ? await response.json() as AuthStatusResponse : null;
  } catch {
    return null;
  }
}
