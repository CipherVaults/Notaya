// These entry-point tests may not connect to an app, even on loopback.
globalThis.fetch = () => { throw new Error('NETWORK_FORBIDDEN_IN_GRADER_QUALIFICATION'); };
globalThis.WebSocket = class {
  constructor() { throw new Error('NETWORK_FORBIDDEN_IN_GRADER_QUALIFICATION'); }
};
