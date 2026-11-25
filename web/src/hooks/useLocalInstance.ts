import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { getLocalInstance, updateLocalInstance } from "../api/instance";
import type { UpdateLocalInstance } from "../types/UpdateLocalInstance";

const localInstanceKey = ["local-instance"];

/** Instance-wide settings, readable before sign-in. Some fields are hidden until signed in. */
export function useLocalInstance() {
  return useQuery({
    queryKey: localInstanceKey,
    queryFn: getLocalInstance,
    // SetupGate waits on this, so an unreachable server shows the app at once instead of after retries.
    retry: false,
    // Once failed, mounting another reader doesn't refetch, or SetupGate would unmount the app and loop.
    retryOnMount: false,
  });
}

/** Admins only. */
export function useUpdateLocalInstance() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body: UpdateLocalInstance) => updateLocalInstance(body),
    onSuccess: (data) => qc.setQueryData(localInstanceKey, data),
  });
}
