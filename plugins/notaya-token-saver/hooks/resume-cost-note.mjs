// Notaya Token Saver: SessionStart hook for "resume" and "fork".
//
// What it does: when you resume an old session whose prompt cache has expired, it shows YOU one
// line (systemMessage) saying how much the first message will re-send. It uses only fields that
// Claude Code itself puts in the hook input (prompt_cache_likely_expired, context_tokens,
// seconds_since_last_response, estimated_cache_write_usd; Claude Code v2.1.251 or newer).
//
// What it does NOT do: no network, no file writes, no shell, no reading of your conversation.
// It adds nothing to Claude's context (systemMessage is shown to the person only).
// If anything is missing or goes wrong it prints nothing and exits 0, so the session starts as normal.
// Output is ASCII only so a Windows console shows it correctly.
import { readFileSync } from "node:fs";

function num(value) {
  const n = Number(value);
  return Number.isFinite(n) ? n : null;
}

function readSetting(name, fallback) {
  const n = num(process.env[name]);
  return n !== null && n >= 0 ? n : fallback;
}

function approxTokens(n) {
  return n >= 1000 ? `${Math.round(n / 1000)}k` : String(Math.round(n));
}

function approxDuration(seconds) {
  const minutes = Math.round(seconds / 60);
  if (minutes < 120) return `${minutes} min`;
  const hours = Math.floor(minutes / 60);
  return `${hours} h ${minutes % 60} min`;
}

function main() {
  let input;
  try {
    input = JSON.parse(readFileSync(0, "utf8"));
  } catch {
    return;
  }
  if (!input || (input.source !== "resume" && input.source !== "fork")) return;
  if (input.prompt_cache_likely_expired !== true) return;

  const tokens = num(input.context_tokens);
  if (tokens === null) return;
  // Small sessions are cheap to re-send, so stay quiet. Change with NOTAYA_TOKEN_SAVER_MIN_TOKENS.
  if (tokens < readSetting("NOTAYA_TOKEN_SAVER_MIN_TOKENS", 50000)) return;

  const seconds = num(input.seconds_since_last_response);
  const usd = num(input.estimated_cache_write_usd);

  const idle =
    seconds !== null && seconds >= 300
      ? `This session has been idle for about ${approxDuration(seconds)}, so its saved prompt cache has expired.`
      : "This session's saved prompt cache is no longer warm.";
  const cost =
    usd !== null && usd > 0
      ? ` (Claude Code estimates about $${usd.toFixed(2)} at API prices; on a plan it counts against your limit)`
      : "";

  const message =
    `Token saver: ${idle} Your first message will re-send about ${approxTokens(tokens)} tokens` +
    `${cost}. If you do not need the old conversation, run /clear (or /rename first, so you can /resume it later) ` +
    "and describe the task in a few lines instead.";

  process.stdout.write(JSON.stringify({ systemMessage: message }) + "\n");
}

try {
  main();
} catch {
  // Fail open: never block or break a session start.
}
process.exit(0);
