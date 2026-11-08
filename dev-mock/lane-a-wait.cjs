// Await each mock-settings read: Playwright 1.65's waitForFunction treats a Promise
// as truthy before its value resolves, so an async predicate can finish too early.
const assert = require('node:assert/strict');
const { isDeepStrictEqual } = require('node:util');
const { setTimeout: pause } = require('node:timers/promises');

async function waitForSavedPlace(readPlace, expected) {
  const deadline = Date.now() + 5000;
  let actual;
  do {
    actual = await readPlace();
    if (isDeepStrictEqual(actual, expected)) return actual;
    await pause(50);
  } while (Date.now() < deadline);
  assert.deepEqual(actual, expected, 'The chosen database location must be saved within 5 seconds');
}

module.exports = { waitForSavedPlace };
