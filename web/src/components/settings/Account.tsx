import { type SubmitEvent, useContext, useState } from "react";
import { AuthContext } from "../../context/AuthContext";
import { useAccount, useDeleteAccount, useUpdatePassword } from "../../hooks/useAccount";
import { useNewPassword } from "../../hooks/useNewPassword";
import type { AccountResponse } from "../../types/AccountResponse";
import { ENDPOINTS } from "../../utils/api";
import { AUTH } from "../../utils/constants";

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : "Something went wrong. Please try again.";
}

function PasswordForm({ hasPassword }: { hasPassword: boolean }) {
  const { mutate, isPending, error, isSuccess, reset } = useUpdatePassword();
  const [currentPassword, setCurrentPassword] = useState("");
  const newPassword = useNewPassword();

  const ready = (!hasPassword || currentPassword !== "") && newPassword.ready;

  function handleSubmit(e: SubmitEvent<HTMLFormElement>) {
    e.preventDefault();
    if (!ready || isPending) return;
    mutate(
      { current_password: hasPassword ? currentPassword : undefined, new_password: newPassword.password },
      {
        onSuccess: () => {
          setCurrentPassword("");
          newPassword.clear();
        },
      },
    );
  }

  return (
    <form className="settings-section account-form" onSubmit={handleSubmit} onChange={() => isSuccess && reset()}>
      <h2 className="settings-section-title">{hasPassword ? "Change password" : "Set a password"}</h2>
      {!hasPassword && (
        <p className="settings-hint">A password lets you sign in when your identity provider is unavailable.</p>
      )}
      {hasPassword && (
        <div className="settings-field">
          <label>Current password</label>
          <input
            type="password"
            value={currentPassword}
            onChange={(e) => setCurrentPassword(e.target.value)}
            autoComplete="current-password"
          />
        </div>
      )}
      <div className="settings-field">
        <label>New password</label>
        <input
          type="password"
          value={newPassword.password}
          onChange={(e) => newPassword.setPassword(e.target.value)}
          autoComplete="new-password"
          placeholder={`At least ${AUTH.minPasswordLength} characters`}
        />
      </div>
      <div className="settings-field">
        <label>Confirm new password</label>
        <input
          type="password"
          value={newPassword.confirm}
          onChange={(e) => newPassword.setConfirm(e.target.value)}
          autoComplete="new-password"
          aria-invalid={newPassword.showMismatch}
        />
      </div>
      <div className="settings-form-actions">
        <button type="submit" className="btn btn-primary" disabled={!ready || isPending}>
          {isPending ? "Saving…" : hasPassword ? "Change password" : "Set password"}
        </button>
      </div>
      <div className="account-form-status">
        {newPassword.showMismatch && <p className="settings-status settings-status-err">The passwords don't match.</p>}
        {error && <p className="settings-status settings-status-err" role="alert">{errorMessage(error)}</p>}
        {isSuccess && <p className="settings-status settings-status-ok">Password saved.</p>}
      </div>
    </form>
  );
}

function DeleteAccountForm({ account }: { account: AccountResponse }) {
  const { mutate, isPending, error } = useDeleteAccount();
  const [confirmation, setConfirmation] = useState("");

  function handleSubmit(e: SubmitEvent<HTMLFormElement>) {
    e.preventDefault();
    if (confirmation === "" || isPending) return;
    const body = account.has_password ? { password: confirmation } : { username: confirmation };
    // Reload to land on the sign-in screen, the response already cleared the session cookie.
    mutate(body, { onSuccess: () => window.location.reload() });
  }

  return (
    <form className="settings-section account-form" onSubmit={handleSubmit}>
      <h2 className="settings-section-title">Delete account</h2>
      <p className="settings-hint">
        Deletes your account and all your feeds, folders, webhooks and saved posts. This can't be undone.
        Export your feeds as OPML first if you want to keep them.
      </p>
      <div className="settings-field">
        <label>{account.has_password ? "Password" : `Type "${account.username}" to confirm`}</label>
        <input
          type={account.has_password ? "password" : "text"}
          value={confirmation}
          onChange={(e) => setConfirmation(e.target.value)}
          autoComplete={account.has_password ? "current-password" : "off"}
        />
      </div>
      <div className="settings-form-actions">
        <button type="submit" className="btn" disabled={confirmation === "" || isPending}>
          {isPending ? "Deleting…" : "Delete account"}
        </button>
      </div>
      <div className="account-form-status">
        {error && <p className="settings-status settings-status-err" role="alert">{errorMessage(error)}</p>}
      </div>
    </form>
  );
}

function OidcLink({ linked }: { linked: boolean }) {
  const { linkError } = useContext(AuthContext);
  const params = new URLSearchParams({ link: "true", return_to: "/settings?tab=account" });

  return (
    <div className="settings-section account-form">
      <h2 className="settings-section-title">OpenID Connect</h2>
      <p className="settings-hint">
        {linked
          ? "Your account is linked to your identity provider, so you can sign in with it."
          : "Link your account to your identity provider to sign in with it."}
      </p>
      <div className="settings-form-actions">
        <button
          type="button"
          className="btn"
          onClick={() => window.location.assign(`${ENDPOINTS.auth.oidcLogin()}?${params}`)}
        >
          {linked ? "Link a different identity" : "Link OIDC"}
        </button>
      </div>
      <div className="account-form-status">
        {linkError && <p className="settings-status settings-status-err" role="alert">{linkError}</p>}
      </div>
    </div>
  );
}

export function Account() {
  const { signOut, methods } = useContext(AuthContext);
  const { data: account, isLoading } = useAccount();
  const [signOutError, setSignOutError] = useState(false);
  const [signingOut, setSigningOut] = useState(false);

  async function handleSignOut() {
    setSigningOut(true);
    setSignOutError(false);
    try {
      await signOut();
    } catch {
      setSignOutError(true);
      setSigningOut(false);
    }
  }

  if (isLoading) return <div className="pane-empty">Loading…</div>;

  return (
    <section className="settings-section">
      <h2 className="settings-section-title">Account</h2>
      {account && (
        <p className="account-signed-in">
          Signed in as <strong>{account.username}</strong>{account.is_admin && " (admin)"}
        </p>
      )}
      <div className="settings-form-actions">
        <button className="btn" disabled={signingOut} onClick={handleSignOut}>
          {signingOut ? "Signing out…" : "Sign out"}
        </button>
        {signOutError && <span className="settings-status settings-status-err" role="alert">Sign-out failed. Please try again.</span>}
      </div>

      {account && <PasswordForm hasPassword={account.has_password} />}
      {account && methods.includes("oidc") && <OidcLink linked={account.oidc_linked} />}
      {account && <DeleteAccountForm account={account} />}
    </section>
  );
}
