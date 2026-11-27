import { useLocalInstance } from "../../hooks/useLocalInstance";
import { SetupScreen } from "./SetupScreen";

// Sits outside AuthGate, since nobody can sign in before setup.
export function SetupGate({ children }: { children: React.ReactNode }) {
  const { data: instance, isError } = useLocalInstance();

  // If the request fails, show the app. Its own requests will surface the error.
  if (isError) return children;
  if (!instance) return null;
  if (!instance.is_setup) return <SetupScreen />;
  return children;
}
