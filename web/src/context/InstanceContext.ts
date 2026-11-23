import { createContext } from "react";
import type { LocalInstanceResponse } from "../types/LocalInstanceResponse";

/** The instance SetupGate loaded. Null when /instance couldn't be reached. */
export const InstanceContext = createContext<LocalInstanceResponse | null>(null);
