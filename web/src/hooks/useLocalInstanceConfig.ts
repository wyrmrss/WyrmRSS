import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { getLocalInstanceConfig, updateLocalInstance } from "../api/instance";
import type { UpdateLocalInstance } from "../types/UpdateLocalInstance";

const localInstanceConfigKey = ["local-instance-config"];

/** Instance-wide settings. Admins only. */
export function useLocalInstanceConfig() {
  return useQuery({
    queryKey: localInstanceConfigKey,
    queryFn: getLocalInstanceConfig,
  });
}

export function useUpdateLocalInstance() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body: UpdateLocalInstance) => updateLocalInstance(body),
    onSuccess: (data) => qc.setQueryData(localInstanceConfigKey, data),
  });
}
