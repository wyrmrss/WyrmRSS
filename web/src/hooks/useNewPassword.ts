import { useState } from "react";
import { AUTH } from "../utils/constants";

/** State and checks for a new password typed twice. */
export function useNewPassword() {
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");

  // Counts characters, not UTF-16 units, like the server does.
  const longEnough = [...password].length >= AUTH.minPasswordLength;
  const showMismatch = confirm !== "" && password !== confirm;
  const ready = longEnough && password === confirm;

  function clear() {
    setPassword("");
    setConfirm("");
  }

  return { password, setPassword, confirm, setConfirm, showMismatch, ready, clear };
}

export type NewPassword = ReturnType<typeof useNewPassword>;
