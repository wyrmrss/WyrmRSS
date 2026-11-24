import { useContext, useState } from "react";
import { isTauri } from "@tauri-apps/api/core";
import { AuthContext } from "../context/AuthContext";
import { Account } from "../components/settings/Account";
import { Appearance } from "../components/settings/Appearance";
import { Credentials } from "../components/settings/Credentials";
import { Feeds } from "../components/settings/Feeds";
import { Folders } from "../components/settings/Folders";
import { General } from "../components/settings/General";
import { Instance } from "../components/settings/Instance";
import { Integrations } from "../components/settings/Integrations";
import { Opml } from "../components/settings/Opml";
import { Shortcuts } from "../components/settings/Shortcuts";
import { Webhooks } from "../components/settings/Webhooks";
import { useAccount } from "../hooks/useAccount";
import { useIsMobile } from "../hooks/useIsMobile";
import { version } from "../../package.json";

type Tab =
  | "general"
  | "appearance"
  | "shortcuts"
  | "feeds"
  | "integrations"
  | "credentials"
  | "folders"
  | "webhooks"
  | "opml"
  | "account"
  | "instance";

export function SettingsPage() {
  // `?tab=account` opens the account section, e.g. after linking OIDC.
  const [tab, setTab] = useState<Tab>(() =>
    new URLSearchParams(location.search).get("tab") === "account" ? "account" : "general",
  );
  const isMobile = useIsMobile();
  const { signedIn } = useContext(AuthContext);
  // Desktop is signed in through its key and has no account to manage.
  const showAccount = signedIn && !isTauri();
  const { data: account } = useAccount();
  // Desktop is single-user, so registration doesn't apply there.
  const showAdmin = showAccount && account?.is_admin === true;

  return (
    <div className="settings-page">
      <div className="settings-tabs">
        <button
          className={`settings-tab${tab === "general" ? " active" : ""}`}
          onClick={() => setTab("general")}
        >
          General
        </button>
        <button
          className={`settings-tab${tab === "appearance" ? " active" : ""}`}
          onClick={() => setTab("appearance")}
        >
          Appearance
        </button>
        {!isMobile && (
          <button
            className={`settings-tab${tab === "shortcuts" ? " active" : ""}`}
            onClick={() => setTab("shortcuts")}
          >
            Shortcuts
          </button>
        )}
        <button
          className={`settings-tab${tab === "feeds" ? " active" : ""}`}
          onClick={() => setTab("feeds")}
        >
          Feeds
        </button>
        <button
          className={`settings-tab${tab === "integrations" ? " active" : ""}`}
          onClick={() => setTab("integrations")}
        >
          Integrations
        </button>
        <button
          className={`settings-tab${tab === "credentials" ? " active" : ""}`}
          onClick={() => setTab("credentials")}
        >
          Credentials
        </button>
        <button
          className={`settings-tab${tab === "folders" ? " active" : ""}`}
          onClick={() => setTab("folders")}
        >
          Folders
        </button>
        <button
          className={`settings-tab${tab === "webhooks" ? " active" : ""}`}
          onClick={() => setTab("webhooks")}
        >
          Webhooks
        </button>
        <button
          className={`settings-tab${tab === "opml" ? " active" : ""}`}
          onClick={() => setTab("opml")}
        >
          Import / Export
        </button>
        {showAccount && (
          <button
            className={`settings-tab${tab === "account" ? " active" : ""}`}
            onClick={() => setTab("account")}
          >
            Account
          </button>
        )}
        {showAdmin && (
          <>
            <div className="settings-tab-group">Admin</div>
            <button
              className={`settings-tab${tab === "instance" ? " active" : ""}`}
              onClick={() => setTab("instance")}
            >
              Instance
            </button>
          </>
        )}
      </div>

      <div className="settings-content">
        {tab === "general" && <General />}
        {tab === "appearance" && <Appearance />}
        {tab === "shortcuts" && !isMobile && <Shortcuts />}
        {tab === "feeds" && <Feeds />}
        {tab === "integrations" && <Integrations />}
        {tab === "credentials" && <Credentials />}
        {tab === "folders" && <Folders />}
        {tab === "webhooks" && <Webhooks />}
        {tab === "opml" && <Opml />}
        {tab === "account" && showAccount && <Account />}
        {tab === "instance" && showAdmin && <Instance />}
        <div className="settings-app-version">v{version}</div>
      </div>
    </div>
  );
}
