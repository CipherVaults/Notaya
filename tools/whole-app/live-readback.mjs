import assert from 'node:assert/strict';
import { connect } from './cdp.mjs';
const [port, mode, scenario] = process.argv.slice(2);
const client = await connect(port);
try {
  const status = await client.invoke('vault_status');
  if (mode === 'lockflow' || mode === 'restore-flow') {
    assert.equal(status.securityOn, true, 'fixture keeps security enabled across restart');
    assert.equal(status.unlocked, false, 'protected restart must be locked');
    await client.invoke('unlock_vault', { password: process.env.NOTAYA_TEST_PASSWORD, keyfile: null });
  } else { assert.equal(status.unlocked, true, 'open restart must be unlocked'); }
  const tree = await client.invoke('list_notes');
  if (mode === 'optional-security') {
    const page = tree.pages.find(p => p.title === 'Open note'); assert.ok(page);
    assert.equal((await client.invoke('get_page', { id: page.id })).text, 'OPEN-MODE-NOTE-7731');
    assert.equal(status.securityOn, false);
  } else if (mode === 'restore-flow') {
    // Match the distinctive stored text without depending on the historical fixture title.
    let found = false;
    for (const page of tree.pages) if ((await client.invoke('get_page', { id: page.id })).text === 'CHANGED-AFTER-BACKUP') found = true;
    assert.ok(found, 'restored text persisted after normal quit/reopen');
  } else if (mode === 'lockflow') {
    if (scenario === 'sticky') {
      const notes = await client.invoke('list_stickies'); let found = false;
      for (const note of notes) { const value = await client.invoke('get_sticky', { id: note.id }); if ((value.text + value.content).includes(`FLUSHTEST-${scenario}`)) found = true; }
      assert.ok(found, 'sticky pending text persists after quit/reopen');
    } else {
      const page = tree.pages.find(p => p.title === 'Flush page'); assert.ok(page);
      const value = await client.invoke('get_page', { id: page.id });
      assert.ok((value.text + value.content).includes(`FLUSHTEST-${scenario}`), 'page pending text persists after quit/reopen');
    }
  } else throw Error('Unknown readback mode');
  console.log('PASS actual quit/reopen persisted the expected synthetic content');
} finally { client.close(); }
