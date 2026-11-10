/* DevKit pace forecast: one small model, used by the dashboard page and by build_dashboard.py.
 *
 * It answers one question in rough terms: given Primary's Codex weekly allowance (plus banked
 * resets), Claude's weekly allowance (capped by the supplied input) and a points-per-version plan,
 * on which day does each version finish?  Every input is a row in example/forecast-inputs.csv
 * with a kind of "measured" or "rough"; the model itself owns no facts.
 *
 * Units: a WINDOW is one full Codex 5-hour window of building. The model measures work in
 * windows (points x overhead / points per window) and supply in windows (weekly allowance =
 * windows per week). Claude's allowance is in percent of its week.
 *
 * Run as a program:  node forecast-model.cjs < inputs.json   (prints pace-forecast.csv rows)
 */
(function (root) {
  "use strict";
  var DAY = 864e5;

  // inp: {asOf, codexUsed, codexReset, claudeUsed, claudeReset, claudeCap}
  // p:   {ppw, wpw, ov, banks, cap, cw, base, secondary, mcp, gap}
//      secondary = share of Secondary's own Claude week he spends BUILDING (he has no Codex, only Claude Pro);
//      mcp = % of his Claude week one plan point costs when Claude builds it directly
  // versions: [{id, pts}]
  function run(inp, p, versions) {
    var dt = 0.25;                                  // a quarter of a day per step
    var t = inp.asOf;
    var xb = (1 - inp.codexUsed / 100) * p.wpw;     // Primary's Codex allowance left, in windows
    var cb = inp.claudeCap - inp.claudeUsed;        // Claude allowance left, % of week, up to the cap
    var mWeek = p.secondary > 0 ? (p.secondary * 100 / p.mcp) * p.ov / p.ppw : 0;   // what his Claude week builds, in window-equivalents
    var mb = 0;                                     // Secondary's building allowance, in windows (up to one week's worth)
    var banks = p.banks;
    var cReset = inp.codexReset, kReset = inp.claudeReset;
    var burstEnd = null, bankAt = [], resets = [], hold = 0;
    var work = versions.map(function (v) { return v.pts * p.ov / p.ppw; });
    var out = versions.map(function (v, i) {
      return { id: v.id, pts: v.pts, windows: work[i], start: null, end: null, primary: 0, secondary: 0, claude: 0, waitClaude: 0, waitCodex: 0, run: 0 };
    });
    var vi = 0, rest = work[0] || 0, series = [{ t: t, x: xb, c: cb }], limited = 0;
    for (var n = 0; n < 4 * 420 && vi < versions.length; n++) {
      t += dt * DAY;
      if (t >= cReset) { xb = p.wpw; resets.push({ t: cReset, kind: "codex" }); cReset += 7 * DAY; }
      if (t >= kReset) { cb = inp.claudeCap; resets.push({ t: kReset, kind: "claude" }); kReset += 7 * DAY; }
      cb -= p.base * dt;
      mb = Math.min(mWeek, mb + mWeek / 7 * dt);
      if (t < hold) { if (n % 2 === 0) series.push({ t: t, x: xb, c: cb }); continue; }   // the pause after a version: PC round, review, Primary's words
      // A banked reset (it refills the whole weekly allowance and the 5-hour window) is used only after the weekly allowance is fully spent, and only when the next weekly reset is more than a day away (otherwise it is better to wait).
      if (xb <= 1e-6 && banks > 0 && cb > 0 && (cReset - t) > DAY) { xb = p.wpw; banks--; bankAt.push(t); }
      if (xb <= 1e-6 && banks === 0 && burstEnd === null) burstEnd = t;
      var want = p.cap * dt;
      var byClaude = cb > 0 ? cb / p.cw : 0;
      if (byClaude < want) limited += dt;
      var cur = out[Math.min(vi, out.length - 1)];
      if (byClaude < want && byClaude <= xb) cur.waitClaude += dt * (1 - byClaude / want);
      else if (xb < want) cur.waitCodex += dt * (1 - xb / want);
      cur.run += dt;
      var can = Math.min(want, byClaude);
      var useX = Math.min(xb, can), useM = Math.min(mb, mWeek / 7 * dt * 2);   // Secondary builds on his own account, alongside, up to twice his average pace
      var used = useM + useX;
      var fm = used > 0 ? useM / used : 0;          // share of this step paid by Secondary
      xb -= useX; mb -= useM; cb -= useX * p.cw;   // only Primary's Codex lanes cost Primary's Claude week
      while (used > 1e-9 && vi < versions.length) {
        var o = out[vi];
        if (o.start === null) o.start = t - dt * DAY;
        var take = Math.min(used, rest);
        o.secondary += take * fm; o.primary += take * (1 - fm); o.claude += take * p.cw;
        used -= take; rest -= take;
        if (rest <= 1e-9) { o.end = t; vi++; rest = vi < versions.length ? work[vi] : 0; hold = t + p.gap * DAY; used = 0; }
      }
      if (n % 2 === 0) series.push({ t: t, x: xb, c: cb });
    }
    out.forEach(function (o) {
      o.burst = burstEnd === null || (o.end !== null && o.end <= burstEnd) ? "primary" : "after";
    });
    return { versions: out, burstEnd: burstEnd, bankAt: bankAt, resets: resets, series: series,
             end: out.length ? out[out.length - 1].end : null, claudeLimitedDays: limited };
  }

  // Three cases from one set of settings. Best and worst move the weakest numbers together.
  function cases(inp, p, versions, scale) {
    var s = scale || { bestPts: 0.85, worstPts: 1.3 };
    function make(kind) {
      var q = Object.assign({}, p), k = 1;
      if (kind === "best") { q.ppw = p.ppw * 1.25; q.ov = Math.max(1, p.ov - 0.2); q.cw = p.cw * 0.5; q.gap = p.gap * 0.5; k = s.bestPts; }
      if (kind === "worst") { q.ppw = p.ppw * 0.7; q.ov = p.ov + 0.25; q.cw = p.cw * 2; q.wpw = p.wpw * 0.9; q.gap = p.gap * 2; k = s.worstPts; }
      var vs = versions.map(function (v, i) {
        var pts = v.pts;
        if (i === 0) pts = kind === "best" ? (v.low != null ? v.low : v.pts) : kind === "worst" ? (v.high != null ? v.high : v.pts) : v.pts;
        else pts = v.pts * k;
        return { id: v.id, pts: pts };
      });
      return run(inp, q, vs);
    }
    return { best: make("best"), likely: make("likely"), worst: make("worst") };
  }

  var api = { run: run, cases: cases, DAY: DAY };
  if (typeof module !== "undefined" && module.exports) module.exports = api; else root.DevKitForecast = api;

  if (typeof require !== "undefined" && typeof module !== "undefined" && require.main === module) {
    var buf = "";
    process.stdin.on("data", function (d) { buf += d; });
    process.stdin.on("end", function () {
      var j = JSON.parse(buf), inp = j.inp;
      ["asOf", "codexReset", "claudeReset"].forEach(function (k) { inp[k] = Date.parse(inp[k]); });
      var c = cases(inp, j.params, j.versions);
      function d(ms) { return ms == null ? "beyond forecast horizon" : new Date(ms).toISOString().slice(0, 10); }
      console.log("version,plan_points,codex_windows,codex_pct_of_week,claude_pct_of_week,finish_best,finish_likely,finish_worst,paid_by");
      c.likely.versions.forEach(function (v, i) {
        var b = c.best.versions[i], w = c.worst.versions[i];
        console.log([v.id, v.pts.toFixed(0), v.windows.toFixed(1), (v.windows / j.params.wpw * 100).toFixed(0),
          v.claude.toFixed(0), d(b.end), d(v.end), d(w.end),
          v.burst === "primary" ? "Primary's quota (weekly + banked resets)" : "after Primary's banked resets: his weekly allowance only"].join(","));
      });
    });
  }
})(typeof window !== "undefined" ? window : this);
