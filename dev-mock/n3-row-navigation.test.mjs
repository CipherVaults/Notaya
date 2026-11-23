import assert from "node:assert/strict";
import { before, after, test } from "node:test";
import { fileURLToPath } from "node:url";
import react from "@vitejs/plugin-react";
import { renderToStaticMarkup } from "react-dom/server";
import { createServer } from "vite";

const row = { sectionId: "work", title: "Imported row", icon: "", isRow: true };
const child = { id: "child", sectionId: "work", parent: "row", title: "Folder-owned page", icon: "", trashedAt: null };
const grandchild = { ...child, id: "grandchild", parent: "child", title: "Grandchild" };
const siblingChild = { ...child, id: "sibling-child", parent: "other-row", title: "Sibling's page" };
const origin = { id: "origin", sectionId: "home", parent: "", trashedAt: null };
const tree = {
  notebooks: [{ id: "personal" }, { id: "business" }],
  sections: [{ id: "home", notebookId: "personal" }, { id: "work", notebookId: "business" }],
  pages: [origin],
  rowPages: [child, grandchild, siblingChild],
  databases: [],
};

let server, useApp, applySectionBin, goToPage, history, RowChildPages;
before(async () => {
  globalThis.window = {
    __TAURI_INTERNALS__: {
      transformCallback: () => 1,
      async invoke(command, args) {
        if (command === "plugin:event|listen") return 1;
        assert.equal(command, "get_page_link_target");
        assert.equal(args.id, "row");
        return row;
      },
    },
  };
  server = await createServer({
    configFile: false,
    plugins: [react()],
    resolve: { alias: { "@": fileURLToPath(new URL("../src", import.meta.url)) } },
    optimizeDeps: { noDiscovery: true },
    server: { middlewareMode: true, ws: false, watch: null },
  });
  ({ useApp, applySectionBin } = await server.ssrLoadModule("/src/store/app.ts"));
  ({ goToPage } = await server.ssrLoadModule("/src/features/notes/editor/internalLinks.ts"));
  history = await server.ssrLoadModule("/src/lib/navHistory.ts");
  ({ default: RowChildPages } = await server.ssrLoadModule("/src/features/notes/RowChildPages.tsx"));
});

after(async () => {
  await server?.close();
  delete globalThis.window;
});

const location = () => {
  const { activeNotebookId, activeSectionId, activeKind, activePageId } = useApp.getState();
  return [activeNotebookId, activeSectionId, activeKind, activePageId];
};

test("a cross-section row link makes one Back and Forward step", async () => {
  useApp.setState({ tree, status: { unlocked: true } });
  useApp.getState().openPage(origin.id);
  const stop = history.startNavHistory();
  try {
    await goToPage("row");
    assert.deepEqual(location(), ["business", "work", "row", "row"]);
    history.goBack();
    assert.deepEqual(location(), ["personal", "home", "page", "origin"]);
    assert.equal(history.useNav.getState().canBack, false);
    history.goForward();
    assert.deepEqual(location(), ["business", "work", "row", "row"]);
  } finally {
    stop();
  }
});

test("a row's child button opens folder-owned content and history restores it", async () => {
  useApp.setState({ tree, status: { unlocked: true } });
  useApp.getState().openRow("row", row.sectionId);
  const stop = history.startNavHistory();
  try {
    const pages = [...tree.rowPages, { ...child, id: "trashed", title: "Trashed child", trashedAt: 1 }];
    const list = RowChildPages({ parentId: "row", pages, onOpen: useApp.getState().openPage });
    const html = renderToStaticMarkup(list);
    assert.match(html, /<button[^>]*>.*Folder-owned page/);
    assert.doesNotMatch(html, /Grandchild|Sibling's page|Trashed child/);
    assert.deepEqual(useApp.getState().tree.pages, [origin]);

    const [directChild] = list.props.children;
    directChild.props.children.props.onClick();
    assert.deepEqual(location(), ["business", "work", "page", "child"]);
    history.goBack();
    assert.deepEqual(location(), ["business", "work", "row", "row"]);
    history.goForward();
    assert.deepEqual(location(), ["business", "work", "page", "child"]);

    history.goBack();
    useApp.setState({ tree: { ...tree, rowPages: [siblingChild] } });
    history.goForward();
    assert.deepEqual(location(), ["business", "work", "row", "row"]);
    assert.equal(history.useNav.getState().canForward, false);
  } finally {
    stop();
  }
});

test("sidebar binning keeps an open row child unless its own section is binned", () => {
  useApp.setState({ tree });
  useApp.getState().openPage(grandchild.id);
  assert.deepEqual(location(), ["business", "work", "page", "grandchild"]);
  // Clicking a sidebar section selects it without closing the open page.
  useApp.setState({ activeNotebookId: "personal", activeSectionId: "home" });

  const withoutHome = { ...tree, sections: [tree.sections[1]], pages: [] };
  applySectionBin(withoutHome);
  assert.equal(useApp.getState().tree, withoutHome);
  assert.deepEqual(location(), ["business", "work", "page", "grandchild"]);

  applySectionBin({ ...withoutHome, sections: [], rowPages: [] });
  assert.equal(useApp.getState().activePageId, null);
});

test("binning a row child's former section keeps the page under a moved database open", () => {
  const movedChild = { ...child, sectionId: "home" };
  const movedTree = { ...tree, rowPages: [movedChild] };
  useApp.setState({ tree: movedTree });
  useApp.getState().openPage(movedChild.id);

  applySectionBin({ ...movedTree, sections: [tree.sections[1]], pages: [] });
  assert.equal(useApp.getState().activePageId, movedChild.id);
  assert.equal(useApp.getState().activeKind, "page");
});
