//! The stored shape of a database (A35). A database is one vault record (`Item::Database`) that
//! holds the schema (properties), the saved views and the templates. Its rows are ordinary pages
//! (`Page` with `db` set and `props` holding the cell values as JSON text), so rows open as full
//! pages, can have sub-pages and are found by search like any page.
//!
//! Everything here is plain data with serde defaults, so a field added later never breaks an
//! older record. `sanitize` clamps what the UI sends.

use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::error::{Error, Result};

pub const MAX_PROPERTIES: usize = 100;
pub const MAX_VIEWS: usize = 50;
pub const MAX_TEMPLATES: usize = 100;
pub const MAX_OPTIONS: usize = 500;
pub const MAX_NAME: usize = 120;
/// Accept short legacy identifiers as well as generated ids, without allowing
/// object-prototype keys or unbounded identifiers across the JSON boundary.
pub const MAX_ID_BYTES: usize = 128;

fn check_id(id: &str) -> Result<()> {
    if id.len() > MAX_ID_BYTES || matches!(id, "__proto__" | "prototype" | "constructor") {
        return Err(Error::invalid("Invalid database identifier."));
    }
    Ok(())
}
pub const MAX_FORMULA: usize = 20_000;
pub const MAX_AUTOMATIONS: usize = 50;
pub const MAX_ACTIONS: usize = 20;
pub const MAX_FILTER_DEPTH: usize = 3;
pub const MAX_FILTER_CONDS: usize = 100;
pub const MAX_SORTS: usize = 10;
/// The id of the title property of every database (its value is the row page's title).
pub const TITLE_PROP: &str = "title";
/// Files and media stored in a row: at most this many bytes each (owner, 2026-11-03: 50 MB; 2026-11-04: 2 GB, which one stored record cannot hold, so 900 MB).
pub const MAX_FILE_BYTES: usize = 900 * 1024 * 1024;

/// The kinds of property (Notion's list).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Zeroize, Default)]
#[serde(rename_all = "camelCase")]
pub enum PropKind {
    #[default]
    Text,
    Title,
    Number,
    Select,
    MultiSelect,
    Status,
    Date,
    Person,
    Files,
    Checkbox,
    Url,
    Email,
    Phone,
    Formula,
    Relation,
    Rollup,
    CreatedTime,
    CreatedBy,
    LastEditedTime,
    LastEditedBy,
    Id,
    Place,
    /// Runs its `actions` on the row when pressed (no value is stored).
    Button,
}

impl PropKind {
    /// Kinds whose value is worked out, never stored in the row.
    pub fn is_computed(self) -> bool {
        matches!(
            self,
            PropKind::Formula
                | PropKind::Rollup
                | PropKind::CreatedTime
                | PropKind::CreatedBy
                | PropKind::LastEditedTime
                | PropKind::LastEditedBy
                | PropKind::Button
        )
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Zeroize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct SelectOption {
    pub id: String,
    pub name: String,
    /// A palette name ("gray", "brown", "orange", "yellow", "green", "blue", "purple", "pink", "red") or "".
    pub color: String,
    /// Status only: "todo" | "inProgress" | "complete".
    pub group: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Zeroize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Property {
    pub id: String,
    pub name: String,
    pub kind: PropKind,
    /// Select, multi-select and status.
    pub options: Vec<SelectOption>,
    /// Number: "number" | "numberWithCommas" | "percent" | "dollar" | "euro" | "pound" | "yen" | "rupee" | "won" | "yuan" | "rial".
    pub number_format: String,
    /// Number shown as a bar or ring: "" | "bar" | "ring", filled up to `number_max`.
    pub number_display: String,
    pub number_max: f64,
    /// Date: "full" | "short" | "dmy" | "mdy" | "ymd" | "relative".
    pub date_format: String,
    /// Formula text.
    pub formula: String,
    /// Relation: the id of the database it points to, and the id of the property in that database
    /// that shows the other direction ("" = one-way).
    pub relation_db: String,
    pub relation_back: String,
    /// Relation: only one linked row is allowed.
    pub relation_limit_one: bool,
    /// Rollup: the relation property of this database, the property read in the related database
    /// and the calculation applied ("showOriginal", "count", "sum", ...).
    pub rollup_relation: String,
    pub rollup_prop: String,
    pub rollup_calc: String,
    /// ID property: a prefix such as "TASK" (rows show TASK-1, TASK-2, ...).
    pub id_prefix: String,
    /// Optional unique-looking description shown as a tooltip.
    pub description: String,
    /// Button: the label on the button and what it does to the row (in order).
    pub button_label: String,
    pub actions: Vec<Action>,
}

/// One value to set when an action adds a row.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Zeroize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct SetCell {
    pub prop: String,
    pub value: String,
}

/// What a button or an automation does. Everything stays inside the vault: no network, no programs.
/// `kind`: "setProp" (set `prop` of the row to `value`) | "addRow" (add a row to `database`, "" = this one,
/// named `title`, with `cells`) | "notify" (show `title` inside Notaya) | "trash" (move the row to the trash).
/// A `value` is text for the property, or "@clear", "@now", "@today", "@today+7", "@toggle", "@add:5",
/// "@self" (link to the row that triggered it); `{title}` in a text stands for that row's title.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Zeroize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Action {
    pub kind: String,
    pub prop: String,
    pub value: String,
    pub database: String,
    pub title: String,
    pub cells: Vec<SetCell>,
}

/// `kind`: "rowAdded" | "propChanged" (`prop` = the property, "" = any) | "dateReached" (`prop` = a date
/// property, `offset_mins` = minutes after the date starts; negative = before).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Zeroize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Trigger {
    pub kind: String,
    pub prop: String,
    pub offset_mins: i64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Zeroize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Automation {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub trigger: Trigger,
    /// The row must match this filter for the actions to run.
    pub filter: Filter,
    pub actions: Vec<Action>,
    /// Date triggers: the local minute up to which dates were handled (set by Notaya).
    pub last_run: i64,
}

/// Conditions of a filter. `op` names (per property kind) are listed in `engine::filter`.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Zeroize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Cond {
    pub prop: String,
    pub op: String,
    pub value: String,
    pub value2: String,
    /// Several values (select options, relation rows, ...).
    pub values: Vec<String>,
    /// Date conditions: "exact" | "today" | "tomorrow" | "yesterday" | "oneWeekAgo" | "oneWeekFromNow" |
    /// "oneMonthAgo" | "oneMonthFromNow" | "thisWeek" | "pastWeek" | "pastMonth" | "nextWeek" | "nextMonth" | "pastYear" | "nextYear" | "thisMonth" | "thisYear".
    pub rel: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Zeroize)]
#[serde(rename_all = "camelCase", default)]
pub struct Filter {
    /// "and" | "or"
    pub op: String,
    pub conds: Vec<Cond>,
    pub groups: Vec<Filter>,
}

impl Default for Filter {
    fn default() -> Self {
        Filter { op: "and".into(), conds: Vec::new(), groups: Vec::new() }
    }
}

impl Filter {
    pub fn is_empty(&self) -> bool {
        self.conds.is_empty() && self.groups.iter().all(|g| g.is_empty())
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Zeroize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Sort {
    pub prop: String,
    pub descending: bool,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Zeroize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct GroupBy {
    pub prop: String,
    /// "manual" | "asc" | "desc" (alphabetical / by option order)
    pub sort: String,
    /// Date properties: "day" | "week" | "month" | "year" | "relative".
    pub date_by: String,
    /// Number properties: bucket size (0 = each value is a group).
    pub number_step: f64,
    pub hide_empty: bool,
    /// Ids of groups the person collapsed or hid.
    pub hidden: Vec<String>,
    pub collapsed: Vec<String>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Zeroize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Column {
    pub prop: String,
    pub visible: bool,
    pub width: u32,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Zeroize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ColCalc {
    pub prop: String,
    pub calc: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Zeroize, Default)]
#[serde(rename_all = "camelCase")]
pub enum ViewKind {
    #[default]
    Table,
    Board,
    Calendar,
    Timeline,
    List,
    Gallery,
    Chart,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Zeroize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ChartConfig {
    /// "bar" | "line" | "donut" | "number"
    pub kind: String,
    pub x_prop: String,
    /// Date x axis: "day" | "week" | "month" | "year"; numbers: bucket size is `x_step`.
    pub x_by: String,
    pub x_step: f64,
    pub x_sort: String,
    /// Optional second grouping (stacked / grouped bars, several lines).
    pub group_prop: String,
    /// "count" | "sum" | "avg" | "median" | "min" | "max" | "range" | "unique" | "countValues" | "percentNotEmpty" ...
    pub y_calc: String,
    pub y_prop: String,
    pub stacked: bool,
    pub horizontal: bool,
    pub show_labels: bool,
    pub hide_empty: bool,
    pub color: String,
    pub title: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Zeroize)]
#[serde(rename_all = "camelCase", default)]
pub struct View {
    pub id: String,
    pub name: String,
    pub kind: ViewKind,
    pub filter: Filter,
    pub sorts: Vec<Sort>,
    pub group: Option<GroupBy>,
    pub sub_group: Option<GroupBy>,
    pub columns: Vec<Column>,
    pub calcs: Vec<ColCalc>,
    /// Table: number of frozen columns (the title counts), wrapped cells, row numbers.
    pub freeze: u32,
    pub wrap: bool,
    pub row_numbers: bool,
    /// How a row opens: "full" | "side" | "center".
    pub open_in: String,
    /// Gallery / board cards.
    pub card_size: String,
    pub card_cover: String,
    pub card_fit: bool,
    /// Calendar and timeline.
    pub date_prop: String,
    pub end_prop: String,
    pub calendar_mode: String,
    pub timeline_zoom: String,
    pub show_deps: bool,
    pub show_sub_items: bool,
    /// Sub-items and dependencies are read from these relation properties (self-relations).
    pub chart: ChartConfig,
}

impl Default for View {
    fn default() -> Self {
        View {
            id: String::new(),
            name: "Table".into(),
            kind: ViewKind::Table,
            filter: Filter::default(),
            sorts: Vec::new(),
            group: None,
            sub_group: None,
            columns: Vec::new(),
            calcs: Vec::new(),
            freeze: 1,
            wrap: false,
            row_numbers: false,
            open_in: "full".into(),
            card_size: "medium".into(),
            card_cover: String::new(),
            card_fit: false,
            date_prop: String::new(),
            end_prop: String::new(),
            calendar_mode: "month".into(),
            timeline_zoom: "month".into(),
            show_deps: true,
            show_sub_items: true,
            chart: ChartConfig::default(),
        }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Zeroize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub icon: String,
    /// Cell values (JSON object text, property id -> value).
    pub cells: String,
    /// The page body (TipTap JSON text).
    pub content: String,
    pub text: String,
    /// "" | "daily" | "weekly" | "monthly" (a repeating template adds its row when the database is opened and one is due).
    pub repeat: String,
    pub repeat_last: i64,
}

#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(rename_all = "camelCase")]
pub struct Database {
    pub id: String,
    /// Where it sits in the tree (like a page). Inline databases keep the section of their page.
    pub section_id: String,
    /// Page (or database) this one is under; "" = directly in its section.
    #[serde(default)]
    pub parent: String,
    /// An inline database is shown inside a page and is not listed in the sidebar.
    #[serde(default)]
    pub inline: bool,
    pub title: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub order: i64,
    #[serde(default)]
    pub favorite: bool,
    #[serde(default)]
    pub locked: bool,
    #[serde(default)]
    pub props: Vec<Property>,
    #[serde(default)]
    pub views: Vec<View>,
    #[serde(default)]
    pub templates: Vec<Template>,
    #[serde(default)]
    pub default_template: String,
    /// The next number for the ID property.
    #[serde(default)]
    pub next_id: i64,
    /// Sub-items: the relation properties (this database to itself) for parent and children; "" = off.
    #[serde(default)]
    pub sub_parent_prop: String,
    #[serde(default)]
    pub sub_child_prop: String,
    /// Dependencies: "blocked by" and "blocking" relation properties; "" = off.
    #[serde(default)]
    pub blocked_by_prop: String,
    #[serde(default)]
    pub blocking_prop: String,
    #[serde(default)]
    pub automations: Vec<Automation>,
    pub trashed_at: Option<i64>,
    pub created: i64,
    pub updated: i64,
}

impl Database {
    pub fn prop(&self, id: &str) -> Option<&Property> {
        self.props.iter().find(|p| p.id == id)
    }
    pub fn prop_by_name(&self, name: &str) -> Option<&Property> {
        self.props.iter().find(|p| p.name == name).or_else(|| {
            let l = name.to_lowercase();
            self.props.iter().find(|p| p.name.to_lowercase() == l)
        })
    }
    pub fn view(&self, id: &str) -> Option<&View> {
        self.views.iter().find(|v| v.id == id)
    }
    pub fn title_prop(&self) -> &Property {
        self.props.iter().find(|p| p.kind == PropKind::Title).expect("a database always has a title property")
    }

    /// A new database: a title property, a Status-free starter (Tags), and one table view.
    pub fn new(id: String, section_id: String, title: String, now: i64) -> Database {
        let mut db = Database {
            id,
            section_id,
            parent: String::new(),
            inline: false,
            title,
            icon: String::new(),
            description: String::new(),
            order: 0,
            favorite: false,
            locked: false,
            props: vec![Property { id: TITLE_PROP.into(), name: "Name".into(), kind: PropKind::Title, ..Property::default() }],
            views: Vec::new(),
            templates: Vec::new(),
            default_template: String::new(),
            next_id: 1,
            sub_parent_prop: String::new(),
            sub_child_prop: String::new(),
            blocked_by_prop: String::new(),
            blocking_prop: String::new(),
            automations: Vec::new(),
            trashed_at: None,
            created: now,
            updated: now,
        };
        db.views.push(View { id: crate::model::new_id(), name: "Table".into(), ..View::default() });
        db
    }
}


/// One row of a database: a page with cell values. It opens as a full page (title, icon, cover and
/// body like any page), can hold sub-pages (pages whose `parent` is the row id) and is found by search.
#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(rename_all = "camelCase")]
pub struct Row {
    pub id: String,
    pub db: String,
    pub title: String,
    /// TipTap document as JSON text (the page body).
    #[serde(default)]
    pub content: String,
    /// Plain text of the body for search.
    #[serde(default)]
    pub text: String,
    /// Cell values, a JSON object text: property id -> value (the title is `title`, not here).
    #[serde(default)]
    pub cells: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub cover: String,
    #[serde(default)]
    pub order: i64,
    #[serde(default)]
    pub favorite: bool,
    /// The number of the ID property (0 = none).
    #[serde(default)]
    pub number: i64,
    /// Who made / last edited it (plain text; there are no accounts).
    #[serde(default)]
    pub created_by: String,
    #[serde(default)]
    pub edited_by: String,
    pub trashed_at: Option<i64>,
    pub created: i64,
    pub updated: i64,
}

impl Row {
    pub fn new(id: String, db: &str, title: String, now: i64) -> Row {
        Row {
            id,
            db: db.into(),
            title,
            content: String::new(),
            text: String::new(),
            cells: "{}".into(),
            icon: String::new(),
            cover: String::new(),
            order: 0,
            favorite: false,
            number: 0,
            created_by: String::new(),
            edited_by: String::new(),
            trashed_at: None,
            created: now,
            updated: now,
        }
    }
}

fn check_len(what: &str, s: &str, max: usize) -> Result<()> {
    if s.chars().count() > max {
        return Err(Error::Other(format!("{what} is too long (at most {max} characters).")));
    }
    Ok(())
}

fn check_filter(f: &Filter, depth: usize, count: &mut usize) -> Result<()> {
    if depth > MAX_FILTER_DEPTH {
        return Err(Error::Other("Filters can be nested three levels deep.".into()));
    }
    if f.op != "and" && f.op != "or" {
        return Err(Error::Other("A filter group must be 'and' or 'or'.".into()));
    }
    for c in &f.conds {
        *count += 1;
        if *count > MAX_FILTER_CONDS {
            return Err(Error::Other("Too many filter conditions.".into()));
        }
        check_len("A filter value", &c.value, 2000)?;
        check_len("A filter value", &c.value2, 2000)?;
        if c.values.len() > 1000 {
            return Err(Error::Other("Too many filter values.".into()));
        }
    }
    for g in &f.groups {
        check_filter(g, depth + 1, count)?;
    }
    Ok(())
}

impl Database {
    /// Validates what the UI sent and fixes small things (ids, the title property, view columns).
    /// Properties and views are matched by id; ids are made by the UI or the backend (`new_id`).
    pub fn sanitize(&mut self) -> Result<()> {
        check_id(&self.id)?;
        // Check supplied ids before any legacy defaults replace them.
        for p in &self.props {
            for id in [&p.id, &p.relation_db, &p.relation_back, &p.rollup_relation, &p.rollup_prop] {
                check_id(id)?;
            }
            for option in &p.options {
                check_id(&option.id)?;
            }
        }
        for view in &self.views {
            check_id(&view.id)?;
        }
        for template in &self.templates {
            check_id(&template.id)?;
        }
        for automation in &self.automations {
            check_id(&automation.id)?;
        }
        check_len("The database name", &self.title, MAX_NAME)?;
        check_len("The icon", &self.icon, 40)?;
        check_len("The description", &self.description, 2000)?;
        if self.props.len() > MAX_PROPERTIES {
            return Err(Error::Other(format!("A database can have at most {MAX_PROPERTIES} properties.")));
        }
        if self.views.len() > MAX_VIEWS {
            return Err(Error::Other(format!("A database can have at most {MAX_VIEWS} views.")));
        }
        if self.templates.len() > MAX_TEMPLATES {
            return Err(Error::Other(format!("A database can have at most {MAX_TEMPLATES} templates.")));
        }
        // One title property, always first, id "title".
        let titles = self.props.iter().filter(|p| p.kind == PropKind::Title).count();
        if titles == 0 {
            self.props.insert(0, Property { id: TITLE_PROP.into(), name: "Name".into(), kind: PropKind::Title, ..Property::default() });
        } else if titles > 1 {
            return Err(Error::Other("A database has exactly one title property.".into()));
        }
        let ti = self.props.iter().position(|p| p.kind == PropKind::Title).unwrap();
        if ti != 0 {
            let p = self.props.remove(ti);
            self.props.insert(0, p);
        }
        self.props[0].id = TITLE_PROP.into();
        let mut seen = std::collections::HashSet::new();
        for p in &mut self.props {
            if p.id.is_empty() {
                p.id = crate::model::new_id();
            }
            if !seen.insert(p.id.clone()) {
                return Err(Error::Other("Two properties share an id.".into()));
            }
            check_len("A property name", &p.name, MAX_NAME)?;
            if p.name.trim().is_empty() {
                p.name = "Untitled".into();
            }
            check_len("A formula", &p.formula, MAX_FORMULA)?;
            check_len("An ID prefix", &p.id_prefix, 20)?;
            if p.options.len() > MAX_OPTIONS {
                return Err(Error::Other("A property has too many options.".into()));
            }
            let mut oids = std::collections::HashSet::new();
            for o in &mut p.options {
                if o.id.is_empty() {
                    o.id = crate::model::new_id();
                }
                if !oids.insert(o.id.clone()) {
                    return Err(Error::Other("Two options share an id.".into()));
                }
                check_len("An option name", &o.name, MAX_NAME)?;
                if p.kind == PropKind::Status && !matches!(o.group.as_str(), "todo" | "inProgress" | "complete") {
                    o.group = "todo".into();
                }
                if p.kind != PropKind::Status {
                    o.group.clear();
                }
            }
            if !p.number_max.is_finite() {
                p.number_max = 0.0;
            }
            if !p.kind_has_options() {
                p.options.clear();
            }
        }
        // Views: ids, names, columns for every property, filters within limits.
        let prop_ids: Vec<String> = self.props.iter().map(|p| p.id.clone()).collect();
        if self.views.is_empty() {
            self.views.push(View { id: crate::model::new_id(), ..View::default() });
        }
        let mut vseen = std::collections::HashSet::new();
        for v in &mut self.views {
            if v.id.is_empty() {
                v.id = crate::model::new_id();
            }
            if !vseen.insert(v.id.clone()) {
                return Err(Error::Other("Two views share an id.".into()));
            }
            check_len("A view name", &v.name, MAX_NAME)?;
            if v.sorts.len() > MAX_SORTS {
                return Err(Error::Other("Too many sorts.".into()));
            }
            let mut count = 0;
            check_filter(&v.filter, 1, &mut count)?;
            // Keep columns for existing properties only, in the order given; add missing ones at the end.
            v.columns.retain(|c| prop_ids.contains(&c.prop));
            let mut have = std::collections::HashSet::new();
            v.columns.retain(|c| have.insert(c.prop.clone()));
            for id in &prop_ids {
                if !have.contains(id) {
                    v.columns.push(Column { prop: id.clone(), visible: true, width: 0 });
                }
            }
            for c in &mut v.columns {
                c.width = c.width.min(2000);
            }
            v.sorts.retain(|s| prop_ids.contains(&s.prop));
            v.calcs.retain(|c| prop_ids.contains(&c.prop));
            v.freeze = v.freeze.min(10);
            if !matches!(v.open_in.as_str(), "full" | "side" | "center") {
                v.open_in = "full".into();
            }
            if let Some(g) = &v.group {
                if !prop_ids.contains(&g.prop) {
                    v.group = None;
                }
            }
            if let Some(g) = &v.sub_group {
                if !prop_ids.contains(&g.prop) || v.group.is_none() {
                    v.sub_group = None;
                }
            }
            check_len("The chart title", &v.chart.title, MAX_NAME)?;
        }
        for t in &self.templates {
            check_len("A template name", &t.name, MAX_NAME)?;
            if t.cells.len() > 200_000 || t.content.len() > 20_000_000 {
                return Err(Error::Other("A template is too large.".into()));
            }
        }
        if !self.default_template.is_empty() && !self.templates.iter().any(|t| t.id == self.default_template) {
            self.default_template.clear();
        }
        self.sanitize_automations()?;
        Ok(())
    }
}

fn check_actions(what: &str, actions: &mut Vec<Action>, props: &[String]) -> Result<()> {
    if actions.len() > MAX_ACTIONS {
        return Err(Error::Other(format!("{what} can have at most {MAX_ACTIONS} actions.")));
    }
    for a in actions.iter_mut() {
        if !matches!(a.kind.as_str(), "setProp" | "addRow" | "notify" | "trash") {
            return Err(Error::Other(format!("{what} has an action Notaya does not know.")));
        }
        check_len("An action value", &a.value, 2000)?;
        check_len("An action title", &a.title, 300)?;
        if a.cells.len() > 100 {
            return Err(Error::Other("An action sets too many cells.".into()));
        }
        for c in &a.cells {
            check_len("An action value", &c.value, 2000)?;
        }
    }
    // An action on a property that was deleted is dropped (so deleting a property is never blocked).
    actions.retain(|a| a.kind != "setProp" || props.contains(&a.prop) || a.prop == TITLE_PROP);
    Ok(())
}

impl Database {
    fn sanitize_automations(&mut self) -> Result<()> {
        if self.automations.len() > MAX_AUTOMATIONS {
            return Err(Error::Other(format!("A database can have at most {MAX_AUTOMATIONS} automations.")));
        }
        let prop_ids: Vec<String> = self.props.iter().map(|p| p.id.clone()).collect();
        let date_ids: Vec<String> = self.props.iter().filter(|p| p.kind == PropKind::Date).map(|p| p.id.clone()).collect();
        let mut seen = std::collections::HashSet::new();
        for a in &mut self.automations {
            if a.id.is_empty() {
                a.id = crate::model::new_id();
            }
            if !seen.insert(a.id.clone()) {
                return Err(Error::Other("Two automations share an id.".into()));
            }
            check_len("An automation name", &a.name, MAX_NAME)?;
            if a.name.trim().is_empty() {
                a.name = "Automation".into();
            }
            match a.trigger.kind.as_str() {
                "rowAdded" => a.trigger.prop.clear(),
                "propChanged" => {
                    if !a.trigger.prop.is_empty() && !prop_ids.contains(&a.trigger.prop) {
                        a.trigger.prop.clear();
                    }
                }
                "dateReached" => {
                    if !date_ids.contains(&a.trigger.prop) {
                        a.enabled = false; // its date property is gone: switched off until the person picks another
                    }
                    a.trigger.offset_mins = a.trigger.offset_mins.clamp(-40_320, 40_320);
                }
                _ => return Err(Error::Other(format!("The automation \"{}\" has a trigger Notaya does not know.", a.name))),
            }
            let mut count = 0;
            check_filter(&a.filter, 1, &mut count)?;
            let what = format!("The automation \"{}\"", a.name);
            check_actions(&what, &mut a.actions, &prop_ids)?;
        }
        for p in &mut self.props {
            if p.kind == PropKind::Button {
                check_len("A button label", &p.button_label, 60)?;
                let what = format!("The button \"{}\"", p.name);
                check_actions(&what, &mut p.actions, &prop_ids)?;
            } else {
                p.actions.clear();
                p.button_label.clear();
            }
        }
        Ok(())
    }
}

impl Property {
    pub fn kind_has_options(&self) -> bool {
        matches!(self.kind, PropKind::Select | PropKind::MultiSelect | PropKind::Status)
    }
    pub fn option(&self, id: &str) -> Option<&SelectOption> {
        self.options.iter().find(|o| o.id == id)
    }
    pub fn option_by_name(&self, name: &str) -> Option<&SelectOption> {
        self.options.iter().find(|o| o.name == name).or_else(|| {
            let l = name.to_lowercase();
            self.options.iter().find(|o| o.name.to_lowercase() == l)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db() -> Database {
        Database::new("d1".into(), "s1".into(), "Tasks".into(), 1)
    }

    #[test]
    fn a_new_database_has_a_title_property_and_a_table_view() {
        let mut d = db();
        d.sanitize().unwrap();
        assert_eq!(d.props.len(), 1);
        assert_eq!(d.props[0].id, TITLE_PROP);
        assert_eq!(d.views.len(), 1);
        assert_eq!(d.views[0].kind, ViewKind::Table);
        assert_eq!(d.views[0].columns.len(), 1);
    }

    #[test]
    fn sanitize_adds_columns_and_drops_unknown_ones() {
        let mut d = db();
        d.props.push(Property { id: "a".into(), name: "Done".into(), kind: PropKind::Checkbox, ..Property::default() });
        d.views[0].columns = vec![Column { prop: "ghost".into(), visible: true, width: 10 }];
        d.views[0].sorts = vec![Sort { prop: "ghost".into(), descending: true }];
        d.sanitize().unwrap();
        let ids: Vec<_> = d.views[0].columns.iter().map(|c| c.prop.as_str()).collect();
        assert_eq!(ids, vec!["title", "a"]);
        assert!(d.views[0].sorts.is_empty());
    }

    #[test]
    fn sanitize_refuses_two_titles_and_repeated_ids() {
        let mut d = db();
        d.props.push(Property { id: "t2".into(), name: "T".into(), kind: PropKind::Title, ..Property::default() });
        assert!(d.sanitize().is_err());
        let mut d = db();
        d.props.push(Property { id: "x".into(), name: "A".into(), kind: PropKind::Text, ..Property::default() });
        d.props.push(Property { id: "x".into(), name: "B".into(), kind: PropKind::Text, ..Property::default() });
        assert!(d.sanitize().is_err());
    }

    #[test]
    fn sanitize_restores_blank_property_names_and_keeps_nonblank_text_exact() {
        for name in ["", "  ", "\t\r\n", "\u{2003}\u{00a0}"] {
            let mut d = db();
            d.props[0].name = name.into();
            d.sanitize().unwrap();
            assert_eq!(d.props[0].name, "Untitled");
        }
        let name = "  \u{200f}\u{639}\u{631}\u{628}\u{64a} ?Deutsch e\u{301}  ";
        let mut d = db();
        d.props[0].name = name.into();
        d.sanitize().unwrap();
        assert_eq!(d.props[0].name.as_bytes(), name.as_bytes());
    }

    #[test]
    fn filters_are_limited_to_three_levels() {
        let mut d = db();
        let deep = Filter { op: "and".into(), conds: vec![], groups: vec![Filter { op: "and".into(), conds: vec![], groups: vec![Filter { op: "and".into(), conds: vec![], groups: vec![Filter::default()] }] }] };
        d.views[0].filter = deep;
        assert!(d.sanitize().is_err());
    }

    #[test]
    fn a_database_record_round_trips_and_old_fields_default() {
        let d = db();
        let json = serde_json::to_string(&crate::model::Item::Database(d)).unwrap();
        assert!(json.contains(r#""kind":"database""#));
        let minimal = r#"{"kind":"database","id":"x","sectionId":"s","title":"T","trashedAt":null,"created":1,"updated":1}"#;
        match serde_json::from_str::<crate::model::Item>(minimal).unwrap() {
            crate::model::Item::Database(d) => assert!(d.props.is_empty() && d.views.is_empty() && !d.inline),
            _ => panic!("expected a database"),
        }
    }
}
