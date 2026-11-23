import type { InstanceSetup } from "../types/InstanceSetup";
import type { LocalInstanceResponse } from "../types/LocalInstanceResponse";
import { ENDPOINTS } from "../utils/api";
import { fetchWithAuth } from "../utils/auth";
import { json, noContent } from "../utils/response";

// Public endpoint, so no auth header: it's used before anyone can sign in.
export const getLocalInstance = (): Promise<LocalInstanceResponse> =>
  fetch(ENDPOINTS.instance.get()).then(json<LocalInstanceResponse>);

/** Creates the owner account; the response sets the session cookie. */
// fetchWithAuth for the CSRF header the server checks on POSTs.
export const setupInstance = (body: InstanceSetup): Promise<void> =>
  fetchWithAuth(ENDPOINTS.instance.setup(), {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  }).then(noContent);
