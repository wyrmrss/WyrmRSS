import { type SubmitEvent, useEffect, useState } from "react";
import { useNewPassword } from "../../hooks/useNewPassword";
import { useTheme } from "../../hooks/useTheme";
import { setupInstance } from "../../api/instance";
import { ENDPOINTS } from "../../utils/api";
import { fetchAuthStatus } from "../../utils/auth";
import { AUTH } from "../../utils/constants";
import { NewPasswordFields } from "./NewPasswordFields";

export function SetupScreen() {
  const { theme } = useTheme();
  const [token, setToken] = useState("");
  const [username, setUsername] = useState("admin");
  const newPassword = useNewPassword();
  const [oidcEnabled, setOidcEnabled] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    void fetchAuthStatus().then((status) => {
      if (status) setOidcEnabled(status.methods.includes("oidc"));
    });
  }, []);

  const ready = token.trim() !== "" && username.trim() !== "" && newPassword.ready;

  async function createAccount(linkOidc: boolean) {
    setSubmitting(true);
    setError(null);
    try {
      await setupInstance({ token, username, password: newPassword.password });
      // Signed in now: reload so SetupGate sees the finished setup, or continue to OIDC.
      if (linkOidc) window.location.assign(`${ENDPOINTS.auth.oidcLogin()}?link=true`);
      else window.location.reload();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Setup failed.");
      setSubmitting(false);
    }
  }

  function handleSubmit(e: SubmitEvent<HTMLFormElement>) {
    e.preventDefault();
    if (ready && !submitting) void createAccount(false);
  }

  return (
    <div className="api-key-gate">
      <form className="api-key-form" onSubmit={handleSubmit}>
        <img
          className="setup-logo"
          src={theme === "light" ? "/l-o-dark.svg" : "/l-o-light.svg"}
          alt="Wyrm-RSS"
        />
        <h2 className="setup-title">Set up the owner account</h2>
        <p>This account manages the instance and its users.</p>

        <label className="setup-field">
          Setup token
          <input
            value={token}
            onChange={(e) => setToken(e.target.value)}
            autoComplete="off"
            spellCheck={false}
            autoFocus
          />
          <small>Printed in the server log when Wyrm starts, for example in <code>docker logs wyrm</code>.</small>
        </label>

        <label className="setup-field">
          Username
          <input
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            autoComplete="username"
            placeholder={`${AUTH.usernameLength.min} to ${AUTH.usernameLength.max} letters, digits or _`}
          />
        </label>

        <NewPasswordFields newPassword={newPassword} />

        <p className="setup-note">Any existing feeds, folders and settings will belong to this account.</p>
        {error && <p className="api-key-error" role="alert">{error}</p>}

        <button type="submit" className="btn btn-primary" disabled={!ready || submitting}>
          {submitting ? "Creating account…" : "Create account"}
        </button>
        {oidcEnabled && (
          <button
            type="button"
            className="btn btn-ghost"
            disabled={!ready || submitting}
            onClick={() => void createAccount(true)}
          >
            Create account and link OIDC
          </button>
        )}
      </form>
    </div>
  );
}
