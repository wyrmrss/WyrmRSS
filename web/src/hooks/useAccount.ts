import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { deleteAccount, getAccount, updatePassword } from "../api/account";
import type { DeleteAccount } from "../types/DeleteAccount";
import type { UpdatePassword } from "../types/UpdatePassword";

export const accountKey = ["account"];

export function useAccount() {
  return useQuery({
    queryKey: accountKey,
    queryFn: getAccount,
  });
}

export function useUpdatePassword() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body: UpdatePassword) => updatePassword(body),
    // Setting a first password flips `has_password`.
    onSuccess: () => qc.invalidateQueries({ queryKey: accountKey }),
  });
}

export function useDeleteAccount() {
  return useMutation({
    mutationFn: (body: DeleteAccount) => deleteAccount(body),
  });
}
