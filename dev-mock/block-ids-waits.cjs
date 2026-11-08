const { setTimeout: delay } = require('node:timers/promises');

// Only readiness probes and idempotent fixture setup belong here. Retrying an
// edit, Undo or Redo could apply it twice after a lost evaluation response.
async function waitForMockState(page, label, probe, arg, { timeout = 30000, polling = 100 } = {}) {
  const deadline = Date.now() + timeout;
  let lastNavigation;
  const expired = () => new Error(`Timed out after ${timeout} ms waiting for ${label}${lastNavigation ? `; last navigation: ${lastNavigation.message}` : ''}`);
  while (Date.now() < deadline) {
    let timer;
    try {
      const value = await Promise.race([
        page.evaluate(probe, arg),
        new Promise((_, reject) => { timer = setTimeout(() => reject(expired()), Math.max(1, deadline - Date.now())); }),
      ]);
      if (value) return value;
    } catch (error) {
      if (!/Execution context was destroyed|Cannot find context with (?:specified )?id/i.test(error.message)) throw error;
      lastNavigation = error;
    } finally {
      clearTimeout(timer);
    }
    await delay(Math.max(0, Math.min(polling, deadline - Date.now())));
  }
  throw expired();
}

// Serialized by Playwright into the page; no closure or Node dependencies.
function installEquationFixture({ bid, previousContent }) {
  const element = document.querySelector('.ProseMirror');
  const editor = element?.editor;
  if (document.readyState !== 'complete' || !element?.isConnected || !editor?.isEditable) return false;
  const doc = editor.getJSON();
  // Safe to poll if navigation lost the acknowledgement of our dispatch.
  if (doc.content.length === 2 && doc.content[0].type === 'mathBlock'
    && doc.content[0].attrs.bid === bid && doc.content[0].attrs.latex === 'x^2'
    && doc.content[1].type === 'paragraph' && !doc.content[1].content?.length) return true;
  // Do not silently accept a reload which lost the preceding edit or ids.
  if (JSON.stringify(doc) !== previousContent || !window.__blockIdsModules) return false;
  const { equationFixtureTransaction } = window.__blockIdsModules;
  // No import/await between acquiring this editor and its synchronous use.
  // The isolated transaction keeps setup out of the edit's Undo event.
  editor.view.dispatch(equationFixtureTransaction(editor.state, bid));
  editor.commands.blur();
  return true;
}

module.exports = { waitForMockState, installEquationFixture };
