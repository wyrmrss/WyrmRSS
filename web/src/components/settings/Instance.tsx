import { type SubmitEvent, useState } from "react";
import { isTauri } from "@tauri-apps/api/core";
import { useLocalInstance, useUpdateLocalInstance } from "../../hooks/useLocalInstance";
import type { LocalInstanceResponse } from "../../types/LocalInstanceResponse";
import type { RegistrationMode } from "../../types/RegistrationMode";

const REGISTRATION_OPTIONS: { value: RegistrationMode; label: string }[] = [
  { value: "closed", label: "Closed" },
  { value: "open", label: "Open" },
];

function LocalInstanceForm({ initial }: { initial: LocalInstanceResponse }) {
  const [registrationMode, setRegistrationMode] = useState(initial.registration_mode);
  const [pollInterval, setPollInterval] = useState(String(initial.feed_poll_interval_secs));
  const [manualPollInterval, setManualPollInterval] = useState(String(initial.manual_poll_interval_secs));
  const [httpTimeout, setHttpTimeout] = useState(String(initial.http_timeout_secs));
  const [connectTimeout, setConnectTimeout] = useState(String(initial.http_connect_timeout_secs));
  const [retries, setRetries] = useState(String(initial.http_retries));
  const [userAgent, setUserAgent] = useState(initial.http_user_agent ?? "");
  const update = useUpdateLocalInstance();
  // Desktop is single-user, so registration doesn't apply there.
  const showRegistration = !isTauri();

  function handleSubmit(e: SubmitEvent<HTMLFormElement>) {
    e.preventDefault();
    update.mutate({
      registration_mode: showRegistration ? registrationMode : undefined,
      feed_poll_interval_secs: parseInt(pollInterval),
      manual_poll_interval_secs: parseInt(manualPollInterval),
      http_timeout_secs: parseInt(httpTimeout),
      http_connect_timeout_secs: parseInt(connectTimeout),
      http_retries: parseInt(retries),
      http_user_agent: userAgent.trim() || null,
    });
  }

  return (
    <form className="settings-section" onSubmit={handleSubmit}>
      {showRegistration && (
        <>
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
        </>
      )}

      <h2 className="settings-section-title">Polling</h2>
      <div className="settings-field">
        <label>Poll interval (s)</label>
        <input type="number" min={60} value={pollInterval} onChange={(e) => setPollInterval(e.target.value)} required />
      </div>
      <div className="settings-field">
        <label>Manual refresh limit (s)</label>
        <input type="number" min={0} value={manualPollInterval} onChange={(e) => setManualPollInterval(e.target.value)} required />
      </div>
      <p className="settings-hint">How often each user can refresh their feeds.</p>

      <h2 className="settings-section-title">HTTP</h2>
      <div className="settings-field">
        <label>Timeout (s)</label>
        <input type="number" min={1} value={httpTimeout} onChange={(e) => setHttpTimeout(e.target.value)} required />
      </div>
      <div className="settings-field">
        <label>Connect timeout (s)</label>
        <input type="number" min={1} value={connectTimeout} onChange={(e) => setConnectTimeout(e.target.value)} required />
      </div>
      <div className="settings-field">
        <label>Retries</label>
        <input type="number" min={0} value={retries} onChange={(e) => setRetries(e.target.value)} required />
      </div>
      <div className="settings-field">
        <label>User agent</label>
        <input
          type="text"
          value={userAgent}
          onChange={(e) => setUserAgent(e.target.value)}
          placeholder="Default (wyrm-rss/version)"
        />
      </div>

      <div className="settings-form-actions">
        <button className="btn btn-primary" type="submit" disabled={update.isPending}>
          {update.isPending ? "Saving…" : "Save"}
        </button>
        {update.isSuccess && <span className="settings-status settings-status-ok">Saved.</span>}
        {update.isError && <span className="settings-status settings-status-err">Failed to save.</span>}
      </div>
      <p className="settings-hint">All changes take effect immediately.</p>
    </form>
  );
}

export function Instance() {
  const { data } = useLocalInstance();
  if (!data) return null;
  return <LocalInstanceForm initial={data} />;
}
