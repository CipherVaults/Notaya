// Mermaid diagrams: loads the library only when a diagram is first drawn, draws the code to an SVG with
// the strictest settings, and cleans that SVG again before it reaches the page.
//
// The diagram code comes from the user, but also from imported files, so it is treated as hostile:
//  - securityLevel "strict": labels are escaped, and "click" actions and links do nothing;
//  - init directives and front matter in the code are dropped, so a diagram can't switch the settings
//    below back off (or set its own CSS);
//  - the SVG is parsed and cleaned again here (no scripts, no handlers, no links, no outside addresses)
//    and is added with DOM calls, never as an HTML string;
//  - hard limits on the code length and on the number of edges and the time one drawing may take.
// Nothing is fetched: no fonts, no icon packs, no images.

/** The longest diagram code that is drawn or kept (characters). */
export const MERMAID_MAX_CHARS = 50_000;
const MAX_EDGES = 500;
const RENDER_TIMEOUT_MS = 15_000;

export const MERMAID_TEMPLATES: { key: string; label: string; code: string }[] = [
  {
    key: "flowchart",
    label: "Flowchart",
    code: `flowchart TD
    A[Start] --> B{Is it ready?}
    B -->|Yes| C[Ship it]
    B -->|No| D[Fix it]
    D --> B`,
  },
  {
    key: "sequence",
    label: "Sequence",
    code: `sequenceDiagram
    participant Me
    participant App
    Me->>App: Open a page
    App-->>Me: Show the notes
    Me->>App: Add a diagram
    App-->>Me: Draw it`,
  },
  {
    key: "class",
    label: "Class",
    code: `classDiagram
    class Note {
      +String title
      +String body
      +save()
    }
    class Notebook {
      +String name
      +addNote()
    }
    Notebook "1" --> "*" Note : holds`,
  },
  {
    key: "state",
    label: "State",
    code: `stateDiagram-v2
    [*] --> Draft
    Draft --> Review : submit
    Review --> Draft : changes needed
    Review --> Done : approve
    Done --> [*]`,
  },
  {
    key: "gantt",
    label: "Gantt",
    code: `gantt
    title Project plan
    dateFormat YYYY-MM-DD
    section Build
    Design   :done, d1, 2026-01-05, 7d
    Develop  :active, d2, after d1, 10d
    section Launch
    Test     :d3, after d2, 5d
    Release  :milestone, after d3, 0d`,
  },
  {
    key: "pie",
    label: "Pie",
    code: `pie title Where the week went
    "Writing" : 40
    "Meetings" : 25
    "Reading" : 20
    "Other" : 15`,
  },
  {
    key: "er",
    label: "ER",
    code: `erDiagram
    NOTEBOOK ||--o{ PAGE : contains
    PAGE ||--o{ TAG : has
    PAGE {
      string title
      string body
    }
    TAG {
      string name
    }`,
  },
  {
    key: "mindmap",
    label: "Mindmap",
    code: `mindmap
  root((Notaya))
    Write
      Pages
      Diagrams
    Organise
      Notebooks
      Tags
    Keep safe
      Encrypted
      Offline`,
  },
  {
    key: "git",
    label: "Git graph",
    code: `gitGraph
    commit id: "first"
    branch feature
    checkout feature
    commit id: "work"
    commit id: "more work"
    checkout main
    merge feature
    commit id: "release"`,
  },
];

// ---------------------------------------------------------------------------------------------
// Colours: the diagram follows the look through the app's own colour variables.

const HEX = /^#([0-9a-f]{3}|[0-9a-f]{6})$/i;

function rgbOf(hex: string): [number, number, number] | null {
  const v = hex.trim();
  if (!HEX.test(v)) return null;
  const h = v.length === 4 ? [...v.slice(1)].map((c) => c + c).join("") : v.slice(1);
  return [0, 2, 4].map((i) => parseInt(h.slice(i, i + 2), 16)) as [number, number, number];
}

/** `a` with `t` (0..1) of `b` mixed in, as #rrggbb. */
function mix(a: string, b: string, t: number): string {
  const x = rgbOf(a) ?? [255, 255, 255];
  const y = rgbOf(b) ?? [0, 0, 0];
  return "#" + x.map((c, i) => Math.round(c + (y[i] - c) * t).toString(16).padStart(2, "0")).join("");
}

/** Hues for pie slices, branches and map branches: they stay readable on a light or a dark page. */
const HUES_LIGHT = ["#7aa7e8", "#f0a45d", "#6fc29a", "#e07f8e", "#b595e0", "#e6c453", "#5fbfc9", "#c9916a", "#9bb25b", "#d98cc0", "#8e9bb0", "#e58f6a"];
const HUES_DARK = ["#5b8fd6", "#d88a3f", "#4daa83", "#cc6677", "#9a78cf", "#c9a73a", "#42a8b3", "#b07850", "#82993f", "#c273a8", "#76839a", "#cf7650"];

export interface LookColors {
  dark: boolean;
  bg: string;
  text: string;
  accent: string;
}

function readLook(): LookColors {
  const root = document.documentElement;
  const css = getComputedStyle(root);
  const dark = root.dataset.mode === "dark";
  const pick = (name: string, fallback: string) => {
    const v = css.getPropertyValue(name).trim();
    return HEX.test(v) ? v : fallback;
  };
  return {
    dark,
    bg: pick("--bg", dark ? "#191919" : "#ffffff"),
    text: pick("--text", dark ? "#ececec" : "#1f1f1f"),
    accent: pick("--accent", dark ? "#ffffff" : "#0a0a0a"),
  };
}

/** A key that changes whenever the diagram would look different (so a drawing can be reused or must be redone). */
export function lookKey(): string {
  const l = readLook();
  return `${l.dark ? "d" : "l"}${l.bg}${l.text}${l.accent}`;
}

export function themeVariables(l: LookColors): Record<string, string | boolean> {
  const { bg, text, accent, dark } = l;
  const hues = dark ? HUES_DARK : HUES_LIGHT;
  const box = mix(bg, text, dark ? 0.14 : 0.07);
  const boxBorder = mix(bg, text, 0.42);
  const line = mix(bg, text, 0.62);
  const tint = mix(bg, accent, dark ? 0.3 : 0.2);
  const v: Record<string, string | boolean> = {
    darkMode: dark,
    background: bg,
    fontFamily: "Inter Variable, Segoe UI Variable Text, Segoe UI, system-ui, sans-serif",
    fontSize: "14px",
    primaryColor: box,
    primaryTextColor: text,
    primaryBorderColor: boxBorder,
    secondaryColor: tint,
    secondaryTextColor: text,
    secondaryBorderColor: mix(bg, accent, 0.55),
    tertiaryColor: mix(bg, text, 0.04),
    tertiaryTextColor: text,
    tertiaryBorderColor: mix(bg, text, 0.3),
    lineColor: line,
    textColor: text,
    mainBkg: box,
    nodeBorder: boxBorder,
    nodeTextColor: text,
    clusterBkg: mix(bg, text, 0.035),
    clusterBorder: mix(bg, text, 0.3),
    titleColor: text,
    edgeLabelBackground: bg,
    arrowheadColor: line,
    // Sequence diagrams
    actorBkg: box,
    actorBorder: boxBorder,
    actorTextColor: text,
    actorLineColor: line,
    signalColor: text,
    signalTextColor: text,
    labelBoxBkgColor: box,
    labelBoxBorderColor: boxBorder,
    labelTextColor: text,
    loopTextColor: text,
    noteBkgColor: tint,
    noteTextColor: text,
    noteBorderColor: mix(bg, accent, 0.55),
    activationBkgColor: tint,
    activationBorderColor: boxBorder,
    sequenceNumberColor: bg,
    // Class, state and ER diagrams
    classText: text,
    labelColor: text,
    altBackground: mix(bg, text, 0.05),
    transitionColor: line,
    transitionLabelColor: text,
    stateLabelColor: text,
    stateBkg: box,
    compositeBackground: mix(bg, text, 0.04),
    compositeTitleBackground: box,
    compositeBorder: boxBorder,
    specialStateColor: line,
    errorBkgColor: mix(bg, "#e5484d", 0.3),
    errorTextColor: text,
    attributeBackgroundColorOdd: mix(bg, text, 0.04),
    attributeBackgroundColorEven: mix(bg, text, 0.09),
    // Gantt charts
    gridColor: mix(bg, text, 0.2),
    sectionBkgColor: mix(bg, text, 0.06),
    altSectionBkgColor: bg,
    sectionBkgColor2: mix(bg, text, 0.1),
    taskBkgColor: hues[0],
    taskBorderColor: mix(hues[0], text, 0.35),
    taskTextColor: dark ? "#ffffff" : "#1a1a1a",
    taskTextLightColor: dark ? "#ffffff" : "#1a1a1a",
    taskTextDarkColor: dark ? "#ffffff" : "#1a1a1a",
    taskTextOutsideColor: text,
    taskTextClickableColor: text,
    activeTaskBkgColor: hues[1],
    activeTaskBorderColor: mix(hues[1], text, 0.35),
    doneTaskBkgColor: mix(bg, text, 0.3),
    doneTaskBorderColor: mix(bg, text, 0.5),
    critBkgColor: hues[3],
    critBorderColor: mix(hues[3], text, 0.35),
    todayLineColor: hues[3],
    excludeBkgColor: mix(bg, text, 0.08),
    // Pie charts
    pieTitleTextColor: text,
    pieSectionTextColor: dark ? "#ffffff" : "#1a1a1a",
    pieLegendTextColor: text,
    pieStrokeColor: bg,
    pieOuterStrokeColor: mix(bg, text, 0.3),
    pieOpacity: "0.95",
    // Git graphs
    gitBranchLabel0: dark ? "#ffffff" : "#1a1a1a",
    commitLabelColor: text,
    commitLabelBackground: mix(bg, text, 0.1),
    tagLabelColor: text,
    tagLabelBackground: box,
    tagLabelBorder: boxBorder,
  };
  hues.forEach((h, i) => {
    v[`pie${i + 1}`] = h;
    v[`cScale${i}`] = h;
    v[`cScaleLabel${i}`] = dark ? "#ffffff" : "#1a1a1a";
    v[`cScaleInv${i}`] = mix(h, text, 0.5);
    if (i < 8) {
      v[`git${i}`] = h;
      v[`gitBranchLabel${i}`] = dark ? "#ffffff" : "#1a1a1a";
    }
  });
  return v;
}

// ---------------------------------------------------------------------------------------------
// Drawing

type MermaidApi = typeof import("mermaid").default;
let loading: Promise<MermaidApi> | null = null;

/** Loads the library the first time it is needed, so the app starts as fast as before. */
function load(): Promise<MermaidApi> {
  loading ??= import("mermaid").then((m) => m.default);
  loading.catch(() => (loading = null));
  return loading;
}

/** Removes init directives (`%%{init: ...}%%`) and front matter (`--- ... ---`): see the notes at the top. */
export function prepareCode(raw: string): { code: string; dropped: boolean } {
  let code = raw.replace(/\r\n?/g, "\n");
  let dropped = false;
  const fm = /^\s*---[ \t]*\n[\s\S]*?\n---[ \t]*(?:\n|$)/.exec(code);
  if (fm) {
    code = code.slice(fm[0].length);
    dropped = true;
  }
  const without = code.replace(/%%\{[\s\S]*?\}%%/g, "");
  if (without !== code) {
    code = without;
    dropped = true;
  }
  return { code: code.trim(), dropped };
}

const DROP_ELEMENTS = new Set([
  "script", "iframe", "frame", "frameset", "object", "embed", "applet", "link", "meta", "base", "audio", "video", "source", "track", "img", "image",
  "animate", "animatemotion", "animatetransform", "set", "discard", "handler", "listener", "form", "input", "button", "textarea", "select", "dialog", "template", "slot",
]);
const KEEP_HTML = new Set(["div", "span", "p", "br", "b", "i", "em", "strong", "u", "s", "code", "pre", "small", "sub", "sup", "ul", "ol", "li", "tspan", "title", "desc"]);
const HTML_NS = "http://www.w3.org/1999/xhtml";

/** Nothing that points outside the drawing: only same-document references (#id). */
function cleanCss(css: string): string {
  return css
    .replace(/@import[^;]*;?/gi, "")
    .replace(/url\(\s*(['"]?)\s*(?!#)[^)]*\)/gi, "none")
    .replace(/expression\s*\(/gi, "(")
    .replace(/javascript:/gi, "");
}

function cleanNode(el: Element): void {
  for (const child of Array.from(el.children)) {
    const name = child.localName.toLowerCase();
    const html = child.namespaceURI === HTML_NS;
    if (DROP_ELEMENTS.has(name)) {
      child.remove();
      continue;
    }
    if (name === "a") {
      // A link would open something; keep what was inside it.
      child.replaceWith(...Array.from(child.childNodes));
      cleanNode(el);
      return;
    }
    if (html && !KEEP_HTML.has(name)) {
      child.replaceWith(document.createTextNode(child.textContent ?? ""));
      continue;
    }
    if (name === "style") child.textContent = cleanCss(child.textContent ?? "");
    for (const attr of Array.from(child.attributes)) {
      const n = attr.name.toLowerCase();
      const v = attr.value;
      if (n.startsWith("on") || n === "srcdoc" || n === "formaction" || n === "target" || n === "ping") child.removeAttribute(attr.name);
      else if (n === "href" || n === "xlink:href" || n === "src" || n === "data" || n === "action") {
        if (!(name === "use" || name === "textpath" || /^(lineargradient|radialgradient|pattern|filter|mask|clippath|marker)$/.test(name)) || !v.trim().startsWith("#")) child.removeAttribute(attr.name);
      } else if (n === "style" || /^(fill|stroke|filter|mask|clip-path|marker-start|marker-mid|marker-end|cursor)$/.test(n)) {
        const cleaned = cleanCss(v);
        if (cleaned !== v) child.setAttribute(attr.name, cleaned);
      }
    }
    cleanNode(child);
  }
}

/** Parses the library's SVG text into a node tree and cleans it. Throws when it isn't an SVG. */
export function cleanSvg(svgText: string): SVGSVGElement {
  // The library's text can use the "xlink:" prefix without declaring it, which an XML parser refuses.
  const xml = svgText.replace(/^\s*<svg\b/, (m) => (/xmlns:xlink=/.test(svgText.slice(0, 600)) ? m : `${m} xmlns:xlink="http://www.w3.org/1999/xlink"`));
  let root: Element | null = null;
  const parsed = new DOMParser().parseFromString(xml, "image/svg+xml");
  if (parsed.documentElement?.localName === "svg" && !parsed.getElementsByTagName("parsererror").length) root = parsed.documentElement;
  else root = new DOMParser().parseFromString(svgText, "text/html").body.querySelector("svg");
  if (!root) throw new Error("The diagram could not be drawn.");
  cleanNode(root);
  const svg = document.importNode(root, true) as unknown as SVGSVGElement;
  svg.removeAttribute("height");
  svg.style.maxWidth = svg.style.maxWidth || "100%";
  svg.style.height = "auto";
  svg.setAttribute("role", "img");
  return svg;
}

export type Drawing = { ok: true; svg: SVGSVGElement; dropped: boolean } | { ok: false; error: string };

let counter = 0;
// The library draws one diagram at a time (it uses a shared scratch area), so drawings wait in line.
let line: Promise<unknown> = Promise.resolve();

function shortError(e: unknown): string {
  const raw = e instanceof Error ? e.message : String(e);
  const lines = raw.split("\n").map((l) => l.trimEnd()).filter((l) => l.trim());
  return lines.slice(0, 6).join("\n").slice(0, 500) || "The diagram could not be drawn.";
}

// The last few drawings, so a page that is shown again (or an editor that rebuilds its blocks) doesn't draw them again.
const cache = new Map<string, { svg: SVGSVGElement; dropped: boolean }>();
const CACHE_SIZE = 24;

export function drawMermaid(source: string): Promise<Drawing> {
  const cacheKey = `${lookKey()}\u0000${source}`;
  const hit = cache.get(cacheKey);
  if (hit) return Promise.resolve({ ok: true, svg: hit.svg.cloneNode(true) as SVGSVGElement, dropped: hit.dropped });
  const run = async (): Promise<Drawing> => {
    const { code, dropped } = prepareCode(source);
    if (code.length === 0) return { ok: false, error: "" };
    if (source.length > MERMAID_MAX_CHARS) return { ok: false, error: `The diagram code is longer than ${MERMAID_MAX_CHARS.toLocaleString()} characters.` };
    const id = `notaya-mmd-${++counter}`;
    try {
      const mermaid = await load();
      mermaid.initialize({
        startOnLoad: false,
        securityLevel: "strict",
        theme: "base",
        themeVariables: themeVariables(readLook()),
        fontFamily: "Inter Variable, Segoe UI Variable Text, Segoe UI, system-ui, sans-serif",
        htmlLabels: false,
        flowchart: { htmlLabels: false, useMaxWidth: true },
        sequence: { useMaxWidth: true },
        gantt: { useMaxWidth: true },
        maxTextSize: MERMAID_MAX_CHARS,
        maxEdges: MAX_EDGES,
        suppressErrorRendering: true,
        logLevel: 5,
        deterministicIds: false,
      });
      const work = (async () => {
        await mermaid.parse(code);
        return mermaid.render(id, code);
      })();
      let timer = 0;
      const limit = new Promise<never>((_, reject) => {
        timer = window.setTimeout(() => reject(new Error("Drawing took too long, so it was stopped. Try a smaller diagram.")), RENDER_TIMEOUT_MS);
      });
      try {
        const out = await Promise.race([work, limit]);
        const svg = cleanSvg(out.svg);
        cache.set(cacheKey, { svg: svg.cloneNode(true) as SVGSVGElement, dropped });
        if (cache.size > CACHE_SIZE) cache.delete(cache.keys().next().value as string);
        return { ok: true, svg, dropped };
      } finally {
        window.clearTimeout(timer);
        work.catch(() => {});
      }
    } catch (e) {
      return { ok: false, error: shortError(e) };
    } finally {
      // The library can leave its scratch element behind when a drawing fails.
      document.getElementById(`d${id}`)?.remove();
      document.getElementById(id)?.remove();
    }
  };
  const result = line.then(run, run);
  line = result.catch(() => {});
  return result;
}

/** Calls `cb` whenever the look (light/dark, colours) changes. Returns the way to stop. */
export function onLookChange(cb: () => void): () => void {
  const obs = new MutationObserver(() => cb());
  obs.observe(document.documentElement, { attributes: true, attributeFilter: ["data-theme", "data-mode", "data-accent", "style"] });
  return () => obs.disconnect();
}
