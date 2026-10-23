// shot.mjs - save one PNG screenshot of the Cipherleaf MOCK UI (fake data, plain browser; development only).
// Purpose: documentation and review screenshots without the real app or any real vault.
// Usage:   node shot.mjs <out.png> <url> [js-before] [waitMs] [width] [height]
//          e.g. node shot.mjs home.png http://127.0.0.1:1421/ "" 900 1360 860
//          Needs the mock running (npm run mock, port 1421) and a headless Edge on port 9333 with its own
//          throwaway profile folder. See README.md in this folder for the exact commands and the cleanup.
// Needs:   Node 24 (global fetch and WebSocket), Microsoft Edge.
import { writeFileSync } from "node:fs";

const [out, url, js = "", wait = "900", w = "1360", h = "860"] = process.argv.slice(2);
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

let targets;
for (let i = 0; i < 40; i++) {
  try { targets = await (await fetch("http://127.0.0.1:9333/json")).json(); break; } catch { await sleep(250); }
}
if (!targets) { console.error("No browser answers on port 9333. Start headless Edge first (see README.md)."); process.exit(1); }
const page = targets.find((t) => t.type === "page");
const ws = new WebSocket(page.webSocketDebuggerUrl);
await new Promise((r) => (ws.onopen = r));
let id = 0; const pend = new Map(); const logs = [];
ws.onmessage = (m) => {
  const d = JSON.parse(m.data);
  if (d.id) pend.get(d.id)?.(d);
  else if (d.method === "Runtime.exceptionThrown") logs.push("EXC " + (d.params.exceptionDetails.exception?.description ?? d.params.exceptionDetails.text).slice(0, 300));
  else if (d.method === "Runtime.consoleAPICalled" && ["error", "warning"].includes(d.params.type)) logs.push(d.params.type + " " + d.params.args.map((a) => a.value ?? a.description ?? "").join(" ").slice(0, 300));
};
const send = (method, params = {}) => new Promise((res) => { const i = ++id; pend.set(i, res); ws.send(JSON.stringify({ id: i, method, params })); });
const ev = (expression) => send("Runtime.evaluate", { expression, awaitPromise: true, returnByValue: true }).then((r) => r.result?.result?.value ?? r.result?.exceptionDetails?.exception?.description);

await send("Runtime.enable");
await send("Emulation.setDeviceMetricsOverride", { width: +w, height: +h, deviceScaleFactor: 1, mobile: false });
await send("Page.enable");
await send("Page.navigate", { url });
await sleep(+wait + 1500);
if (js) { const r = await ev(js); if (r) console.log("js:", String(r).slice(0, 300)); await sleep(+wait); }
const shot = await send("Page.captureScreenshot", { format: "png" });
writeFileSync(out, Buffer.from(shot.result.data, "base64"));
console.log("saved", out, "| text:", (await ev("document.body.innerText.replace(/\\s+/g,' ').slice(0,100)")));
if (logs.length) console.log("console:\n  " + [...new Set(logs)].slice(0, 8).join("\n  "));
ws.close();
process.exit(0);
