import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { getSettings, importOpml, updateSettings } from "../api/settings";
import { feedKeys } from "../cache/feeds";
import { postKeys } from "../cache/posts";
import { folderKeys } from "./useFolders";
import { useLocalInstance } from "./useLocalInstance";
import type { UpdateSettings } from "../types/UpdateSettings";

export function useSettings() {
  return useQuery({
    queryKey: ["settings"],
    queryFn: getSettings,
    staleTime: Infinity,
  });
}

/** True when the server runs as the public demo and rejects risky writes. */
export function useDemoMode() {
  return useSettings().data?.run_as_demo ?? false;
}

/** Name of the integration this feed depends on while it's off (the worker skips it), else null. */
export function useIntegrationPaused(url: string): string | null {
  const instance = useLocalInstance().data;
  if (url.startsWith("rsshub://") && !instance?.rsshub_instance) return "RSSHub";
  return null;
}

export function useUpdateSettings() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body: Partial<UpdateSettings>) => updateSettings(body),
    onSuccess: (data) => {
      qc.setQueryData(["settings"], data);
    },
  });
}

export function useImportOpml() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (file: File) => importOpml(file),
    // The server polls the imported feeds and waits up to the HTTP timeout
    // before replying, so posts usually land with these refetches.
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: feedKeys.all });
      qc.invalidateQueries({ queryKey: postKeys.all });
      // Import resolves-or-creates folders from OPML categories.
      qc.invalidateQueries({ queryKey: folderKeys.all });
    }
  });
}

