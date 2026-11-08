import { api, errorText } from "@/lib/ipc";

type ConfirmRestore = (options: { title: string; body: string; confirm: string }) => Promise<boolean>;

/** All restore/Undo surfaces try the old individual restore first. Only a
 * binned notebook/section needs the existing grouped-restore confirmation.
 * Ask the core at click time: loaded bin metadata may already be stale.
 */
export async function restoreFromBin(id: string, confirm: ConfirmRestore) {
  try {
    return await api.restoreBinItem(id, false);
  } catch (error) {
    if (!errorText(error).startsWith("parent_in_bin:")) throw error;
    if (!await confirm({ title: "Restore containing items too?", body: "This item's notebook or section is also in the bin. Restore them together?", confirm: "Restore together" })) return null;
    return api.restoreBinItem(id, true);
  }
}
