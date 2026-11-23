import { type SubmitEvent, useState } from "react";
import { login } from "../../api/auth";
import { useTheme } from "../../hooks/useTheme";
import { ENDPOINTS } from "../../utils/api";
import { ApiError, throwApiError } from "../../utils/error";
import type { AuthMethod } from "../../types/AuthMethod";

interface SignInScreenProps {
  methods: AuthMethod[];
  /** Message for an `auth_error` the server redirected back with. */
  callbackError: string | null;
}

export function SignInScreen({ methods, callbackError }: SignInScreenProps) {
  const { theme } = useTheme();
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState(callbackError);

  async function handlePasswordSignIn(e: SubmitEvent<HTMLFormElement>) {
    e.preventDefault();
    setSubmitting(true);
    setError(null);
    try {
      const response = await login({ username, password });
      if (response.ok) {
        // Reload to start the app with the new session.
        window.location.reload();
        return;
      }
      await throwApiError(response);
    } catch (e) {
      setError(e instanceof ApiError ? e.message : "Unable to reach the server. Please try again.");
    }
    setSubmitting(false);
  }

  const passwordEnabled = methods.includes("password");
  const oidcEnabled = methods.includes("oidc");
  const returnTo = location.pathname + location.search + location.hash;
  const oidcHref = `${ENDPOINTS.auth.oidcLogin()}?${new URLSearchParams({ return_to: returnTo })}`;

  return (
    <div className="api-key-gate">
      <form className="api-key-form" onSubmit={handlePasswordSignIn}>
        <img
          className="setup-logo"
          src={theme === "light" ? "/l-o-dark.svg" : "/l-o-light.svg"}
          alt="Wyrm-RSS"
        />
        {passwordEnabled && (
          <>
            <label className="setup-field">
              Username
              <input
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                autoComplete="username"
                autoFocus
                disabled={submitting}
              />
            </label>
            <label className="setup-field">
              Password
              <input
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                autoComplete="current-password"
                disabled={submitting}
              />
            </label>
            <button
              type="submit"
              className="btn btn-primary"
              disabled={submitting || username.trim() === "" || password === ""}
            >
              {submitting ? "Signing in…" : "Sign in"}
            </button>
          </>
        )}
        {oidcEnabled && (
          <button
            type="button"
            className={passwordEnabled ? "btn" : "btn btn-primary"}
            disabled={submitting}
            onClick={() => window.location.assign(oidcHref)}
          >
            Sign in with OpenID Connect
          </button>
        )}
        {error && <p className="api-key-error" role="alert">{error}</p>}
      </form>
    </div>
  );
}
