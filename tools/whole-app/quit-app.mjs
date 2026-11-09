// Ask only the runner-owned loopback WebView to use Notaya's normal flush/quit path.
// The PowerShell caller proves PID exit separately. No process-name termination.
import { connect } from './cdp.mjs';
const client = await connect(process.argv[2]);
try {
  await client.evaluate(`(() => { setTimeout(() => { void window.__TAURI_INTERNALS__.invoke('quit_app'); }, 25); return true; })()`);
  console.log('PASS polite quit_app request scheduled; caller must confirm owned PID exited');
} finally { await client.close(); }
