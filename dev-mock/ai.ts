// Fake window commands and event triggers, never included in the production build.
// C5 owns the real authorization, token handling, logging and transaction behavior.
import type { AiConfirmEvent, AiConfirmRequest, AiLevel, AiState, AiStatus } from "../src/lib/ipc";
import { confirmRequest, pairingRequest } from "../src/features/ai/contract.ts";
import { emit } from "./event.ts";

const initial = (): AiState => ({
  enabled: false, paused: false, pairings: [], hiddenNotebooks: [],
  bigChangeConfirm: true, bigChangeItems: 20, pairingRequests: [], confirmRequests: [], activity: [],
});
let state = initial();
let sequence = 0;
let locked = false;
let failNext: { command: string; code: string } | null = null;
const calls: { command: string; args: Record<string, unknown> }[] = [];
// Pipe-only failures are test evidence, never invented window events/status fields.
const bridgeErrors: { code: string; message: string }[] = [];
const snapshot = () => structuredClone(state);
const changed = () => { void emit("items-changed", { ids: ["settings"], kinds: ["settings"], tree: false, by: "person", revs: {} }); };
const cancel = () => { state.pairingRequests = []; state.confirmRequests = []; state.activity = []; };

export function aiLock(value: boolean) {
  locked = value;
  if (locked) { cancel(); void emit("vault-locked", "manual"); }
  changed();
}

export function aiCommand(command: string, args: Record<string, unknown>): unknown {
  calls.push({ command, args: structuredClone(args) });
  if (failNext?.command === command) { const { code } = failNext; failNext = null; throw code; }
  if (locked) throw "The vault is locked.";
  if (command === "ai_get_status") {
    // Exactly C5's window response: requests and activity arrive only as events.
    const { enabled, paused, pairings, hiddenNotebooks, bigChangeConfirm, bigChangeItems } = snapshot();
    return { enabled, paused, pairings, hiddenNotebooks, bigChangeConfirm, bigChangeItems } satisfies AiStatus;
  }
  switch (command) {
    case "ai_set_enabled":
      if (args.enabled && args.confirmation !== "ALLOW") throw "not_confirmed";
      state.enabled = args.enabled === true;
      cancel();
      break;
    case "ai_set_level": {
      const pairing = state.pairings.find((p) => p.id === args.pairingId);
      if (!["look", "edit", "full"].includes(String(args.level))) throw "invalid";
      if (!pairing) throw "That item no longer exists.";
      pairing.level = args.level as AiLevel;
      state.confirmRequests = [];
      break;
    }
    case "ai_remove_pairing": {
      state.pairings = state.pairings.filter((p) => p.id !== args.pairingId);
      cancel();
      break;
    }
    case "ai_set_hidden":
      if (!Array.isArray(args.notebookIds) || args.notebookIds.length > 10_000 || !args.notebookIds.every((v) => ["n1", "n2"].includes(v))) throw "invalid";
      state.hiddenNotebooks = [...args.notebookIds as string[]];
      cancel();
      break;
    case "ai_set_big_change":
      if (args.items != null && (!Number.isInteger(args.items) || Number(args.items) < 1 || Number(args.items) > 500)) throw "invalid";
      state.bigChangeConfirm = args.enabled === true;
      if (args.items != null) state.bigChangeItems = Number(args.items);
      cancel();
      break;
    case "ai_pair_answer": {
      const request = state.pairingRequests.find((r) => r.id === args.id);
      if (!request) throw "pairing_refused";
      if (args.allow) {
        if (!state.enabled || state.paused || Date.now() >= request.expiresAt) throw "pairing_refused";
      }
      state.pairingRequests = state.pairingRequests.filter((r) => r.id !== request.id);
      if (args.allow) {
        // The window command acknowledges the broker answer. Persistence happens
        // after that answer; duplicate failure reaches only the bridge in C5d.
        if (state.pairings.length >= 32 || state.pairings.some((p) => p.name === request.name)) {
          bridgeErrors.push({ code: "pairing_refused", message: "pairing_refused" });
        } else {
          state.pairings.push({ id: `paired-${++sequence}`, name: request.name, level: "look", created: Date.now(), lastUsed: null });
          changed();
        }
      }
      return null;
    }
    case "ai_confirm_answer": {
      const request = state.confirmRequests.find((r) => r.id === args.id);
      if (!request) throw "not_confirmed";
      if (args.allow && (!state.enabled || state.paused || Date.now() >= request.expiresAt || (request.kind === "strong" && args.word !== request.word))) throw "not_confirmed";
      state.confirmRequests = state.confirmRequests.filter((r) => r.id !== request.id);
      return null;
    }
    case "ai_pause": state.paused = true; cancel(); break;
    case "ai_resume": state.paused = false; cancel(); break;
    default: throw "invalid";
  }
  changed();
  return null;
}

export const aiMock = {
  state: snapshot,
  calls: () => structuredClone(calls),
  bridgeErrors: () => structuredClone(bridgeErrors),
  reset() { state = initial(); locked = false; calls.length = 0; bridgeErrors.length = 0; failNext = null; changed(); },
  fail(command: string, code = "invalid") { failNext = { command, code }; },
  pair(name = "Writing assistant", expiresAt = Date.now() + 300_000) {
    if (!state.enabled || state.paused || locked) throw new Error("off");
    if (state.pairingRequests.length) throw new Error("pairing_refused");
    const id = `pair-${++sequence}`;
    const event = { id, kind: "pairing" as const, details: { client: name }, word: null };
    state.pairingRequests.push({ ...pairingRequest(event), expiresAt });
    void emit("ai-pairing", event);
    return id;
  },
  confirm(kind: "ask" | "strong", overrides: Partial<AiConfirmRequest> = {}) {
    if (!state.enabled || state.paused || locked) throw new Error("off");
    const id = `confirm-${++sequence}`;
    const event: AiConfirmEvent = { id, kind: "confirm", details: { client: overrides.pairingName ?? "Writing assistant", action: "page.trash",
      items: (overrides.items ?? ["Plan", "Budget", "Notes"]).map((title, i) => ({ id: `p${i + 1}`, title })) }, word: overrides.word ?? (kind === "strong" ? "MAPLE-CEDAR" : null) };
    state.confirmRequests.push({ ...confirmRequest(event), ...overrides });
    void emit("ai-confirm", event);
    return id;
  },
  activity(pairingId: string, claims = ["Plan"], lastActiveAt = Date.now()) {
    const pairing = state.pairings.find((p) => p.id === pairingId);
    if (!pairing || !state.enabled || state.paused || locked) throw new Error("not_paired");
    state.activity = state.activity.filter((a) => a.pairingId !== pairingId);
    state.activity.push({ pairingId, name: pairing.name, lastActiveAt, claims });
    void emit("ai-activity", { pairingId, name: pairing.name, at: lastActiveAt, items: claims });
  },
  lock: aiLock,
};
