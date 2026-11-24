import { createContext } from "react";
import type { InstanceStatusResponse } from "../types/InstanceStatusResponse";

/** The instance SetupGate loaded. Null when /instance couldn't be reached. */
export const InstanceContext = createContext<InstanceStatusResponse | null>(null);
