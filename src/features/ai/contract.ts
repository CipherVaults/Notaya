import type { AiActivityEvent, AiConfirmEvent, AiConfirmRequest, AiPairingEvent, AiPairingRequest, AiState, AiStatus } from "../../lib/ipc";

export function pairingRequest(event: AiPairingEvent, receivedAt = Date.now()): AiPairingRequest {
  return { id: event.id, name: event.details.client, expiresAt: receivedAt + 300_000 };
}

export function confirmRequest(event: AiConfirmEvent, receivedAt = Date.now()): AiConfirmRequest {
  const actions: Record<string, string> = {
    "page.trash": "move pages to the bin.", "page.delete": "permanently delete pages.",
    "notebook.trash": "move a notebook and its contents to the bin.",
    "section.trash": "move a section and its contents to the bin.",
    "bin.empty": "empty the bin.", "history.restore": "restore a page version.",
    "activity.undo": "undo earlier changes.", "page.write": "change pages.",
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
