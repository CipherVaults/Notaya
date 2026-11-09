// Shared bounded transport. CDP errors must never masquerade as successful IPC values.
import assert from 'node:assert/strict';
import { connect, isNavigationError, waitFor } from '../whole-app/cdp.mjs';
const clients = new Set();
export async function connectLegacy(port, match = label => label === 'main') {
  const client = await connect(port, match);
  clients.add(client);
  client.ev = client.evaluate;
  client.waitFor = expression => waitFor(client, expression);
  client.inv = async (command, args = {}) => {
    try { return await client.evaluate(`window.__TAURI_INTERNALS__.invoke(${JSON.stringify(command)}, ${JSON.stringify(args)}).then(v => ({ok: v ?? null}), e => ({err: String(e)}))`); }
    catch (error) {
      // Lock intentionally reloads the WebView. Do not repeat the command; prove
      // the locked state via a fresh read instead of swallowing a protocol error.
      if (command !== 'lock_vault' || !isNavigationError(error)) throw error;
      await waitFor(client, `typeof window.__TAURI_INTERNALS__?.invoke === 'function' && window.__TAURI_INTERNALS__.invoke('vault_status').then(s => s.securityOn && !s.unlocked)`);
      return { ok: null };
    }
  };
  return client;
}
export async function closeLegacyClients() {
  const results = await Promise.allSettled([...clients].map(client => client.close()));
  clients.clear();
  const failed = results.find(result => result.status === 'rejected');
  if (failed) throw failed.reason;
}
export function notesTree(value) {
  assert.ok(value && ['notebooks', 'sections', 'pages', 'databases'].every(key => Array.isArray(value[key])), 'IPC NotesTree must have flat notebooks/sections/pages/databases arrays');
  return value;
}
export async function createNotebookSection(client) {
  const before = notesTree(await client.invoke('list_notes'));
  const ids = new Set(before.notebooks.map(n => n.id));
  // create_notebook returns a whole NotesTree, not a notebook with nested sections.
  const tree = notesTree(await client.invoke('create_notebook', { name: 'T', color: '#888888', icon: 'N' }));
  const added = tree.notebooks.filter(n => !ids.has(n.id));
  assert.equal(added.length, 1, 'one new notebook returned');
  const section = tree.sections.find(s => s.notebookId === added[0].id);
  assert.ok(section?.id, 'new notebook supplied its own section');
  return section;
}
