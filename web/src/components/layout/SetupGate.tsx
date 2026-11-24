import { useEffect, useState } from "react";
import { getLocalInstanceStatus } from "../../api/instance";
import { InstanceContext } from "../../context/InstanceContext";
import type { LocalInstanceStatusResponse } from "../../types/LocalInstanceStatusResponse";
import { SetupScreen } from "./SetupScreen";

// Sits outside AuthGate, since nobody can sign in before setup.
export function SetupGate({ children }: { children: React.ReactNode }) {
  const [instance, setInstance] = useState<LocalInstanceStatusResponse | null>(null);
  const [failed, setFailed] = useState(false);

  useEffect(() => {
    getLocalInstanceStatus()
      .then(setInstance)
      // If the request fails, show the app; its own requests will surface the error.
      .catch(() => setFailed(true));
  }, []);

  if (failed) return children;
  if (!instance) return null;
  if (!instance.is_setup) return <SetupScreen />;
  return <InstanceContext value={instance}>{children}</InstanceContext>;
}
