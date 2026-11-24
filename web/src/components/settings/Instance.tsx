import { type SubmitEvent, useState } from "react";
import { useLocalInstanceConfig, useUpdateLocalInstance } from "../../hooks/useLocalInstanceConfig";
import type { LocalInstanceConfigResponse } from "../../types/LocalInstanceConfigResponse";
import type { RegistrationMode } from "../../types/RegistrationMode";

const REGISTRATION_OPTIONS: { value: RegistrationMode; label: string }[] = [
  { value: "closed", label: "Closed" },
  { value: "open", label: "Open" },
];

function LocalInstanceForm({ initial }: { initial: LocalInstanceConfigResponse }) {
  const [registrationMode, setRegistrationMode] = useState(initial.registration_mode);
  const update = useUpdateLocalInstance();

  function handleSubmit(e: SubmitEvent<HTMLFormElement>) {
    e.preventDefault();
    update.mutate({ registration_mode: registrationMode });
  }

  return (
    <form className="settings-section" onSubmit={handleSubmit}>
      <h2 className="settings-section-title">Registration</h2>
      <div className="settings-field">
        <label>Sign-up</label>
        <div className="segmented">
          {REGISTRATION_OPTIONS.map(({ value, label }) => (
            <button
              key={value}
              type="button"
              className={`segmented-btn${registrationMode === value ? " active" : ""}`}
              onClick={() => setRegistrationMode(value)}
            >
              {label}
            </button>
          ))}
        </div>
      </div>
      <p className="settings-hint">
        {registrationMode === "open"
          ? "Anyone who can reach this instance can create an account."
          : "No one can create an account."}
      </p>

      <div className="settings-form-actions">
        <button className="btn btn-primary" type="submit" disabled={update.isPending}>
          {update.isPending ? "Saving…" : "Save"}
        </button>
        {update.isSuccess && <span className="settings-status settings-status-ok">Saved.</span>}
        {update.isError && <span className="settings-status settings-status-err">Failed to save.</span>}
      </div>
    </form>
  );
}

export function Instance() {
  const { data } = useLocalInstanceConfig();
  if (!data) return null;
  return <LocalInstanceForm initial={data} />;
}
