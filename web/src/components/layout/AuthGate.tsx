import { useEffect, useState } from "react";
import { fetchAuthStatus, fetchWithAuth } from "../../utils/auth";
import { ENDPOINTS } from "../../utils/api";
import { AUTH } from "../../utils/constants";
import { AuthContext } from "../../context/AuthContext";
import type { AuthStatusResponse } from "../../types/AuthStatusResponse";
import { SignInScreen } from "./SignInScreen";

// Waits for /auth/status before rendering, then shows the app or the sign-in screen.
// A 401 later on (e.g. an expired session) switches to the sign-in screen.
export function AuthGate({ children }: { children: React.ReactNode }) {
  // Null until /auth/status answers, and stays null if it can't be reached.
  const [status, setStatus] = useState<AuthStatusResponse | null>(null);
  const [checked, setChecked] = useState(false);

  const [callbackError] = useState<string | null>(() => {
    const callbackError = new URLSearchParams(location.search).get(AUTH.errorParam);
    if (callbackError === "oidc") {
      return "Sign-in failed. Check that your account is allowed, then try again.";
    }
    if (callbackError === "oidc_unavailable") {
      return "The identity provider is unavailable. Try again later.";
    }
    if (callbackError === "oidc_not_linked") {
      return "No account is linked to this identity. Sign in with your password and link it from your account settings.";
    }
    if (callbackError === "oidc_already_linked") {
      return "This identity is already linked to another account.";
    }
    return null;
  });

  useEffect(() => {
    let active = true;

    void fetchAuthStatus().then((status) => {
      if (!active) return;
      setStatus(status);
      setChecked(true);
    });

    return () => { active = false; };
  }, []);

  // Listener for unauthorizedEvent (wyrm:unauthorized) set when
  // API calls return a 401 response (see utils/response.ts).
  useEffect(() => {
    const handle = () => {
      setStatus((current) => ({ methods: current?.methods ?? [], signed_in: false }));
    };

    window.addEventListener(AUTH.unauthorizedEvent, handle);

    return () => window.removeEventListener(AUTH.unauthorizedEvent, handle);
  }, []);

  // Strip the sign-in error param once it's been read into `callbackError`.
  useEffect(() => {
    const url = new URL(location.href);
    if (url.searchParams.has(AUTH.errorParam)) {
      url.searchParams.delete(AUTH.errorParam);
      history.replaceState(null, "", url);
    }
  }, []);

  async function signOut() {
    const response = await fetchWithAuth(ENDPOINTS.auth.logout(), { method: "POST" });
    if (!response.ok) throw new Error("Sign-out failed. Please try again.");
    // A full navigation clears reader caches and returns to the sign-in screen.
    window.location.reload();
  }

  if (!checked) return null;

  // No status means the server couldn't be reached. Show the app, which reports that itself.
  if (!status || status.signed_in) return (
    <AuthContext value={{ signedIn: status?.signed_in ?? false, signOut }}>
      {children}
    </AuthContext>
  );

  return <SignInScreen methods={status.methods} callbackError={callbackError} />;
}
