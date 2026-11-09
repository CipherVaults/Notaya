import type { AiActivityEvent, AiConfirmEvent, AiConfirmRequest, AiPairingEvent, AiPairingRequest, AiState, AiStatus } from "../../lib/ipc";

export function pairingRequest(event: AiPairingEvent, receivedAt = Date.now()): AiPairingRequest {
  return { id: event.id, name: event.details.client, expiresAt: receivedAt + 300_000 };
}

export function confirmRequest(event: AiConfirmEvent, receivedAt = Date.now()): AiConfirmRequest {
  const actions: Record<string, string> = {
    "page.trash": "move pages to the bin.", "bin.delete_forever": "permanently delete items from the bin.",
    "notebook.trash": "move a notebook and its contents to the bin.",
    "section.trash": "move a section and its contents to the bin.",
    "bin.empty": "empty the bin.", "history.restore": "restore a page version.",
    "activity.undo": "undo earlier changes.", "page.write": "change pages.",
    "page.append": "add content to a page.", "page.insert": "insert content into a page.",
    "page.replace_block": "replace a block on a page.", "page.delete_blocks": "remove blocks from a page.",
    "page.move_blocks": "move blocks between pages.", "sticky.trash": "move a sticky note to the bin.",
    "db.trash": "move a database and its rows to the bin.", "db.row.trash": "move a database row to the bin.",
    "db.property.change_kind": "change a database property's type.", "db.property.trash": "remove a database property and its values.",
    "db.view.trash": "remove a database view.", "db.row.create": "create database rows.",
    "db.row.set": "change database row values.", "db.row.write_page": "change a database row's page.",
    "db.row.duplicate": "duplicate database rows.", "db.row.reorder": "reorder database rows.",
    "db.template.trash": "remove a database template.", "db.automation.list": "list database automations.",
    "db.automation.create": "create a database automation.", "db.automation.update": "change a database automation.",
    "db.automation.pause": "change whether a database automation is paused.", "db.automation.trash": "remove a database automation.",
    "db.button.press": "run a database button's actions.", "db.file.remove": "remove a file from a database row.",
    "calc.clear": "clear the calculator history.",
    "ui.wait_for_person": "wait for your confirmation before continuing.",
  };
  return {
    id: event.id, pairingName: event.details.client,
    summary: actions[event.details.action] ?? "make this change.",
    items: event.details.items.map((item) => item.title || item.id),
    kind: event.word === null ? "ask" : "strong", word: event.word,
    expiresAt: receivedAt + 300_000,
  };
}

/** Fixed AI error copy: never echo a client label, request body or backend message. */
export function aiErrorText(error: unknown, fallback = "The AI access change could not be saved. Try again."): string {
  const code = typeof error === "string" ? error
    : error instanceof Error ? error.message
    : error && typeof error === "object" && "code" in error ? error.code : null;
  if (code === "rate_limited") return "Too many AI requests are waiting. Wait a moment, then try again.";
  if (code === "pairing_refused") return "The connection could not be paired. Check existing pairings in Settings and try again.";
  if (code === "The vault is locked." || code === "locked") return "Unlock Notaya and try again.";
  return fallback;
}

export function activityUpdate<T extends Pick<AiState, "activity">>(state: T, event: AiActivityEvent): T {
  return { ...state, activity: [...state.activity.filter((a) => a.pairingId !== event.pairingId), {
    pairingId: event.pairingId, name: event.name, lastActiveAt: event.at, claims: event.items,
  }] };
}

export function withStatus(previous: AiState | null, status: AiStatus): AiState {
  // Adding a pairing does not invalidate other pending pairings. Permission/rule
  // changes do cancel requests in C5; a level change cancels confirmations only.
  // On/Resume may be observed after a new request's event. Off/Stop clear below;
  // reopening must retain those newly received requests.
  const invalidated = previous && (previous.bigChangeConfirm !== status.bigChangeConfirm || previous.bigChangeItems !== status.bigChangeItems
    || JSON.stringify(previous.hiddenNotebooks) !== JSON.stringify(status.hiddenNotebooks)
    || previous.pairings.some((p) => !status.pairings.some((next) => next.id === p.id)));
  const levelChanged = previous?.pairings.some((p) => status.pairings.some((next) => next.id === p.id && next.level !== p.level));
  const clear = !status.enabled || status.paused || invalidated;
  return { ...status,
    pairingRequests: clear ? [] : previous?.pairingRequests ?? [],
    confirmRequests: clear || levelChanged ? [] : previous?.confirmRequests ?? [],
    activity: clear ? [] : previous?.activity.filter((a) => status.pairings.some((p) => p.id === a.pairingId)) ?? [],
  };
}
