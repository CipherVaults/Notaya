// Notaya Token Saver: read-only settings checkup.
//
// Looks at a few places that decide how much Claude Code re-reads on every request and prints a
// plain report. It only READS files; it writes nothing, makes no network calls and runs no other
// program. It never prints the values of `env` entries (they can hold secrets): only their names.
// Output is ASCII only so a Windows console shows it correctly.
//
// Usage: node checkup.mjs [project-folder]     (default: the current folder)
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

const project = path.resolve(process.argv[2] || process.cwd());
const userDir = process.env.CLAUDE_CONFIG_DIR || path.join(os.homedir(), ".claude");

const lines = [];
const out = (text = "") => lines.push(text);
const notes = [];

function readText(file) {
  try {
    return fs.readFileSync(file, "utf8");
  } catch {
    return null;
  }
}

function readJson(file) {
  const text = readText(file);
  if (text === null) return { found: false };
  try {
    return { found: true, data: JSON.parse(text) };
  } catch {
    return { found: true, broken: true };
  }
}

function roughTokens(chars) {
  // Very rough: about 4 characters per token for English text. Only for comparing files.
  return Math.round(chars / 4);
}

// Everything printed from a settings file goes through here: only short plain values are shown, so a
// settings file in a project you did not write cannot push long text into Claude's context.
function clip(text, max = 80) {
  const t = String(text).replace(/[^\x20-\x7e]/g, "?");
  return t.length > max ? t.slice(0, max) + "..." : t;
}

function show(value) {
  if (value === undefined) return "(not set)";
  if (typeof value === "number" || typeof value === "boolean") return String(value);
  if (typeof value === "string") return JSON.stringify(clip(value, 60));
  return "(not a simple value)";
}

// 1. Settings files -----------------------------------------------------------------------------
out("SETTINGS (keys that change how much is re-read)");
const settingsFiles = [
  ["user", path.join(userDir, "settings.json")],
  ["project", path.join(project, ".claude", "settings.json")],
  ["project (local)", path.join(project, ".claude", "settings.local.json")],
];
const merged = {};
for (const [label, file] of settingsFiles) {
  const result = readJson(file);
  if (!result.found) {
    out(`  ${label}: no file at ${file}`);
    continue;
  }
  if (result.broken || typeof result.data !== "object" || result.data === null) {
    out(`  ${label}: ${file} could not be read as JSON (skipped)`);
    continue;
  }
  const d = result.data;
  out(`  ${label}: ${file}`);
  for (const key of ["autoCompactWindow", "bashOutputMaxChars", "model", "effortLevel"]) {
    if (d[key] !== undefined) {
      out(`    ${key} = ${show(d[key])}`);
      merged[key] = d[key]; // later files win, as in Claude Code's own order
    }
  }
  if (d.env && typeof d.env === "object") {
    out(`    env names set: ${Object.keys(d.env).map((k) => clip(k)).join(", ") || "(none)"}   (values not shown)`);
  }
  if (d.enabledPlugins && typeof d.enabledPlugins === "object") {
    const on = Object.entries(d.enabledPlugins).filter(([, v]) => v === true).map(([k]) => k);
    out(`    plugins turned on: ${on.length}${on.length ? " -> " + on.map((k) => clip(k)).join(", ") : ""}`);
    merged.pluginCount = (merged.pluginCount || 0) + on.length;
  }
  if (d.hooks && typeof d.hooks === "object") {
    out(`    hook events configured: ${Object.keys(d.hooks).map((k) => clip(k)).join(", ") || "(none)"}`);
  }
}
out();
if (merged.bashOutputMaxChars === undefined) {
  notes.push(
    "bashOutputMaxChars is not set, so Claude receives up to 30,000 characters of a successful command's output. " +
      'Setting it to 12000 is a suggestion (not measured): longer output is saved to a file and Claude gets a preview plus the path. Add  "bashOutputMaxChars": 12000  to your user settings.json.'
  );
}
if (merged.autoCompactWindow === undefined) {
  notes.push(
    "autoCompactWindow is not set. Claude Code then lets a session grow very large before it summarises. " +
      "Valid values are 100000 to 1000000; your own choice (for example 500000) is a trade-off between cost and lost detail."
  );
}
if (merged.effortLevel === "xhigh" || merged.effortLevel === "max") {
  notes.push(
    `effortLevel is ${merged.effortLevel}. Claude Code's guide warns of diminishing returns at the top levels; "high" is usually enough, and "medium" for everyday work.`
  );
}
if ((merged.pluginCount || 0) > 5) {
  notes.push(`${merged.pluginCount} plugins are turned on. Each adds skill descriptions or tool names to every session. Turn off the ones you do not use (/plugin).`);
}

// 2. CLAUDE.md files ----------------------------------------------------------------------------
out("CLAUDE.md FILES (loaded into every request)");
const memoryFiles = [
  ["user", path.join(userDir, "CLAUDE.md")],
  ["project", path.join(project, "CLAUDE.md")],
  ["project (.claude)", path.join(project, ".claude", "CLAUDE.md")],
  ["project (local)", path.join(project, "CLAUDE.local.md")],
];
let memoryTokens = 0;
for (const [label, file] of memoryFiles) {
  const text = readText(file);
  if (text === null) continue;
  const count = text.split(/\r?\n/).length;
  const tokens = roughTokens(text.length);
  memoryTokens += tokens;
  out(`  ${label}: ${count} lines, about ${tokens} tokens   ${file}`);
  if (count > 200) {
    notes.push(
      `${label} CLAUDE.md has ${count} lines. Claude Code's own cost guide says to keep it under 200 lines and to move rarely used procedures into skills, which load only when used.`
    );
  }
}
if (!memoryTokens) out("  none found");
out();

// 3. Skills (their descriptions are listed to Claude every session) ----------------------------------
out("SKILLS (name + description are listed every session; the body loads only when a skill is used)");
function describeSkills(dir, label) {
  let entries = [];
  try {
    entries = fs.readdirSync(dir, { withFileTypes: true }).filter((e) => e.isDirectory());
  } catch {
    return 0;
  }
  let total = 0;
  for (const e of entries) {
    const text = readText(path.join(dir, e.name, "SKILL.md"));
    if (text === null) continue;
    const front = /^---\r?\n([\s\S]*?)\r?\n---/.exec(text);
    const block = front ? front[1] : "";
    const descMatch = /^description:\s*([\s\S]*?)(?=^\S|\s*$)/m.exec(block);
    const description = descMatch ? descMatch[1].replace(/\s+/g, " ").trim() : "";
    const manual = /^disable-model-invocation:\s*true/m.test(block);
    total += description.length;
    out(
      `  ${label}/${clip(e.name)}: description ${description.length} chars` +
        (manual ? " (manual only: not listed to Claude)" : "") +
        `; SKILL.md ${text.split(/\r?\n/).length} lines`
    );
  }
  return total;
}
const skillChars =
  describeSkills(path.join(userDir, "skills"), "user") + describeSkills(path.join(project, ".claude", "skills"), "project");
if (skillChars) out(`  total description text listed to Claude: ${skillChars} chars (about ${roughTokens(skillChars)} tokens)`);
out();

// 4. MCP servers --------------------------------------------------------------------------------
out("MCP SERVERS in this project (.mcp.json)");
const mcp = readJson(path.join(project, ".mcp.json"));
if (!mcp.found) {
  out("  none");
} else if (mcp.broken || !mcp.data || typeof mcp.data.mcpServers !== "object") {
  out("  .mcp.json could not be read (skipped)");
} else {
  const names = Object.keys(mcp.data.mcpServers);
  out(`  ${names.length}: ${names.map((k) => clip(k)).join(", ") || "(none)"}`);
  if (names.length) notes.push("Each MCP server adds names (and its instructions) to every session. Use /mcp to turn off the ones you are not using; CLI tools such as gh are lighter.");
}
out("  (servers from your user settings or plugins are not listed here: run /mcp inside Claude Code)");
out();

// 5. Summary ------------------------------------------------------------------------------------
out("THINGS TO CONSIDER");
if (!notes.length) out("  Nothing stands out in these files.");
for (const n of notes) out(`  - ${n}`);
out();
out("Not checked here (needs Claude Code itself): run /context to see what fills the window,");
out("/usage to see where your plan usage went and what it recommends (press f to apply fixes, v2.1.305+).");
out("Token counts above are rough (about 4 characters per token), good for comparing files only.");

process.stdout.write(lines.join("\n") + "\n");
