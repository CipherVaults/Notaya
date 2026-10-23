// act.mjs - drive the Cipherleaf MOCK UI (fake data, plain browser; development only) with a list of steps.
// Purpose: scripted clicks / typing / screenshots of the mock, for documentation and review.
// Usage:   node act.mjs <url> <steps.json>      (env VW / VH set the viewport, default 1360x860)
//          steps.json is an array of steps: {js}|{key}|{type}|{click:[x,y]}|{clickJs}|{move:[x,y]}|{wait}|{shot:"file.png"}
//          Needs the mock running (npm run mock, port 1421) and a headless Edge on port 9333 with its own
//          throwaway profile folder. See README.md in this folder.
// Needs:   Node 24 (global fetch and WebSocket), Microsoft Edge.
import { readFileSync, writeFileSync } from "node:fs";
const [url, stepsFile] = process.argv.slice(2);
const steps = JSON.parse(readFileSync(stepsFile, "utf8"));
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const targets = await (await fetch("http://127.0.0.1:9333/json")).json();
const page = targets.find((t) => t.type === "page");
const ws = new WebSocket(page.webSocketDebuggerUrl); await new Promise((r) => (ws.onopen = r));
let id = 0; const pend = new Map(); const logs = [];
ws.onmessage = (m) => { const d = JSON.parse(m.data); if (d.id) pend.get(d.id)?.(d);
  else if (d.method === "Runtime.exceptionThrown") logs.push("EXC " + (d.params.exceptionDetails.exception?.description ?? d.params.exceptionDetails.text).slice(0, 3500));
  else if (d.method === "Runtime.consoleAPICalled" && d.params.type === "error") logs.push("console.error " + d.params.args.map((a) => a.value ?? a.description ?? "").join(" ").slice(0, 3500)); };
const send = (method, params = {}) => new Promise((res) => { const i = ++id; pend.set(i, res); ws.send(JSON.stringify({ id: i, method, params })); });
const ev = (e) => send("Runtime.evaluate", { expression: e, awaitPromise: true, returnByValue: true }).then((r) => r.result?.result?.value ?? r.result?.exceptionDetails?.exception?.description);
await send("Runtime.enable"); await send("Page.enable");
await send("Emulation.setDeviceMetricsOverride", { width: +(process.env.VW||1360), height: +(process.env.VH||860), deviceScaleFactor: 1, mobile: false });
await send("Page.navigate", { url }); await sleep(2800);
const KEYS = { Enter: 13, Escape: 27, ArrowDown: 40, ArrowUp: 38, Tab: 9, Backspace: 8 };
for (const s of steps) {
  if (s.js) { const r = await ev(s.js); if (r !== undefined && r !== null) console.log("js:", JSON.stringify(r).slice(0, 400)); }
  if (s.type) for (const ch of s.type) { await send("Input.dispatchKeyEvent", { type: "keyDown", text: ch, key: ch }); await send("Input.dispatchKeyEvent", { type: "keyUp", key: ch }); await sleep(30); }
  if (s.key) { const vk = KEYS[s.key]; await send("Input.dispatchKeyEvent", { type: "rawKeyDown", key: s.key, windowsVirtualKeyCode: vk, code: s.key }); await send("Input.dispatchKeyEvent", { type: "keyUp", key: s.key, windowsVirtualKeyCode: vk, code: s.key }); }
  if (s.click) { const [x, y] = s.click; await send("Input.dispatchMouseEvent", { type: "mouseMoved", x, y }); await send("Input.dispatchMouseEvent", { type: "mousePressed", x, y, button: "left", clickCount: s.count ?? 1 }); await send("Input.dispatchMouseEvent", { type: "mouseReleased", x, y, button: "left", clickCount: s.count ?? 1 }); }
  if (s.clickJs) { const xy = await ev(s.clickJs); const [x, y] = xy; await send("Input.dispatchMouseEvent", { type: "mouseMoved", x, y }); await send("Input.dispatchMouseEvent", { type: "mousePressed", x, y, button: "left", clickCount: s.count ?? 1 }); await send("Input.dispatchMouseEvent", { type: "mouseReleased", x, y, button: "left", clickCount: s.count ?? 1 }); }
  if (s.move) await send("Input.dispatchMouseEvent", { type: "mouseMoved", x: s.move[0], y: s.move[1] });
  if (s.wait) await sleep(s.wait);
  if (s.shot) { const shot = await send("Page.captureScreenshot", { format: "png" }); writeFileSync(s.shot, Buffer.from(shot.result.data, "base64")); console.log("saved", s.shot); }
}
console.log(logs.length ? "ERRORS:\n" + [...new Set(logs)].join("\n") : "no console errors");
process.exit(0);
