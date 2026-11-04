// Reminders and the messages of automations are shown inside Notaya as small notices (title only).

import { toast } from "sonner";
import { api } from "@/lib/ipc";
import type { Reminder } from "@/lib/database-types";
import { useApp } from "@/store/app";
import { rowsChanged } from "./dbStore";

export function showReminders(list: Reminder[]) {
  for (const r of list) {
    const fromAutomation = !!r.message;
    toast(r.title || "Untitled", {
      description: fromAutomation ? `${r.message}${r.databaseTitle ? ` · ${r.databaseTitle}` : ""}` : r.databaseTitle ? `Reminder · ${r.databaseTitle}` : "Reminder",
      duration: 20000,
      action: r.rowId ? { label: "Open", onClick: () => useApp.getState().openRow(r.rowId) } : undefined,
    });
  }
}

/** Asks Notaya for the notices that are waiting (date reminders, date automations, messages from automations). */
export function pollNotices(): Promise<void> {
  return api
    .dueReminders()
    .then(showReminders)
    .catch(() => undefined);
}

/** After an edit or a button press: a message from an automation may be waiting, and other rows may have changed. */
export function afterRowChange(automationsMayHaveRun: boolean) {
  if (!automationsMayHaveRun) return;
  void pollNotices().then(() => rowsChanged());
}
