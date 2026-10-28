// installed.mjs - real-exe test of where Notaya keeps its notes (installed vs portable).
// Purpose: checks the first screen of an INSTALLED copy (no portable.txt, no data folder), that the
//          notes go to <LOCALAPPDATA>\Notaya Notes\data, that a second start opens them by itself, and
//          that a PORTABLE copy still uses its own data folder and refuses the import command.
//          Prints PASS / FAIL lines.
// Usage:   normally started by run-installed.ps1:  node installed.mjs <port> <phase> <expected-data-dir>
//          <phase> is first | second | portable. LOCALAPPDATA was set to a temp folder for the app.
// Needs:   Node 24 (global fetch and WebSocket) and a Notaya window started with the debug port.
//          Throwaway data only; it never makes or reads a real vault.
const [port, phase, expectedDir] = process.argv.slice(2);
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function connectMain() {
  for (let i = 0; i < 60; i++) {
    try {
      const t = (await (await fetch(`http://127.0.0.1:${port}/json`)).json()).filter((x) => x.type === "page");
      for (const hit of t) {
        const ws = new WebSocket(hit.webSocketDebuggerUrl);
        await new Promise((r) => (ws.onopen = r));
        let id = 0;
        const pend = new Map();
        ws.onmessage = (m) => { const d = JSON.parse(m.data); if (d.id) pend.get(d.id)?.(d); };
        const send = (method, params = {}) => new Promise((res) => { const i = ++id; pend.set(i, res); ws.send(JSON.stringify({ id: i, method, params })); });
        const ev = (expression) => send("Runtime.evaluate", { expression, awaitPromise: true, returnByValue: true }).then((r) => r.result?.result?.value ?? r.result?.exceptionDetails?.exception?.description);
        const inv = (cmd, args = {}) => ev(`window.__TAURI_INTERNALS__.invoke(${JSON.stringify(cmd)}, ${JSON.stringify(args)}).then(v => ({ok: v ?? null}), e => ({err: String(e)}))`);
        await send("Runtime.enable");
        const label = await ev("window.__TAURI_INTERNALS__?.metadata?.currentWindow?.label ?? ''");
        if (label === "main") return { inv, ev };
        ws.close();
      }
    } catch {}
    await sleep(500);
  }
  throw new Error("no main window");
}
const check = (name, ok, extra = "") => console.log(`${ok ? "PASS" : "FAIL"}  ${name} ${extra}`);
const m = await connectMain();
await sleep(1500);
const sameDir = (a, b) => typeof a === "string" && a.replace(/\\\\\?\\/, "").toLowerCase() === b.toLowerCase();

if (phase === "first") {
  const st = (await m.inv("vault_status")).ok;
  console.log("status:", JSON.stringify(st));
  check("installed: no notes yet, so the first screen offers the choice", st && st.exists === false && st.offerImport === true && st.unlocked === false);
  check("installed: the notes folder is under LOCALAPPDATA\\Notaya Notes", sameDir(st?.dataDir, expectedDir), st?.dataDir);
  const text = await m.ev("document.body.innerText");
  check("first screen says Welcome to Notaya", /Welcome to Notaya/.test(text));
  check("first screen has the import button", /Import notes from a portable copy/.test(text));
  check("first screen has the start button", /Start new notes/.test(text));
  await m.ev(`[...document.querySelectorAll("button")].find(b => /Start new notes/.test(b.innerText))?.click()`);
  await sleep(2500);
  const after = (await m.inv("vault_status")).ok;
  check("Start new notes makes and opens the notes", after && after.exists === true && after.unlocked === true && after.offerImport === false, JSON.stringify(after));
} else if (phase === "second") {
  const st = (await m.inv("vault_status")).ok;
  console.log("status:", JSON.stringify(st));
  check("second start: the notes open by themselves, no first screen", st && st.exists === true && st.unlocked === true && st.offerImport === false);
  check("second start: same notes folder", sameDir(st?.dataDir, expectedDir), st?.dataDir);
} else if (phase === "portable") {
  const st = (await m.inv("vault_status")).ok;
  console.log("status:", JSON.stringify(st));
  check("portable: the notes open by themselves, no first screen", st && st.exists === true && st.unlocked === true && st.offerImport === false);
  check("portable: the notes folder is next to the exe", sameDir(st?.dataDir, expectedDir), st?.dataDir);
  const imp = await m.inv("import_portable_notes");
  check("portable: the import command is refused (before any dialog)", !!imp.err && /installed Notaya/.test(imp.err), JSON.stringify(imp));
} else {
  throw new Error("phase must be first, second or portable");
}
process.exit(0); // the open DevTools socket would keep node (and run-installed.ps1) waiting for ever
