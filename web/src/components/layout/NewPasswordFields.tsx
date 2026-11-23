import { useState } from "react";
import type { NewPassword } from "../../hooks/useNewPassword";
import { AUTH } from "../../utils/constants";

/** Password and confirm fields for the setup and sign-up screens. */
export function NewPasswordFields({ newPassword, disabled }: { newPassword: NewPassword; disabled?: boolean }) {
  const [shown, setShown] = useState(false);
  const type = shown ? "text" : "password";

  return (
    <>
      <label className="setup-field">
        Password
        <span className="setup-password">
          <input
            type={type}
            value={newPassword.password}
            onChange={(e) => newPassword.setPassword(e.target.value)}
            autoComplete="new-password"
            placeholder={`At least ${AUTH.minPasswordLength} characters`}
            disabled={disabled}
          />
          <button type="button" className="btn btn-ghost" onClick={() => setShown((isShown) => !isShown)}>
            {shown ? "Hide" : "Show"}
          </button>
        </span>
      </label>

      <label className="setup-field">
        Confirm password
        <input
          type={type}
          value={newPassword.confirm}
          onChange={(e) => newPassword.setConfirm(e.target.value)}
          autoComplete="new-password"
          aria-invalid={newPassword.showMismatch}
          disabled={disabled}
        />
      </label>

      {newPassword.showMismatch && <p className="api-key-error">The passwords don't match.</p>}
    </>
  );
}
