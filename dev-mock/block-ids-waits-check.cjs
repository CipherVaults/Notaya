// Browser-less regression for the actual readiness/reload seam used by the UI
// check. No Playwright, app, server, vault or additional dependency is needed.
const assert = require('node:assert/strict');
const vm = require('node:vm');
const { waitForMockState, installEquationFixture } = require('./block-ids-waits.cjs');

(async () => {
  let calls = 0;
  const page = { async evaluate(probe, arg) {
    calls++;
    if (calls === 1) throw new Error('locator.evaluate: Execution context was destroyed, most likely because of a navigation.');
    // The new document exists but its specific editor is not ready yet.
    return probe({ ...arg, editorReady: calls >= 3 });
  } };
  assert.deepEqual(await waitForMockState(page, 'the replacement editor', (state) => state.editorReady && state.ids,
    { ids: ['eeeeeeeeeeeeeeee'] }, { timeout: 1000, polling: 1 }), ['eeeeeeeeeeeeeeee']);
  assert.equal(calls, 3);
  console.log('PASS navigation_retries_until_the_specific_editor_is_ready');

  calls = 0;
  let dispatches = 0;
  const previous = { content: [{ type: 'paragraph', content: [{ type: 'text', text: 'block id edit' }] }] };
  const fixture = { content: [{ type: 'mathBlock', attrs: { bid: 'eeeeeeeeeeeeeeee', latex: 'x^2' } }, { type: 'paragraph' }] };
  let current = previous;
  const element = { isConnected: true, editor: { isEditable: true, state: {},
    getJSON: () => current, commands: { blur() {} }, view: { dispatch(tr) { dispatches++; current = tr; } },
  } };
  const sandbox = { document: { readyState: 'complete', querySelector: () => element },
    window: { __blockIdsModules: { equationFixtureTransaction: () => fixture } },
    arg: { bid: 'eeeeeeeeeeeeeeee', previousContent: JSON.stringify(previous) },
  };
  const install = () => vm.runInNewContext(`(${installEquationFixture.toString()})(arg)`, sandbox);
  element.isConnected = false;
  assert.equal(install(), false, 'Wait for the current editor to attach');
  element.isConnected = true;
  current = { content: [] };
  assert.equal(install(), false, 'Never replace a page that lost the preceding saved edit');
  assert.equal(dispatches, 0);
  current = previous;
  const fixturePage = { async evaluate() {
    calls++;
    const installed = install();
    if (calls === 1) throw new Error('Cannot find context with specified id');
    return installed;
  } };
  await waitForMockState(fixturePage, 'the acknowledged fixture', () => true, null, { timeout: 1000, polling: 1 });
  assert.equal(dispatches, 1, 'Retrying an idempotent fixture must not dispatch twice');
  assert.equal(calls, 2);
  console.log('PASS navigation_after_fixture_dispatch_can_be_acknowledged_without_replay');

  const defect = new Error('Equation fixture has a real defect');
  calls = 0;
  await assert.rejects(waitForMockState({ async evaluate() { calls++; throw defect; } }, 'the fixture', () => true), (e) => e === defect);
  assert.equal(calls, 1, 'Real JavaScript errors must fail immediately');
  console.log('PASS readiness_wait_does_not_swallow_real_errors');

  for (const evaluate of [async () => false, () => new Promise(() => {}), async () => { throw new Error('Execution context was destroyed'); }]) {
    await assert.rejects(waitForMockState({ evaluate }, 'the fixture equation with its stable bid', () => false, null,
      { timeout: 30, polling: 1 }), /Timed out after 30 ms waiting for the fixture equation with its stable bid/);
  }
  console.log('PASS missing_state_stalled_import_and_repeated_navigation_have_named_timeouts');
  console.log('ALL OK (4 checks)');
})().catch((error) => { console.error(error); process.exitCode = 1; });
