// PC runner only. Separate optimiser caches prevent mock/views invalidating one another.
import assert from 'node:assert/strict';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createServer, mergeConfig } from 'vite';
import mockConfig from '../../vite.mock.config.ts';
import viewsConfig from '../../vite.views.config.ts';
const [mode, portText] = process.argv.slice(2);
assert.ok(['mock', 'views'].includes(mode));
const port = Number(portText);
assert.ok(Number.isInteger(port) && port > 0 && port < 65536);
assert.ok(process.env.TEMP, 'runner must supply disposable TEMP');
const root = fileURLToPath(new URL('../..', import.meta.url));
const server = await createServer(mergeConfig(mode === 'mock' ? mockConfig : viewsConfig, {
  root, configFile: false,
  cacheDir: path.join(process.env.TEMP, `vite-${mode}`),
  // Apply the same narrow mock-only allowance to the views harness without
  // changing vite.views.config.ts or the production config.
  server: { host: '127.0.0.1', port, strictPort: true, fs: mockConfig.server.fs },
}));
await server.listen();
server.printUrls();
for (const signal of ['SIGINT', 'SIGTERM']) process.once(signal, async () => { await server.close(); });
