import type { AccountResponse } from "../types/AccountResponse";
import type { DeleteAccount } from "../types/DeleteAccount";
import type { UpdatePassword } from "../types/UpdatePassword";
import { ENDPOINTS } from "../utils/api";
import { fetchWithAuth } from "../utils/auth";
import { json, noContent } from "../utils/response";

export const getAccount = (): Promise<AccountResponse> =>
  fetchWithAuth(ENDPOINTS.account.get()).then(json<AccountResponse>);

export const updatePassword = (body: UpdatePassword): Promise<void> =>
  fetchWithAuth(ENDPOINTS.account.password(), {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  }).then(noContent);

/** The response clears the session cookie. */
export const deleteAccount = (body: DeleteAccount): Promise<void> =>
  fetchWithAuth(ENDPOINTS.account.delete(), {
    method: "DELETE",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  }).then(noContent);
