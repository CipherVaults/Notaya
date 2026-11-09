// Execute original CommonJS suites with narrow, recorded infrastructure substitutions.
// Assertions, selectors, fixtures and production imports remain the original source's.
const fs = require('node:fs');
const path = require('node:path');
const Module = require('node:module');
const crypto = require('node:crypto');
const { root, mocks } = require('./catalog.cjs');
const [file, artifactDir, mockUrl, viewsUrl] = process.argv.slice(2);
if (!mocks.some(m => m.file === file) || !file.endsWith('.cjs')) throw Error('Unknown CJS suite');
fs.mkdirSync(artifactDir, { recursive: true });
const originalPath = path.join(root, 'dev-mock', file);
let source = fs.readFileSync(originalPath, 'utf8');
const sourceHash = crypto.createHash('sha256').update(source).digest('hex');
const changes = [];
function substitute(from, to) {
  if (source.includes(from)) { source = source.split(from).join(to); changes.push({ from, to }); }
}
substitute('http://127.0.0.1:1421', mockUrl);
substitute('http://127.0.0.1:1423', file === 'database-views-check.cjs' ? viewsUrl : mockUrl);
substitute('http://127.0.0.1:1424', mockUrl);
const slash = s => s.replaceAll('\\', '/');
substitute('D:/Notaya-lanes/out/C2', slash(artifactDir));
substitute('D:/Notaya-lanes/out/C3', slash(artifactDir));
substitute('D:/Notaya-lanes/out/A-shots', slash(artifactDir));
for (const old of ['D:/Notaya Tests/v1-6-C2/browser', 'D:/Notaya Tests/v1-6-C3u', 'D:/Notaya Tests/v1-6-C4d']) {
  substitute(old, slash(process.env.TEMP));
}
// Preserve __dirname and original relative resolution; calculator output goes outside repo.
if (file === 'calc-check.cjs') {
  const ts = require(path.join(root, 'node_modules/typescript'));
  const dir = path.join(process.env.TEMP, 'calc-compiled');
  fs.mkdirSync(dir, { recursive: true });
  const result = ts.transpileModule(fs.readFileSync(path.join(root, 'src/features/sticky/calc.ts'), 'utf8'), {
    compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2020 },
  });
  fs.writeFileSync(path.join(dir, 'calc.cjs'), result.outputText);
  substitute("require('../.calc-check/calc.js')", `require(${JSON.stringify(path.join(dir, 'calc.cjs'))})`);
}
const originalLoad = Module._load;
Module._load = function (request, parent, isMain) {
  if (request === 'playwright' || request === 'playwright-core' || request === process.env.NOTAYA_PLAYWRIGHT_MODULE) {
    const pw = originalLoad(process.env.NOTAYA_PLAYWRIGHT_MODULE, parent, isMain);
    const chromium = new Proxy(pw.chromium, { get(target, property) {
      if (property === 'launch') return options => target.launch({ ...options, executablePath: process.env.NOTAYA_CHROMIUM_PATH, headless: true });
      const value = Reflect.get(target, property); return typeof value === 'function' ? value.bind(target) : value;
    } });
    return { ...pw, chromium };
  }
  return originalLoad(request, parent, isMain);
};
const entry = mocks.find(m => m.file === file);
const base = file === 'database-views-check.cjs' ? viewsUrl : mockUrl;
const args = entry.mode === 'node' ? [] : entry.args === 'url' ? [base] : entry.args === 'url-shots' ? [base + '/', artifactDir] : [artifactDir];
process.env.MOCK_URL = file === 'database-fixes-ui-check.cjs' ? `${mockUrl}/?db=1` : mockUrl;
process.env.NOTAYA_MOCK_URL = mockUrl;
process.env.NOTAYA_AI_SHOTS = artifactDir;
process.argv = [process.execPath, originalPath, ...args];
fs.writeFileSync(path.join(artifactDir, 'adapter.json'), JSON.stringify({ originalPath, sourceHash, changes, args, note: 'Playwright module/Chrome/headless override; calculator transpile is infrastructure only; original assertions unchanged.' }, null, 2));
fs.writeFileSync(path.join(artifactDir, 'executed-source.cjs'), source);
const instance = new Module(originalPath, module);
instance.filename = originalPath;
instance.paths = Module._nodeModulePaths(path.dirname(originalPath));
instance._compile(source, originalPath);
