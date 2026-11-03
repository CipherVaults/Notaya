// Quick check of the mock database engine without a browser (development only):
//   node --experimental-strip-types dev-mock/database-mock-check.mts
(globalThis as unknown as { location: unknown }).location = { search: "?db=1" };
const { databaseCommand, databaseMetas } = await import("./database.ts");
let fail = 0;
const is = (name: string, got: unknown, want: unknown) => {
  if (JSON.stringify(got) !== JSON.stringify(want)) { fail++; console.log("FAIL", name, JSON.stringify(got), "!==", JSON.stringify(want)); } else console.log("ok  ", name);
};
const call = (cmd: string, args: Record<string, unknown> = {}) => databaseCommand(cmd, args, {}) as any;

const metas = databaseMetas();
is("two databases", metas.map((m) => m.title).sort(), ["Projects", "Tasks"]);
const tasks = metas.find((m) => m.title === "Tasks")!;
const db = call("get_database", { id: tasks.id });
is("tasks rows", tasks.rows, 25);
const table = db.views[0];
let res = call("query_view", { request: { databaseId: tasks.id, viewId: table.id } });
is("all rows", res.rows.length, 25);
const prio = db.props.find((p: any) => p.name === "Priority");
const high = prio.options.find((o: any) => o.name === "High").id;
const v2 = { ...table, filter: { op: "and", conds: [{ prop: prio.id, op: "is", value: "", value2: "", values: [high], rel: "" }], groups: [] } };
res = call("query_view", { request: { databaseId: tasks.id, viewId: table.id, view: v2 } });
is("filter high", res.filtered, res.rows.filter((r: any) => r.values[prio.id] === high).length);
const score = db.props.find((p: any) => p.name === "Score");
const r0 = res.rows[0];
const est = db.props.find((p: any) => p.name === "Estimate");
is("formula", r0.values[score.id].v, (r0.values[est.id] as number) * 10 + 20);
const status = db.props.find((p: any) => p.name === "Status");
const v3 = { ...table, group: { prop: status.id, sort: "manual", dateBy: "", numberStep: 0, hideEmpty: false, hidden: [], collapsed: [] }, sorts: [{ prop: est.id, descending: true }] };
res = call("query_view", { request: { databaseId: tasks.id, viewId: table.id, view: v3 } });
is("group count", res.groups.filter((g: any) => g.key !== "").length, 4);
is("group sum", res.groups.reduce((a: number, g: any) => a + g.count, 0), 25);
is("footer sum", res.calcs[est.id], String(res.rows.reduce((a: number, r: any) => a + r.values[est.id], 0)));
const created = call("create_row", { databaseId: tasks.id, title: "New one", cells: JSON.stringify({ [est.id]: 4 }), templateId: "" });
is("created number", created.row.number, 26);
let threw = false;
try { call("set_cells", { rowId: created.row.id, title: null, patch: JSON.stringify({ [status.id]: "nope" }) }); } catch { threw = true; }
is("bad cell rejected", threw, true);
const proj = call("get_database", { id: metas.find((m) => m.title === "Projects")!.id });
const roll = proj.props.find((p: any) => p.kind === "rollup");
const pres = call("query_view", { request: { databaseId: proj.id, viewId: proj.views[0].id } });
is("rollup text", typeof pres.rows[0].values[roll.id].text, "string");
const chart = call("chart_data", { request: { databaseId: tasks.id, viewId: db.views.find((v: any) => v.kind === "chart").id } });
is("chart labels", chart.labels.length > 0, true);
console.log(fail ? `${fail} FAILED` : "ALL OK");
process.exit(fail ? 1 : 0);
