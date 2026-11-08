// Fake window commands and event triggers, never included in the production build.
// C5 owns the real authorization, token handling, logging and transaction behavior.
import type { AiConfirmRequest, AiLevel, AiState } from "../src/lib/ipc";
import { emit } from "./event.ts";

const initial = (): AiState => ({
  enabled: false, paused: false, pairings: [], hiddenNotebooks: [],
  bigChangeConfirm: true, bigChangeItems: 20, pairingRequests: [], confirmRequests: [], activity: [],
  connectionHelp: {
    terminal: '"D:\\Notaya Tests\\v1-6-E\\notaya-bridge.exe"',
    config: '[mcp_servers.notaya]\ncommand = "D:\\\\Notaya Tests\\\\v1-6-E\\\\notaya-bridge.exe"',
  },
});
let state = initial();
let sequence = 0;
let locked = false;
let failNext: string | null = null;
const calls: { command: string; args: Record<string, unknown> }[] = [];
const snapshot = () => structuredClone(state);
const changed = () => { void emit("ai-state-changed"); };
const cancel = () => { state.pairingRequests = []; state.confirmRequests = []; state.activity = []; };

export function aiLock(value: boolean) {
  locked = value;
  if (locked) cancel();
  changed();
}

export function aiCommand(command: string, args: Record<string, unknown>): unknown {
  calls.push({ command, args: structuredClone(args) });
  if (failNext === command) { failNext = null; throw "invalid"; }
  if (locked) throw "locked";
  if (command === "ai_get_state") return snapshot();
  switch (command) {
    case "ai_set_enabled":
      if (args.enabled && args.confirmation !== "ALLOW") throw "not_confirmed";
      state.enabled = args.enabled === true;
      if (!state.enabled) { state.paused = false; cancel(); }
      break;
    case "ai_set_level": {
      const pairing = state.pairings.find((p) => p.id === args.id);
      if (!pairing || !["look", "edit", "full"].includes(String(args.level))) throw "invalid";
      pairing.level = args.level as AiLevel;
      break;
    }
    case "ai_remove_pairing": {
      const name = state.pairings.find((p) => p.id === args.id)?.name;
      state.pairings = state.pairings.filter((p) => p.id !== args.id);
      state.activity = state.activity.filter((p) => p.pairingId !== args.id);
      state.confirmRequests = state.confirmRequests.filter((p) => p.pairingName !== name);
      break;
    }
    case "ai_set_hidden":
      if (!Array.isArray(args.notebookIds) || !args.notebookIds.every((v) => ["n1", "n2"].includes(v))) throw "invalid";
      state.hiddenNotebooks = [...new Set(args.notebookIds as string[])];
      break;
    case "ai_set_big_change": state.bigChangeConfirm = args.enabled === true; break;
    case "ai_pairing_answer": {
      const request = state.pairingRequests.find((r) => r.id === args.id);
      if (!request) throw "not_confirmed";
      if (args.allow) {
        if (!state.enabled || state.paused || Date.now() >= request.expiresAt) throw "not_confirmed";
        state.pairings.push({ id: request.id, name: request.name, level: "look" });
      }
      state.pairingRequests = state.pairingRequests.filter((r) => r.id !== request.id);
      break;
    }
    case "ai_confirm_answer": {
      const request = state.confirmRequests.find((r) => r.id === args.id);
      if (!request) throw "not_confirmed";
      if (args.allow && (!state.enabled || state.paused || Date.now() >= request.expiresAt || (request.kind === "strong" && args.word !== request.word))) throw "not_confirmed";
      state.confirmRequests = state.confirmRequests.filter((r) => r.id !== request.id);
      break;
    }
    case "ai_pause": state.paused = true; cancel(); break;
    case "ai_resume": state.paused = false; break;
    default: throw "invalid";
  }
  changed();
  return null;
}

export const aiMock = {
  state: snapshot,
  calls: () => structuredClone(calls),
  reset() { state = initial(); locked = false; calls.length = 0; failNext = null; changed(); },
  fail(command: string) { failNext = command; },
  pair(name = "Writing assistant", expiresAt = Date.now() + 300_000) {
    if (!state.enabled || state.paused || locked) throw new Error("off");
    const id = `pair-${++sequence}`;
    state.pairingRequests.push({ id, name, expiresAt });
    void emit("ai-pairing-request");
    return id;
  },
  confirm(kind: "ask" | "strong", overrides: Partial<AiConfirmRequest> = {}) {
    if (!state.enabled || state.paused || locked) throw new Error("off");
    const id = `confirm-${++sequence}`;
    state.confirmRequests.push({ id, pairingName: "Writing assistant", summary: "move 3 pages to the bin. Allow?",
      items: ["Plan", "Budget", "Notes"], kind, word: kind === "strong" ? "MAPLE" : null, expiresAt: Date.now() + 300_000, ...overrides });
    void emit("ai-confirm-request");
    return id;
  },
  activity(pairingId: string, claims = ["Plan"], lastActiveAt = Date.now()) {
    const pairing = state.pairings.find((p) => p.id === pairingId);
    if (!pairing || !state.enabled || state.paused || locked) throw new Error("not_paired");
    state.activity = state.activity.filter((a) => a.pairingId !== pairingId);
    state.activity.push({ pairingId, name: pairing.name, lastActiveAt, claims });
    void emit("ai-activity");
  },
  lock: aiLock,
};
