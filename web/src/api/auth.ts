import type { LoginRequest } from "../types/LoginRequest";
import type { RegisterAccount } from "../types/RegisterAccount";
import { ENDPOINTS } from "../utils/api";
import { fetchWithAuth } from "../utils/auth";

/** Signs in with a password. Returns the raw response for the sign-in form to check. */
export const login = (body: LoginRequest): Promise<Response> =>
  fetchWithAuth(ENDPOINTS.auth.login(), {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });

/** Sends the browser to the identity provider to link it to the signed-in account. */
export function startOidcLink(returnTo?: string) {
  const params = new URLSearchParams({ link: "true" });
  if (returnTo) params.set("return_to", returnTo);
  window.location.assign(`${ENDPOINTS.auth.oidcLogin()}?${params}`);
}

/** Creates an account and signs in. Returns the raw response for the sign-up form to check. */
export const register = (body: RegisterAccount): Promise<Response> =>
  fetchWithAuth(ENDPOINTS.auth.register(), {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
