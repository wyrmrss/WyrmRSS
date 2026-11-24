import type { InstanceSetup } from "../types/InstanceSetup";
import type { LocalInstanceConfigResponse } from "../types/LocalInstanceConfigResponse";
import type { LocalInstanceStatusResponse } from "../types/LocalInstanceStatusResponse";
import type { UpdateLocalInstance } from "../types/UpdateLocalInstance";
import { ENDPOINTS } from "../utils/api";
import { fetchWithAuth } from "../utils/auth";
import { json, noContent } from "../utils/response";

// Public endpoint, so no auth header: it's used before anyone can sign in.
export const getLocalInstanceStatus = (): Promise<LocalInstanceStatusResponse> =>
  fetch(ENDPOINTS.instance.status()).then(json<LocalInstanceStatusResponse>);

/** Creates the owner account; the response sets the session cookie. */
// fetchWithAuth for the CSRF header the server checks on POSTs.
export const setupInstance = (body: InstanceSetup): Promise<void> =>
  fetchWithAuth(ENDPOINTS.instance.setup(), {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  }).then(noContent);

/** Admins only. */
export const getLocalInstanceConfig = (): Promise<LocalInstanceConfigResponse> =>
  fetchWithAuth(ENDPOINTS.admin.instance()).then(json<LocalInstanceConfigResponse>);

/** Admins only. */
export const updateLocalInstance = (body: UpdateLocalInstance): Promise<LocalInstanceConfigResponse> =>
  fetchWithAuth(ENDPOINTS.admin.instance(), {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  }).then(json<LocalInstanceConfigResponse>);
