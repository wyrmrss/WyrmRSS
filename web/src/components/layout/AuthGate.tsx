import { useEffect, useState } from "react";
import { fetchAuthStatus, fetchWithAuth } from "../../utils/auth";
import { ENDPOINTS } from "../../utils/api";
import { AUTH } from "../../utils/constants";
import { AuthContext } from "../../context/AuthContext";
import type { AuthStatusResponse } from "../../types/AuthStatusResponse";

export function AuthGate({ children }: { children: React.ReactNode }) {
  // Preserve immediate access for open instances; a protected API returning
  // 401 locks the gate and prompts for the configured sign-in methods.
  const [locked, setLocked] = useState(false);

  const [error] = useState<string | null>(() => {
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

  const [status, setStatus] = useState<AuthStatusResponse>({
    methods: [], signed_in: false,
  });

  // Runs on mount and on lock changes, so a burst of 401s fetches status once.
  useEffect(() => {
    let active = true;

    void fetchAuthStatus().then((status) => {
      if (active && status) setStatus(status);
    });

    return () => { active = false; };
  }, [locked]);

  // Listener for unauthorizedEvent (wyrm:unauthorized) set when 
  // API calls return a 401 response (see utils/response.ts).
  useEffect(() => {
    const handle = () => {
      setLocked(true);
      setStatus((current) => ({ ...current, signed_in: false }));
    };

    window.addEventListener(AUTH.unauthorizedEvent, handle);

    return () => window.removeEventListener(AUTH.unauthorizedEvent, handle);
  }, []);

  // Strip the sign-in error param once it's been read into `error`.
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

  if (!locked) return (
    <AuthContext value={{ signedIn: status.signed_in, signOut }}>
      {children}
    </AuthContext>
  );

  const oidcEnabled = status.methods.includes("oidc");

  return (
    <div className="api-key-gate">
      <div className="api-key-form">
        <h2>Sign in to Wyrm</h2>
        {oidcEnabled && (
          <>
            <p>Sign in with your identity provider to access this instance.</p>
            <a className="btn btn-primary" href={`${ENDPOINTS.auth.login()}?${new URLSearchParams({ return_to: location.pathname + location.search + location.hash })}`}>
              Sign in with OpenID Connect
            </a>
          </>
        )}
        {error && <p className="api-key-error" role="alert">{error}</p>}
      </div>
    </div>
  );
}
