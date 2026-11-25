import type { InstanceSetup } from "../types/InstanceSetup";
import type { LocalInstanceResponse } from "../types/LocalInstanceResponse";
import type { UpdateLocalInstance } from "../types/UpdateLocalInstance";
import { ENDPOINTS } from "../utils/api";
import { fetchWithAuth } from "../utils/auth";
import { json, noContent } from "../utils/response";

/** Public, readable before sign-in. Some fields are hidden until signed in. */
// fetchWithAuth so the desktop key is sent, which counts as signed in.
export const getLocalInstance = (): Promise<LocalInstanceResponse> =>
  fetchWithAuth(ENDPOINTS.instance.get()).then(json<LocalInstanceResponse>);

/** Creates the owner account; the response sets the session cookie. */
// fetchWithAuth for the CSRF header the server checks on POSTs.
export const setupInstance = (body: InstanceSetup): Promise<void> =>
  fetchWithAuth(ENDPOINTS.instance.setup(), {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  }).then(noContent);

/** Admins only. */
export const updateLocalInstance = (body: UpdateLocalInstance): Promise<LocalInstanceResponse> =>
  fetchWithAuth(ENDPOINTS.admin.instance(), {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  }).then(json<LocalInstanceResponse>);
