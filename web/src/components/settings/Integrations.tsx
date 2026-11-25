import { useAccount } from "../../hooks/useAccount";
import { useLocalInstance } from "../../hooks/useLocalInstance";
import { RSSHubForm } from "../integrations/RSSHubForm";

export function Integrations() {
  const { data: instance } = useLocalInstance();
  const { data: account } = useAccount();
  if (!instance || !account) return null;
  return <RSSHubForm initialInstance={instance.rsshub_instance ?? ""} canEdit={account.is_admin} />;
}
