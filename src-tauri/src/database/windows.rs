//! Session-owned snapshots for the table IPC. The cache has no storage path.

use std::collections::{BTreeMap, VecDeque};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use zeroize::Zeroize;

use super::engine::{Ctx, GroupOut, RowOut, ViewResult};
use super::types::{Database, PropKind, View, ViewKind, MAX_ROWS_PER_DATABASE};
use crate::model::{new_id, Item};

const MAX_CACHED_VIEWS: usize = 8;
const MAX_CACHED_ROWS: usize = 2 * MAX_ROWS_PER_DATABASE;
pub const MAX_WINDOW_ROWS: usize = 256;

#[derive(Default)]
pub struct QueryCache {
    revision: u64,
    entries: VecDeque<CachedView>,
}

pub struct CachedView {
    database_id: String,
    view_id: String,
    settings: View,
    revision: u64,
    clock: (i64, i64),
    generation: String,
    result: ViewResult,
}

impl QueryCache {
    pub fn invalidate(&mut self) {
        while let Some(mut entry) = self.entries.pop_front() {
            entry.zeroize();
        }
        self.revision = self.revision.wrapping_add(1);
    }

    pub fn query<'a>(
        &mut self,
        items: impl Iterator<Item = &'a Item>,
        db: &Database,
        view_id: &str,
        view: View,
        clock: (i64, i64),
    ) -> &CachedView {
        // Normalize to settings that affect engine output. Collapse, widths and
        // other presentation settings can change without changing a generation.
        let settings = View {
            filter: view.filter,
            sorts: view.sorts,
            group: view.group.map(|mut g| { g.collapsed.clear(); g }),
            sub_group: view.sub_group.map(|mut g| { g.collapsed.clear(); g }),
            calcs: view.calcs,
            ..View::default()
        };
        if let Some(index) = self.entries.iter().position(|entry| {
            entry.database_id == db.id && entry.view_id == view_id && entry.settings == settings
                && entry.revision == self.revision && entry.clock == clock
        }) {
            return self.promote(index);
        }
        let ctx = Ctx::new(items, clock.0, clock.1);
        let result = ctx.run_view(ctx.dbs.get(db.id.as_str()).copied().unwrap_or(db), &settings);
        // Keep other inline tables warm, with a vault-wide row budget.
        let mut cached_rows: usize = self.entries.iter().map(|entry| entry.result.rows.len()).sum();
        while self.entries.len() >= MAX_CACHED_VIEWS || cached_rows + result.rows.len() > MAX_CACHED_ROWS {
            if let Some(entry) = self.entries.pop_front() {
                cached_rows -= entry.result.rows.len();
            }
        }
        self.entries.push_back(CachedView {
            database_id: db.id.clone(),
            view_id: view_id.into(),
            settings,
            revision: self.revision,
            clock,
            generation: new_id(),
            result,
        });
        self.entries.back().unwrap()
    }

    pub fn get(&mut self, database_id: &str, view_id: &str, generation: &str) -> Option<&CachedView> {
        let index = self.entries.iter().position(|entry| {
            entry.database_id == database_id && entry.view_id == view_id
                && entry.revision == self.revision && entry.generation == generation
        })?;
        Some(self.promote(index))
    }

    fn promote(&mut self, index: usize) -> &CachedView {
        let entry = self.entries.remove(index).unwrap();
        self.entries.push_back(entry);
        self.entries.back().unwrap()
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineWindowMeta {
    pub min_day: Option<i64>,
    pub max_day: Option<i64>,
    pub unscheduled_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_parents: Option<BTreeMap<String, Vec<String>>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewWindow {
    pub row_ids: Vec<String>,
    pub total: usize,
    pub filtered: usize,
    pub truncated: bool,
    pub groups: Option<Vec<GroupOut>>,
    pub calcs: Map<String, Value>,
    pub generation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matching_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeline: Option<TimelineWindowMeta>,
}

#[derive(Clone, Copy, Deserialize)]
pub struct RowRange {
    pub start: usize,
    pub count: usize,
}

impl RowRange {
    pub fn valid(ranges: &[Self]) -> bool {
        ranges.len() <= MAX_WINDOW_ROWS
            && ranges.iter().try_fold(0usize, |n, r| n.checked_add(r.count))
                .is_some_and(|n| n <= MAX_WINDOW_ROWS)
    }
}

impl CachedView {
    pub fn metadata(&self, db: &Database, find: &str) -> ViewWindow {
        let r = &self.result;
        let find = trim_js_whitespace(find).to_lowercase();
        ViewWindow {
            row_ids: r.rows.iter().map(|row| row.id.clone()).collect(),
            total: r.total,
            filtered: r.filtered,
            truncated: r.truncated,
            groups: r.groups.clone(),
            calcs: r.calcs.clone(),
            generation: self.generation.clone(),
            matching_ids: (!find.is_empty()).then(|| r.rows.iter()
                .filter(|row| row_matches(db, row, &find))
                .map(|row| row.id.clone())
                .collect()),
            timeline: None,
        }
    }

    pub fn metadata_for_view(&self, db: &Database, view: &View, find: &str) -> ViewWindow {
        let mut window = self.metadata(db, find);
        if view.kind == ViewKind::Timeline {
            if let Some(dp) = db.prop(&view.date_prop).filter(|p| p.kind == PropKind::Date) {
                let end_prop = (!view.end_prop.is_empty() && view.end_prop != dp.id)
                    .then(|| db.prop(&view.end_prop).filter(|p| p.kind == PropKind::Date))
                    .flatten();
                let find = trim_js_whitespace(find).to_lowercase();
                let sub_prop = (view.show_sub_items && !db.sub_parent_prop.is_empty()).then_some(db.sub_parent_prop.as_str());
                let mut sub_parents = sub_prop.map(|_| BTreeMap::new());
                let mut min_day: Option<i64> = None;
                let mut max_day: Option<i64> = None;
                let mut unscheduled_ids = Vec::new();
                for row in &self.result.rows {
                    if !find.is_empty() && !row_matches(db, row, &find) {
                        continue;
                    }
                    if let (Some(prop_id), Some(map)) = (sub_prop, sub_parents.as_mut()) {
                        if let Some(arr) = row.values.get(prop_id).and_then(Value::as_array) {
                            let ids: Vec<String> = arr.iter()
                                .filter_map(Value::as_str)
                                .filter(|id| *id != row.id)
                                .map(str::to_owned)
                                .collect();
                            if !ids.is_empty() {
                                map.insert(row.id.clone(), ids);
                            }
                        }
                    }
                    let Some((start, mut end)) = row.values.get(&dp.id).and_then(timeline_span) else {
                        unscheduled_ids.push(row.id.clone());
                        continue;
                    };
                    if let Some(ep) = end_prop {
                        if let Some((_, b_end)) = row.values.get(&ep.id).and_then(timeline_span) {
                            end = b_end;
                        }
                    }
                    let end = end.max(start);
                    min_day = Some(min_day.map_or(start, |m| m.min(start)));
                    max_day = Some(max_day.map_or(end, |m| m.max(end)));
                }
                window.timeline = Some(TimelineWindowMeta { min_day, max_day, unscheduled_ids, sub_parents });
            }
        }
        window
    }

    pub fn rows(&self, ranges: &[RowRange]) -> Vec<RowOut> {
        ranges.iter().flat_map(|range| {
            let start = range.start.min(self.result.rows.len());
            let end = start.saturating_add(range.count).min(self.result.rows.len());
            self.result.rows[start..end].iter().cloned()
        }).collect()
    }

    /// Recreate the uncached query with the snapshot's clock. No vault values
    /// appear in a failure message, including on serialization failures.
    #[cfg(any(debug_assertions, test))]
    pub fn check_shadow<'a>(&self, items: impl Iterator<Item = &'a Item>, db: &Database) {
        let ctx = Ctx::new(items, self.clock.0, self.clock.1);
        let current = ctx.run_view(ctx.dbs.get(db.id.as_str()).copied().unwrap_or(db), &self.settings);
        let full = serde_json::to_vec(&current).expect("could not encode uncached view");
        let cached = serde_json::to_vec(&self.result).expect("could not encode cached view");
        assert!(full == cached, "cached database view differs from full computation");
    }
}

impl Zeroize for CachedView {
    fn zeroize(&mut self) {
        self.database_id.zeroize();
        self.view_id.zeroize();
        self.settings.zeroize();
        self.revision.zeroize();
        self.clock.0.zeroize();
        self.clock.1.zeroize();
        self.generation.zeroize();
        self.result.zeroize();
    }
}

impl Drop for CachedView {
    fn drop(&mut self) {
        self.zeroize();
    }
}

impl Zeroize for QueryCache {
    fn zeroize(&mut self) {
        self.revision.zeroize();
        while let Some(mut entry) = self.entries.pop_front() {
            entry.zeroize();
        }
    }
}

impl Drop for QueryCache {
    fn drop(&mut self) {
        self.zeroize();
    }
}

fn parse_timeline_day(s: &str) -> Option<i64> {
    super::value::parse_date(s).map(|d| d.start.div_euclid(super::value::MINUTES_PER_DAY))
}

fn timeline_span(v: &Value) -> Option<(i64, i64)> {
    let o = v.as_object()?;
    let start = o.get("start").and_then(Value::as_str).and_then(parse_timeline_day)?;
    let end = o.get("end").and_then(Value::as_str).and_then(parse_timeline_day)
        .filter(|&e| e >= start)
        .unwrap_or(start);
    Some((start, end))
}

// Same fields as the table toolbar's cellText search, including both a select
// option's stored id and its displayed name.
fn trim_js_whitespace(s: &str) -> &str {
    s.trim_matches(|c: char| c.is_whitespace() && c != '\u{0085}')
}

fn row_matches(db: &Database, row: &RowOut, find: &str) -> bool {
    let has = |s: &str| s.to_lowercase().contains(find);
    has(&row.title) || db.props.iter().filter(|p| p.kind != PropKind::Title).any(|p| {
        let Some(value) = row.values.get(&p.id) else { return false };
        match value {
            Value::String(s) => has(s) || matches!(p.kind, PropKind::Select | PropKind::Status)
                && p.option(s).is_some_and(|option| has(&option.name)),
            Value::Bool(b) => has(if *b { "Yes" } else { "No" }),
            Value::Number(n) => n.as_f64().is_some_and(|n| {
                let exact = row.number_text.get(&p.id);
                if exact.is_some_and(|text| has(text)) {
                    return true;
                }
                let legacy = super::value::format_table_number(n, &p.number_format);
                (exact.is_none() || p.number_format == "numberWithCommas" && legacy.contains('e'))
                    && has(&legacy)
            }),
            Value::Object(o) if o.contains_key("t") => o.get("text").and_then(Value::as_str).is_some_and(has),
            _ => false,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::engine::zeroize_json_value;
    use crate::database::types::{ColCalc, GroupBy, PropKind, Property, Row, ViewKind, TITLE_PROP};
    use serde_json::json;

    fn sample_db(id: &str, view_id: &str) -> Database {
        let mut db = Database::new(id.into(), "sec".into(), "DB".into(), 0);
        db.props.push(Property {
            id: "n".into(),
            name: "Num".into(),
            kind: PropKind::Number,
            number_format: "numberWithCommas".into(),
            ..Property::default()
        });
        db.views = vec![View { id: view_id.into(), name: "Table".into(), kind: ViewKind::Table, ..View::default() }];
        db
    }

    fn sample_rows(db_id: &str, count: usize) -> Vec<Item> {
        (0..count)
            .map(|i| {
                let mut row = Row::new(format!("r{i:04}"), db_id, format!("Secret Row {i}"), 100);
                row.number = i as i64 + 1;
                row.order = i as i64;
                row.cells = if i == 0 { "{\"n\":0.0000001}".into() } else { format!("{{\"n\":{i}}}") };
                Item::Row(row)
            })
            .collect()
    }

    #[test]
    fn row_range_validates_hostile_counts_overflow_and_bounds() {
        assert!(RowRange::valid(&[]));
        assert!(RowRange::valid(&[RowRange { start: 0, count: MAX_WINDOW_ROWS }]));
        assert!(!RowRange::valid(&[RowRange { start: 0, count: MAX_WINDOW_ROWS + 1 }]));
        assert!(!RowRange::valid(&[
            RowRange { start: 0, count: MAX_WINDOW_ROWS },
            RowRange { start: 0, count: 1 },
        ]));
        assert!(!RowRange::valid(&[
            RowRange { start: 0, count: usize::MAX },
            RowRange { start: 10, count: 1 },
        ]));
        let many: Vec<RowRange> = (0..=MAX_WINDOW_ROWS).map(|i| RowRange { start: i, count: 0 }).collect();
        assert!(!RowRange::valid(&many));

        let mut cache = QueryCache::default();
        let db = sample_db("db1", "v1");
        let mut items = vec![Item::Database(db.clone())];
        items.extend(sample_rows("db1", 5));
        let entry = cache.query(items.iter(), &db, "v1", db.views[0].clone(), (1000, 0));

        let clamped = entry.rows(&[
            RowRange { start: usize::MAX, count: 10 },
            RowRange { start: 3, count: usize::MAX },
            RowRange { start: 0, count: 2 },
        ]);
        let ids: Vec<&str> = clamped.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(ids, vec!["r0003", "r0004", "r0000", "r0001"]);
    }

    #[test]
    fn cache_normalizes_collapsed_groups_evicts_lru_and_checks_clock() {
        let mut cache = QueryCache::default();
        let db = sample_db("db1", "v1");
        let mut items = vec![Item::Database(db.clone())];
        items.extend(sample_rows("db1", 3));

        let mut view_a = db.views[0].clone();
        view_a.group = Some(GroupBy { prop: TITLE_PROP.into(), collapsed: vec!["a".into()], ..GroupBy::default() });
        let gen_a = cache.query(items.iter(), &db, "v1", view_a, (1000, 0)).generation.clone();

        let mut view_collapsed = db.views[0].clone();
        view_collapsed.group = Some(GroupBy { prop: TITLE_PROP.into(), collapsed: vec!["b".into()], ..GroupBy::default() });
        let gen_b = cache.query(items.iter(), &db, "v1", view_collapsed.clone(), (1000, 0)).generation.clone();
        assert_eq!(gen_a, gen_b, "presentation-only collapse state must reuse snapshot");

        let gen_clock = cache.query(items.iter(), &db, "v1", view_collapsed.clone(), (1001, 0)).generation.clone();
        assert_ne!(gen_a, gen_clock, "clock change must create a fresh generation");

        view_collapsed.calcs.push(ColCalc { prop: "n".into(), calc: "sum".into() });
        let gen_calc = cache.query(items.iter(), &db, "v1", view_collapsed, (1001, 0)).generation.clone();
        assert_ne!(gen_clock, gen_calc, "engine-relevant calc change must create a fresh generation");

        let mut cache = QueryCache::default();
        let mut tokens = Vec::new();
        for i in 0..=MAX_CACHED_VIEWS {
            let db_i = sample_db(&format!("db_{i}"), "v1");
            let mut db_items = vec![Item::Database(db_i.clone())];
            db_items.extend(sample_rows(&db_i.id, 1));
            let token = cache.query(db_items.iter(), &db_i, "v1", db_i.views[0].clone(), (1000, 0)).generation.clone();
            tokens.push((db_i.id.clone(), token));
        }
        assert_eq!(cache.entries.len(), MAX_CACHED_VIEWS);
        assert!(cache.get(&tokens[0].0, "v1", &tokens[0].1).is_none(), "oldest view evicted");
        assert!(cache.get(&tokens[1].0, "v1", &tokens[1].1).is_some(), "newer view kept");
    }

    #[test]
    fn zeroize_wipes_json_cache_and_entries_on_invalidate() {
        let mut val = json!({
            "secretKey": "secretValue",
            "nested": ["alpha", {"inner": "beta"}]
        });
        zeroize_json_value(&mut val);
        assert!(val.is_null());

        let mut row = RowOut {
            id: "row-1".into(),
            title: "Confidential Title".into(),
            icon: "lock".into(),
            number: 42,
            created: 100,
            updated: 200,
            favorite: true,
            values: serde_json::from_str(r#"{"p":"sensitive cell"}"#).unwrap(),
            number_text: BTreeMap::from([("p".into(), "$123.45".into())]),
            number_input: BTreeMap::from([("p".into(), "123.4500".into())]),
        };
        row.zeroize();
        assert!(row.id.is_empty() && row.title.is_empty() && row.icon.is_empty());
        assert_eq!((row.number, row.created, row.updated, row.favorite), (0, 0, 0, false));
        assert!(row.values.is_empty() && row.number_text.is_empty() && row.number_input.is_empty());

        let mut cache = QueryCache::default();
        let db = sample_db("db1", "v1");
        let mut items = vec![Item::Database(db.clone())];
        items.extend(sample_rows("db1", 2));
        let token = cache.query(items.iter(), &db, "v1", db.views[0].clone(), (1000, 0)).generation.clone();
        assert!(cache.get("db1", "v1", &token).is_some());
        // U+0085 is not JS whitespace; it must not collapse to an empty Find query.
        assert_eq!(cache.get("db1", "v1", &token).unwrap().metadata(&db, "\u{0085}").matching_ids, Some(Vec::new()));
        cache.invalidate();
        assert!(cache.entries.is_empty());
        assert_eq!(cache.revision, 1);
        assert!(cache.get("db1", "v1", &token).is_none());
        cache.zeroize();
        assert_eq!(cache.revision, 0);
    }

    #[test]
    fn timeline_metadata_covers_entire_view_bounds_and_unscheduled_rows() {
        let mut db = sample_db("db-tl", "v-tl");
        db.props.push(Property { id: "due".into(), name: "Due".into(), kind: PropKind::Date, ..Property::default() });
        db.props.push(Property { id: "end".into(), name: "End".into(), kind: PropKind::Date, ..Property::default() });
        let tl_view = View {
            id: "v-tl".into(),
            name: "Timeline".into(),
            kind: ViewKind::Timeline,
            date_prop: "due".into(),
            end_prop: "end".into(),
            ..View::default()
        };
        db.views = vec![tl_view.clone()];
        let mut items = vec![Item::Database(db.clone())];
        let mut r0 = Row::new("r0".into(), "db-tl", "Alpha".into(), 100);
        r0.order = 0;
        r0.cells = r#"{"due":{"start":"2026-01-01"}}"#.into();
        let mut r1 = Row::new("r1".into(), "db-tl", "Future".into(), 100);
        r1.order = 1;
        r1.cells = r#"{"due":{"start":"2026-01-10"},"end":{"start":"2099-12-31"}}"#.into();
        let mut r2 = Row::new("r2".into(), "db-tl", "Tail undated".into(), 100);
        r2.order = 2;
        items.extend([Item::Row(r0), Item::Row(r1), Item::Row(r2)]);

        let mut cache = QueryCache::default();
        let entry = cache.query(items.iter(), &db, "v-tl", tl_view.clone(), (1000, 0));
        let meta = entry.metadata_for_view(&db, &tl_view, "");
        let tl = meta.timeline.expect("timeline metadata");
        assert_eq!(tl.min_day, parse_timeline_day("2026-01-01"));
        assert_eq!(tl.max_day, parse_timeline_day("2099-12-31"));
        assert_eq!(tl.unscheduled_ids, vec!["r2".to_string()]);

        let filtered = entry.metadata_for_view(&db, &tl_view, "undated");
        let tl_filtered = filtered.timeline.expect("filtered timeline metadata");
        assert_eq!((tl_filtered.min_day, tl_filtered.max_day), (None, None));
        assert_eq!(tl_filtered.unscheduled_ids, vec!["r2".to_string()]);
    }

    #[test]
    fn timeline_metadata_includes_sub_parents_across_dated_and_unscheduled_rows() {
        let mut db = sample_db("db-sub", "v-sub");
        db.props.push(Property { id: "due".into(), name: "Due".into(), kind: PropKind::Date, ..Property::default() });
        db.props.push(Property {
            id: "parent".into(),
            name: "Parent".into(),
            kind: PropKind::Relation,
            relation_db: "db-sub".into(),
            ..Property::default()
        });
        db.sub_parent_prop = "parent".into();
        let tl_off = View {
            id: "v-sub".into(),
            name: "Timeline".into(),
            kind: ViewKind::Timeline,
            date_prop: "due".into(),
            show_sub_items: false,
            ..View::default()
        };
        let tl_on = View { show_sub_items: true, ..tl_off.clone() };
        db.views = vec![tl_on.clone()];

        let mut items = vec![Item::Database(db.clone())];
        let mut r0 = Row::new("r0".into(), "db-sub", "Parent Row".into(), 100);
        r0.order = 0;
        r0.cells = r#"{"due":{"start":"2026-01-01"},"parent":["r0"]}"#.into();
        let mut r1 = Row::new("r1".into(), "db-sub", "Dated Child".into(), 100);
        r1.order = 1;
        r1.cells = r#"{"due":{"start":"2026-01-05"},"parent":["r0"]}"#.into();
        let mut r2 = Row::new("r2".into(), "db-sub", "Unscheduled Child".into(), 100);
        r2.order = 2;
        r2.cells = r#"{"parent":["r0"]}"#.into();
        items.extend([Item::Row(r0), Item::Row(r1), Item::Row(r2)]);

        let mut cache = QueryCache::default();
        let entry = cache.query(items.iter(), &db, "v-sub", tl_on.clone(), (1000, 0));

        let meta_off = entry.metadata_for_view(&db, &tl_off, "");
        assert!(meta_off.timeline.expect("timeline metadata").sub_parents.is_none());

        let meta_on = entry.metadata_for_view(&db, &tl_on, "");
        let tl_meta = meta_on.timeline.expect("timeline metadata");
        let parents = tl_meta.sub_parents.expect("sub_parents map");
        assert!(!parents.contains_key("r0"), "self-links should be excluded");
        assert_eq!(parents.get("r1"), Some(&vec!["r0".to_string()]));
        assert_eq!(parents.get("r2"), Some(&vec!["r0".to_string()]));

        let filtered = entry.metadata_for_view(&db, &tl_on, "Dated");
        let filtered_parents = filtered.timeline.expect("filtered timeline").sub_parents.expect("filtered sub_parents");
        assert_eq!(filtered_parents.len(), 1);
        assert_eq!(filtered_parents.get("r1"), Some(&vec!["r0".to_string()]));
    }
}
