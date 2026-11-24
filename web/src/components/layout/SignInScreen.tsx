import { type SubmitEvent, useContext, useState } from "react";
import { login, register, startOidcLink } from "../../api/auth";
import { InstanceContext } from "../../context/InstanceContext";
import { useNewPassword } from "../../hooks/useNewPassword";
import { useTheme } from "../../hooks/useTheme";
import { ENDPOINTS } from "../../utils/api";
import { AUTH } from "../../utils/constants";
import { ApiError, throwApiError } from "../../utils/error";
import type { AuthMethod } from "../../types/AuthMethod";
import { NewPasswordFields } from "./NewPasswordFields";

interface SignInScreenProps {
  methods: AuthMethod[];
  /** Message for an `auth_error` the server redirected back with. */
  callbackError: string | null;
}

export function SignInScreen({ methods, callbackError }: SignInScreenProps) {
  const { theme } = useTheme();
  const instance = useContext(InstanceContext);
  const [signingUp, setSigningUp] = useState(false);
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const newPassword = useNewPassword();
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState(callbackError);

  async function submit(linkOidc: boolean) {
    setSubmitting(true);
    setError(null);
    try {
      const response = signingUp
        ? await register({ username, password: newPassword.password })
        : await login({ username, password });
      if (response.ok) {
        // Signed in now: reload to start the app with the new session, or continue to OIDC.
        if (linkOidc) startOidcLink(returnTo);
        else window.location.reload();
        return;
      }
      await throwApiError(response);
    } catch (e) {
      setError(e instanceof ApiError ? e.message : "Unable to reach the server. Please try again.");
    }
    setSubmitting(false);
  }

  function handleSubmit(e: SubmitEvent<HTMLFormElement>) {
    e.preventDefault();
    void submit(false);
  }

  function switchMode() {
    setSigningUp((isSigningUp) => !isSigningUp);
    setError(null);
  }

  const passwordEnabled = methods.includes("password");
  const oidcEnabled = methods.includes("oidc");
  // Sign-up creates a password account, so it needs password sign-in too.
  const registrationOpen = passwordEnabled && instance?.registration_mode === "open";
  const ready = username.trim() !== "" && (signingUp ? newPassword.ready : password !== "");
  const returnTo = location.pathname + location.search + location.hash;
  const oidcHref = `${ENDPOINTS.auth.oidcLogin()}?${new URLSearchParams({ return_to: returnTo })}`;

  return (
    <div className="api-key-gate">
      <form className="api-key-form" onSubmit={handleSubmit}>
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
                placeholder={
                  signingUp ? `${AUTH.usernameLength.min} to ${AUTH.usernameLength.max} letters, digits or _` : undefined
                }
                autoFocus
                disabled={submitting}
              />
            </label>
            {signingUp ? (
              <NewPasswordFields newPassword={newPassword} disabled={submitting} />
            ) : (
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
            )}
            <button type="submit" className="btn btn-primary" disabled={submitting || !ready}>
              {signingUp
                ? submitting ? "Signing up…" : "Sign up"
                : submitting ? "Signing in…" : "Sign in"}
            </button>
            {oidcEnabled && signingUp && (
              <button
                type="button"
                className="btn"
                disabled={submitting || !ready}
                onClick={() => void submit(true)}
              >
                Sign up and link OIDC
              </button>
            )}
          </>
        )}
        {oidcEnabled && !signingUp && (
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
        {registrationOpen && (
          <button type="button" className="link-button" disabled={submitting} onClick={switchMode}>
            {signingUp ? "Sign in" : "Sign up"}
          </button>
        )}
      </form>
    </div>
  );
}
