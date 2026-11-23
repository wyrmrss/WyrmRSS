import type { LoginRequest } from "../types/LoginRequest";
import { ENDPOINTS } from "../utils/api";
import { fetchWithAuth } from "../utils/auth";

/** Signs in with a password. Returns the raw response for the sign-in form to check. */
export const login = (body: LoginRequest): Promise<Response> =>
  fetchWithAuth(ENDPOINTS.auth.login(), {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
