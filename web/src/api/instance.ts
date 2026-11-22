import type { InstanceSetup } from "../types/InstanceSetup";
import type { LocalInstanceResponse } from "../types/LocalInstanceResponse";
import { ENDPOINTS } from "../utils/api";
import { json, noContent } from "../utils/response";

// Public endpoints, so no auth header: they're used before anyone can sign in.
export const getLocalInstance = (): Promise<LocalInstanceResponse> =>
  fetch(ENDPOINTS.instance.get()).then(json<LocalInstanceResponse>);

/** Creates the owner account; the response sets the session cookie. */
export const setupInstance = (body: InstanceSetup): Promise<void> =>
  fetch(ENDPOINTS.instance.setup(), {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  }).then(noContent);
