// Dev-only display fixtures. Never included in the production IPC path.
import type { ActivityEntry, HistoryBody, HistoryMeta } from "../src/lib/ipc";
type Page = HistoryBody & { id: string };
const versions = new Map<string, { meta: HistoryMeta; body: HistoryBody }[]>();
let serial = 0;
const activity: ActivityEntry[] = [
  { seq: 2, at: Date.now() - 30_000, actor: "ai:Test AI", changeSet: "mock-ai-change", action: "items.changed", items: [{ id: "p1", kind: "page", titleAtTheTime: "Weekly plan" }], counts: { changed: 1 }, undoable: true },
  { seq: 1, at: Date.now() - 60_000, actor: "person", changeSet: "mock-person-change", action: "items.changed", items: [{ id: "n1", kind: "notebook", titleAtTheTime: "Personal" }], counts: { created: 1 }, undoable: false },
];
function snapshot(page: Page, actor = "person", at = Date.now()) {
  const body = { title: page.title, content: page.content, text: page.text, icon: page.icon, cover: page.cover };
  const list = versions.get(page.id) ?? [];
  if (list[list.length - 1] && JSON.stringify(list[list.length - 1]?.body) === JSON.stringify(body)) return;
  const meta: HistoryMeta = { pageId: page.id, versionId: `mock-version-${++serial}`, at, actor, changeSet: "mock-ai-change", title: body.title, words: body.text.split(/\s+/).filter(Boolean).length, pictureIds: [], beforeAi: actor.startsWith("ai:") };
  list.push({ meta, body }); versions.set(page.id, list);
}
export async function journalCommand(cmd: string, args: Record<string, unknown>, read: (id: string) => Promise<Page>, write: (id: string, body: HistoryBody) => Promise<void>) {
  const id = String(args.pageId ?? "p1");
  if (cmd.startsWith("history_")) {
    const current = await read(id);
    if (!versions.has(id)) {
      snapshot({ ...current, title: "Earlier saved title", text: "Earlier saved text", content: '{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"Earlier saved text"}]}]}' }, "ai:Test AI", Date.now() - 3_600_000);
      snapshot(current);
    }
    if (cmd === "history_snapshot") { snapshot(current); return null; }
    if (cmd === "history_list") return versions.get(id)!.map((version) => version.meta).slice().reverse();
    const version = versions.get(id)!.find((version) => version.meta.versionId === args.versionId);
    if (!version) throw "That version no longer exists.";
    if (cmd === "history_get") return version.body;
    if (cmd === "history_restore") { snapshot(current); await write(id, version.body); return null; }
  }
  if (cmd === "activity_list") return activity.filter((entry) => (!args.who || entry.actor === args.who) && (!args.before || entry.seq < Number(args.before))).slice(0, Number(args.limit ?? 100));
  if (cmd === "activity_verify") return null;
  if (cmd === "activity_clear") {
    if (args.confirmation !== "CLEAR") throw "Type CLEAR to clear the activity log.";
    activity.splice(0, activity.length, { seq: 1, at: Date.now(), actor: "person", changeSet: "mock-cleared", action: "log.cleared", items: [], counts: {}, undoable: false }); return null;
  }
  if (cmd === "activity_undo") {
    const entry = activity.find((entry) => entry.changeSet === args.changeSet && entry.undoable);
    if (!entry) throw "That change cannot be undone.";
    const current = await read("p1");
    snapshot(current);
    await write("p1", { ...current, title: "Earlier saved title", text: "Earlier saved text", content: '{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"Earlier saved text"}]}]}' });
    entry.undoable = false;
    return { restored: 1, skipped: 0 };
  }
  throw "Unknown journal command.";
}
