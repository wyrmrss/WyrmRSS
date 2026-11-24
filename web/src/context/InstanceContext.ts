import { createContext } from "react";
import type { LocalInstanceStatusResponse } from "../types/LocalInstanceStatusResponse";

/** The instance SetupGate loaded. Null when /instance/status couldn't be reached. */
export const InstanceContext = createContext<LocalInstanceStatusResponse | null>(null);
