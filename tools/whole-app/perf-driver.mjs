// Measures actual Rust IPC and WebView UI, using synthetic fixtures only. No direct vault edits.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import crypto from 'node:crypto';
import { connect, waitFor } from './cdp.mjs';
const [port, mode, workload, countText, manifestPath, outputPath] = process.argv.slice(2);
const count = Number(countText);
assert.ok(Number.isInteger(count) && count >= 1 && count <= 50000);
assert.ok(['blank', 'notes', 'long-page', 'database'].includes(workload));
let client;
const result = { mode, workload, count, startedUtc: new Date().toISOString(), timingsMs: {}, limitations: [] };
const token = 'WPERFTOKEN';
const paragraph = text => ({ type: 'paragraph', content: [{ type: 'text', text }] });
const document = text => JSON.stringify({ type: 'doc', content: [paragraph(text)] });
const sha = text => crypto.createHash('sha256').update(text).digest('hex');
async function timed(name, action) { const start = performance.now(); const value = await action(); result.timingsMs[name] = performance.now() - start; return value; }
async function checkpoint(manifest) { fs.writeFileSync(manifestPath, JSON.stringify(manifest, null, 2)); }
try {
  client = await connect(port);
  await timed('readyHome', () => waitFor(client, `Array.from(document.querySelectorAll('button')).some(b=>b.textContent.trim()==='Home')`));
  const status = await timed('vaultStatus', () => client.invoke('vault_status'));
  assert.equal(status.securityOn, false, 'performance fixture is explicitly security-off');
  assert.equal(status.unlocked, true);
  let manifest;
  if (mode === 'seed') {
    manifest = { workload, count, securityMode: 'off', generatedUtc: new Date().toISOString(), pageIds: [], completed: false };
    checkpoint(manifest);
    if (workload !== 'blank') {
      const tree = await client.invoke('create_notebook', { name: 'W performance', color: '#888888', icon: 'W' });
      manifest.notebook = tree.notebooks.find(n => n.name === 'W performance')?.id;
      manifest.section = tree.sections.find(s => s.notebookId === manifest.notebook)?.id;
      assert.ok(manifest.section, 'created notebook supplied a section');
      if (workload === 'notes') {
        await timed('seedNotes', async () => {
          for (let i = 0; i < count; i++) {
            const text = `${token} ${i} \u200e\u2066 exact\tspaces  \u2069 مرحبا שלום`;
            const page = await client.invoke('create_page', { sectionId: manifest.section, title: `W perf note ${i}` });
            await client.invoke('save_page', { id: page.id, title: page.title, content: document(text), text });
            manifest.pageIds.push(page.id);
            if (i === 0) manifest.first = { id: page.id, title: page.title, text, textSha256: sha(text) };
            if (i % 100 === 0) { checkpoint(manifest); console.log(`CHECKPOINT: seeded ${i + 1}/${count} fake notes done`); }
          }
        });
      } else if (workload === 'long-page') {
        const blocks = Array.from({ length: count }, (_, i) => i % 10 === 0
          ? { type: 'codeBlock', content: [{ type: 'text', text: `// ${i}\n\tcode  ` }] }
          : i % 5 === 0 ? { type: 'bulletList', content: [{ type: 'listItem', content: [paragraph(`List ${i}`)] }] }
          : paragraph(`${token} ${i} مرحبا שלום with exact  spaces`));
        const content = JSON.stringify({ type: 'doc', content: blocks });
        const text = blocks.map((_, i) => `${token} ${i}`).join('\n');
        const page = await client.invoke('create_page', { sectionId: manifest.section, title: `W perf long ${count}` });
        await timed('seedLongPage', () => client.invoke('save_page', { id: page.id, title: page.title, content, text }));
        const stored = await client.invoke('get_page', { id: page.id });
        manifest.first = { id: page.id, title: page.title, text, textSha256: sha(text), storedContentSha256: sha(stored.content), blocks: count };
        manifest.pageIds = [page.id];
        assert.equal(JSON.parse(stored.content).content.length, count);
        result.limitations.push('Long-page mix: paragraphs/code/lists/RTL; no pictures/tables/selection/undo/export workload in this slice.');
      } else if (workload === 'database') {
        let db = (await client.invoke('create_database', { sectionId: manifest.section, parent: '', inline: false, title: 'W perf database', starterKind: 'blank' })).database;
        db.props.push({ id: 'wamount', name: 'Amount', kind: 'number' }, { id: 'wdate', name: 'Date', kind: 'date' });
        const prototype = db.views[0]; assert.ok(prototype);
        db.views = ['table', 'board', 'calendar', 'timeline', 'list', 'gallery', 'chart'].map((kind, i) => ({ ...prototype, id: `wview${i}`, name: kind, kind, dateProp: 'wdate', endProp: '', chart: { ...prototype.chart, kind: 'number', yProp: 'wamount', yCalc: 'sum' } }));
        db = await client.invoke('save_database', { databaseJson: JSON.stringify(db) });
        manifest.database = { id: db.id, title: db.title, views: db.views, firstRow: null };
        await timed('seedDatabase', async () => {
          for (let i = 0; i < count; i++) {
            const value = await client.invoke('create_row', { databaseId: db.id, title: `W perf row ${i}`, cells: JSON.stringify({ wamount: i, wdate: { start: `2026-11-${String(i % 28 + 1).padStart(2, '0')}` } }), templateId: '-' });
            if (i === 0) manifest.database.firstRow = value.row.id;
            if (i % 100 === 0) { checkpoint(manifest); console.log(`CHECKPOINT: seeded ${i + 1}/${count} fake rows done`); }
          }
        });
        manifest.database.expectedAmountSum = count * (count - 1) / 2;
      }
    }
    manifest.completed = true; checkpoint(manifest);
  } else if (mode === 'probe') {
    manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
    assert.equal(manifest.completed, true, 'never benchmark an incomplete fixture as the requested size');
    const tree = await timed('listNotes', () => client.invoke('list_notes'));
    if (workload === 'notes') {
      assert.equal(tree.pages.filter(p => manifest.pageIds.includes(p.id)).length, count);
      const hits = await timed('search', () => client.invoke('search', { query: token }));
      result.searchReturned = hits.length;
      assert.ok(hits.length > 0);
      result.limitations.push('Search count may be capped by the app; only nonempty hits and exact first-page readback are asserted.');
    }
    if (manifest.first) {
      const page = await timed('readPage', () => client.invoke('get_page', { id: manifest.first.id }));
      assert.equal(sha(page.text), manifest.first.textSha256);
      if (manifest.first.storedContentSha256) assert.equal(sha(page.content), manifest.first.storedContentSha256);
      for (const key of [`nb:${manifest.notebook}`, `sec:${manifest.section}`]) {
        // Expand via the actual tree affordance; do not call development-only store imports.
        await client.evaluate(`(()=>{const row=document.querySelector('[data-rowkey="'+${JSON.stringify(key)}+'"]');if(row?.getAttribute('aria-expanded')==='false')row.querySelector('span[aria-hidden="true"]')?.click()})()`);
        await new Promise(resolve => setTimeout(resolve, 100));
      }
      const clickStart = performance.now();
      const clicked = await client.evaluate(`(()=>{const el=Array.from(document.querySelectorAll('aside span')).find(e=>e.textContent===${JSON.stringify(manifest.first.title)});el?.click();return !!el})()`);
      assert.ok(clicked, 'fixture page has a real sidebar navigation target');
      await waitFor(client, `document.querySelector('input[placeholder="Untitled"]')?.value===${JSON.stringify(manifest.first.title)} && !!document.querySelector('.ProseMirror[contenteditable="true"]')`, 60000);
      await client.evaluate('new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)))');
      result.timingsMs.sidebarOpenToEditableAndTwoFrames = performance.now() - clickStart;
    }
    if (manifest.database) {
      const db = manifest.database;
      for (const view of db.views) {
        const queried = await timed(`query-${view.kind}`, () => client.invoke('query_view', { request: { databaseId: db.id, viewId: view.id } }));
        assert.equal(queried.total, count);
        result[`query-${view.kind}`] = { total: queried.total, filtered: queried.filtered, returned: queried.rows.length, truncated: queried.truncated };
      }
      result.limitations.push('All seven backend view queries measured; per-view UI render/scroll/formula/relation/rollup edit timing remains unmeasured.');
      const row = (await client.invoke('get_row', { rowId: db.firstRow })).row;
      assert.equal(row.values.wamount, 0);
    }
    result.fixtureVerified = true;
  } else throw Error('Expected seed or probe');
  result.status = 'passed';
} catch (error) { result.status = 'failed'; result.error = error.message; process.exitCode = 1; }
finally {
  try { if (client) await client.close(); }
  catch (error) { result.status = 'failed'; result.closeError = error.message; process.exitCode = 1; }
  result.finishedUtc = new Date().toISOString();
  result.totalMs = Date.parse(result.finishedUtc) - Date.parse(result.startedUtc);
  fs.writeFileSync(outputPath, JSON.stringify(result, null, 2));
}
