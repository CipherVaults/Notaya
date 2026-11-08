// Run the actual equation node-view input handler with ProseMirror history.
// Minimal DOM doubles keep this regression offline and browser-independent.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const vm = require('node:vm');
const ts = require('typescript');
const { Schema } = require('@tiptap/pm/model');
const { EditorState } = require('@tiptap/pm/state');
const { history, undo, redo, undoDepth } = require('@tiptap/pm/history');
const elements = [];
function element() {
  const el = { dataset: {}, style: {}, handlers: {}, classList: { toggle() {} },
    append() {}, focus() {}, setSelectionRange() {}, remove() {}, contains() { return false; },
    getBoundingClientRect: () => ({ left: 10, right: 100, top: 10, bottom: 30, width: 90 }),
    addEventListener(event, handler) { this.handlers[event] = handler; },
  };
  elements.push(el);
  return el;
}
const exportsObject = {};
const source = fs.readFileSync('src/features/notes/editor/math.ts', 'utf8');
vm.runInNewContext(ts.transpileModule(source, { compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2020, esModuleInterop: true } }).outputText, {
  exports: exportsObject,
  require(name) {
    if (name === 'katex') return { render() {} };
    if (name === '@tiptap/react') return { Node: { create: (config) => ({ config }) } };
    if (name === '@tiptap/pm/state') return require(name);
    throw new Error('Unexpected test dependency');
  },
  document: { createElement: element, body: { append() {} } },
  window: { innerWidth: 1000, innerHeight: 1000, addEventListener() {}, removeEventListener() {} },
});
const schema = new Schema({ nodes: {
  doc: { content: 'block+' }, paragraph: { group: 'block', content: 'text*' }, text: {},
  mathBlock: { group: 'block', atom: true, attrs: { latex: { default: '' }, bid: { default: null } } },
} });
const bid = 'eeeeeeeeeeeeeeee';
const exact = '\u200f\u0639\u05d0 e\u0301\u200e  ';
const fixtureExports = {};
const fixtureSource = fs.readFileSync('dev-mock/equation-fixture.ts', 'utf8');
vm.runInNewContext(ts.transpileModule(fixtureSource, { compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2020 } }).outputText, {
  exports: fixtureExports, require,
});
const create = exportsObject.MathBlock.config.addNodeView.call({ storage: { justInserted: { current: false } } });
const original = { latex: String.raw`x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}`, bid: 'aaaaaaaaaaaaaaaa' };
for (const isolated of [false, true]) {
  elements.length = 0;
  let state = EditorState.create({ schema, plugins: [history()], doc: schema.node('doc', null, [
    schema.node('mathBlock', original), schema.node('paragraph', null, schema.text('Reading list')),
  ]) });
  let nodeView;
  // Fixed transaction times force the setup and edit inside the history delay.
  // No sleeps or wall-clock luck: this reproduces the browser failure every run.
  let time = 1000;
  const dispatch = (tr) => { state = state.apply(tr.setTime(time++)); nodeView?.update(state.doc.firstChild); };
  dispatch(state.tr.insertText('block id edit', state.doc.content.size - 1));
  assert.equal(undoDepth(state), 1, 'Keep the preceding real edit in history');
  time = 2000;
  if (isolated) {
    dispatch(fixtureExports.equationFixtureTransaction(state, bid));
    assert.equal(undoDepth(state), 1, 'Fixture setup must not add an undo event');
  } else {
    // TipTap setContent replaces the whole document in one ordinary transaction.
    dispatch(state.tr.replaceWith(0, state.doc.content.size, [
      schema.node('mathBlock', { latex: 'x^2', bid }), schema.node('paragraph'),
    ]));
  }
  const editor = { isEditable: true, get state() { return state; }, view: { dispatch } };
  nodeView = create({ node: state.doc.firstChild, getPos: () => 0, editor });
  nodeView.dom.handlers.click();
  const area = elements.find((el) => Object.hasOwn(el, 'rows'));
  assert(area, 'The real equation editor must open');
  assert.equal(area.value, 'x^2');
  area.value = exact;
  area.handlers.input();
  const current = () => ({ ...state.doc.firstChild.attrs });
  assert.deepEqual(current(), { latex: exact, bid });
  assert(undo(state, dispatch));
  assert.deepEqual(current(), isolated ? { latex: 'x^2', bid } : original);
  assert(redo(state, dispatch));
  assert.deepEqual(current(), { latex: exact, bid });
  console.log(isolated
    ? 'PASS equation_node_view_edit_undo_redo_keeps_id_and_exact_text (isolated fixture)'
    : 'PASS reproduces_setContent_and_equation_edit_sharing_one_undo_event');
}
