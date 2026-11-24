import { useEffect, useState } from "react";
import { getLocalInstance } from "../../api/instance";
import { InstanceContext } from "../../context/InstanceContext";
import type { InstanceStatusResponse } from "../../types/InstanceStatusResponse";
import { SetupScreen } from "./SetupScreen";

// Sits outside AuthGate, since nobody can sign in before setup.
export function SetupGate({ children }: { children: React.ReactNode }) {
  const [instance, setInstance] = useState<InstanceStatusResponse | null>(null);
  const [failed, setFailed] = useState(false);

  useEffect(() => {
    getLocalInstance()
      .then(setInstance)
      // If the request fails, show the app; its own requests will surface the error.
      .catch(() => setFailed(true));
  }, []);

  if (failed) return children;
  if (!instance) return null;
  if (!instance.is_setup) return <SetupScreen />;
  return <InstanceContext value={instance}>{children}</InstanceContext>;
}
