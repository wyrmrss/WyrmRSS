import { createContext } from "react";
import type { AuthMethod } from "../types/AuthMethod";

export const AuthContext = createContext({
  signedIn: false,
  signOut: async (): Promise<void> => {},
  methods: [] as AuthMethod[],
  /** Message for an `auth_error` from a failed OIDC link, shown in the account section. */
  linkError: null as string | null,
});
