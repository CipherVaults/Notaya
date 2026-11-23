//! Everything stored in the vault. Each value is one encrypted record.
//! Secret-bearing structs wipe their memory when dropped (`ZeroizeOnDrop`).

use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

pub fn new_id() -> String {
    hex::encode(crate::crypto::random_bytes::<16>())
}

/// The encrypted previous value of an AI edit replaced by a person's whole save.
#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(rename_all = "camelCase")]
pub struct ReplacedEdit {
    pub id: String,
    pub rev: u64,
    pub by: String,
    pub status: String,
    pub previous: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Zeroize)]
#[serde(deny_unknown_fields)]
pub struct SourceTag { pub app: String, pub id: String }

/// The settings record always uses this id.
pub const SETTINGS_ID: &str = "00000000000000000000000000000000";

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Item {
    Notebook(Notebook),
    Section(Section),
    Page(Page),
    Sticky(Sticky),
    Settings(Settings),
    /// A database (A35, V1.5.0). V1.4.0 and earlier cannot read a vault that holds one.
    Database(crate::database::types::Database),
    /// A row of a database (A35): a page with cell values.
    Row(crate::database::types::Row),
}

impl Item {
    pub fn id(&self) -> &str {
        match self {
            Item::Notebook(x) => &x.id,
            Item::Section(x) => &x.id,
            Item::Page(x) => &x.id,
            Item::Sticky(x) => &x.id,
            Item::Settings(_) => SETTINGS_ID,
            Item::Database(x) => &x.id,
            Item::Row(x) => &x.id,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(rename_all = "camelCase")]
pub struct Notebook {
    #[serde(default)]
    pub source: Option<SourceTag>,
    pub id: String,
    pub name: String,
    pub color: String,
    pub icon: String,
    pub order: i64,
    pub created: i64,
    pub updated: i64,
    #[serde(default)]
    pub trashed_at: Option<i64>,
}

#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(rename_all = "camelCase")]
pub struct Section {
    #[serde(default)]
    pub source: Option<SourceTag>,
    pub id: String,
    pub notebook_id: String,
    pub name: String,
    pub color: String,
    pub order: i64,
    pub created: i64,
    pub updated: i64,
    #[serde(default)]
    pub trashed_at: Option<i64>,
}

#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    #[serde(default)]
    pub source: Option<SourceTag>,
    pub id: String,
    pub section_id: String,
    pub title: String,
    /// TipTap document as JSON text.
    pub content: String,
    /// Plain-text copy for search and word count (encrypted like everything else).
    pub text: String,
    pub order: i64,
    pub favorite: bool,
    /// Emoji shown beside the title, or `pic:<attachment id>` for a small picture (empty = none).
    #[serde(default)]
    pub icon: String,
    /// "gradient:<name>", "solid:#rrggbb" or an attachment URL, the last optionally ending in `#y=<0..100>`
    /// (how far down the picture is shown, in percent). Empty = no cover.
    #[serde(default)]
    pub cover: String,
    /// Id of the page this one sits inside (empty = directly in its section).
    #[serde(default)]
    pub parent: String,
    pub trashed_at: Option<i64>,
    pub created: i64,
    pub updated: i64,
}

/// Page without its body, used for lists and the tree.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageMeta {
    pub parent: String,
    pub id: String,
    pub section_id: String,
    pub title: String,
    pub preview: String,
    pub words: usize,
    pub order: i64,
    pub favorite: bool,
    pub icon: String,
    /// The page's cover (see `Page::cover`), so a card can show a strip of it without opening the page.
    pub cover: String,
    pub trashed_at: Option<i64>,
    pub created: i64,
    pub updated: i64,
}

impl From<&Page> for PageMeta {
    fn from(p: &Page) -> Self {
        PageMeta {
            id: p.id.clone(),
            section_id: p.section_id.clone(),
            title: p.title.clone(),
            preview: p.text.chars().take(160).collect(),
            words: p.text.split_whitespace().count(),
            order: p.order,
            favorite: p.favorite,
            icon: p.icon.clone(),
            cover: p.cover.clone(),
            parent: p.parent.clone(),
            trashed_at: p.trashed_at,
            created: p.created,
            updated: p.updated,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(rename_all = "camelCase")]
pub struct Sticky {
    pub id: String,
    pub content: String,
    pub text: String,
    pub color: String,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub on_top: bool,
    pub open: bool,
    pub created: i64,
    pub updated: i64,
    #[serde(default)]
    pub trashed_at: Option<i64>,
}

#[derive(Clone, Serialize, Deserialize, Zeroize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// Internal authenticated migration latch. Older readers ignore this field.
    pub journals_anchored: bool,
    pub ai: crate::ai::pairing::Settings,
    /// Previous AI values replaced by a person. Encrypted; never sent by get_settings.
    #[serde(default)]
    pub replaced_edits: Vec<ReplacedEdit>,
    /// "sunrise" | "sunrise-dark" | "black" | "graphite" | "white" | "paper" | "system"
    pub theme: String,
    pub accent: String,
    /// "mica" | "acrylic" | "none"
    pub backdrop: String,
    /// "inter" | "system" | "serif" | "mono"
    pub font: String,
    pub font_size: u32,
    /// "narrow" | "normal" | "wide" | "full"
    pub editor_width: String,
    /// "comfortable" | "compact"
    pub density: String,
    /// "aurora" | "mesh" | "midnight" | "plain"
    pub unlock_background: String,
    pub idle_lock_minutes: u32,
    pub lock_on_minimize: bool,
    pub screen_capture_protection: bool,
    pub sticky_default_color: String,
    /// The calculator history, newest first, shared by every calculator window. It is kept
    /// here (not as its own record) so an older version (Cipherleaf 1.x) can still open the vault: it
    /// ignores the field. The UI reads and changes it only through the calculator commands.
    pub calc_history: Vec<CalcEntry>,
    /// The last few things searched for, newest first (V1.5.1). Kept here like the calculator history: it is
    /// encrypted with the vault, wiped from memory when the vault locks, and the UI reads and changes it only
    /// through the search-history commands.
    pub search_history: Vec<String>,
    /// The last note location, encrypted here like calculator history. Older versions ignore it.
    /// Only `set_last_place` changes it; ordinary appearance saves must preserve it.
    #[serde(default)]
    pub last_place: Option<LastPlace>,
}

/// Identifiers only: no titles, text or paths. A missing location opens Home.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Zeroize)]
#[serde(rename_all = "camelCase")]
pub struct LastPlace {
    pub kind: String,
    pub id: String,
    #[serde(default)]
    pub view_id: Option<String>,
}

impl LastPlace {
    pub fn valid(&self) -> bool {
        let valid_id = |id: &str| !id.is_empty() && id.len() <= 128 && id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_');
        matches!(self.kind.as_str(), "page" | "row" | "database")
            && valid_id(&self.id)
            && self.view_id.as_deref().is_none_or(|id| self.kind == "database" && valid_id(id))
    }
}

/// How many searches the history keeps, and the longest one it stores (in characters).
pub const SEARCH_HISTORY_MAX: usize = 5;
pub const SEARCH_TERM_MAX: usize = 200;

/// How many calculations the history keeps.
pub const CALC_HISTORY_MAX: usize = 500;

/// One line of the calculator history: what was typed and the answer shown.
#[derive(Clone, Default, Serialize, Deserialize, Zeroize)]
#[serde(rename_all = "camelCase", default)]
pub struct CalcEntry {
    pub expr: String,
    pub result: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            journals_anchored: false,
            ai: crate::ai::pairing::Settings::default(),
            replaced_edits: Vec::new(),
            // Sunrise is the default look (A28). Sunrise dark and the others are chosen in Settings.
            theme: "sunrise".into(),
            accent: "mono".into(),
            backdrop: "none".into(),
            font: "inter".into(),
            font_size: 15,
            editor_width: "normal".into(),
            density: "comfortable".into(),
            unlock_background: "midnight".into(),
            idle_lock_minutes: 5,
            lock_on_minimize: false,
            // Off by default, always (A21). The person turns it on in Settings, and it
            // only has an effect while security is on.
            screen_capture_protection: false,
            sticky_default_color: "yellow".into(),
            calc_history: Vec::new(),
            search_history: Vec::new(),
            last_place: None,
        }
    }
}

impl Settings {
    /// Clamps values coming from the UI into safe ranges.
    pub fn sanitized(mut self) -> Settings {
        // Vaults created before the black-and-white redesign stored violet as their
        // default accent; show them the new default unless a color was chosen since.
        if self.accent.eq_ignore_ascii_case("#8b5cf6") {
            self.accent = "mono".into();
        }
        // The first two looks were called "dark" and "light".
        self.theme = match self.theme.as_str() {
            "dark" => "black".into(),
            "light" => "white".into(),
            "sunrise" | "sunrise-dark" | "black" | "graphite" | "white" | "paper" | "system" => self.theme.clone(),
            _ => "black".into(),
        };
        if self.accent != "mono"
            && !(self.accent.starts_with('#')
                && self.accent.len() == 7
                && self.accent[1..].bytes().all(|b| b.is_ascii_hexdigit()))
        {
            self.accent = "mono".into();
        }
        self.font = match self.font.as_str() {
            "inter" | "system" | "serif" | "mono" => self.font.clone(),
            _ => "inter".into(),
        };
        self.editor_width = match self.editor_width.as_str() {
            "narrow" | "normal" | "wide" | "full" => self.editor_width.clone(),
            _ => "normal".into(),
        };
        self.density = match self.density.as_str() {
            "comfortable" | "compact" => self.density.clone(),
            _ => "comfortable".into(),
        };
        self.backdrop = match self.backdrop.as_str() {
            "mica" | "acrylic" | "none" => self.backdrop.clone(),
            _ => "none".into(),
        };
        self.unlock_background = match self.unlock_background.as_str() {
            "aurora" | "mesh" | "midnight" | "plain" => self.unlock_background.clone(),
            _ => "midnight".into(),
        };
        self.sticky_default_color = match self.sticky_default_color.as_str() {
            "yellow" | "pink" | "green" | "blue" | "purple" | "orange" | "gray" | "charcoal" => {
                self.sticky_default_color.clone()
            }
            _ => "yellow".into(),
        };
        self.font_size = self.font_size.clamp(12, 22);
        self.idle_lock_minutes = self.idle_lock_minutes.clamp(1, 240);
        self.calc_history.truncate(CALC_HISTORY_MAX);
        self.search_history.retain(|t| !t.trim().is_empty());
        self.search_history.truncate(SEARCH_HISTORY_MAX);
        for t in &mut self.search_history {
            if t.chars().count() > SEARCH_TERM_MAX {
                *t = t.chars().take(SEARCH_TERM_MAX).collect();
            }
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn last_place_defaults_and_round_trips_without_breaking_old_readers() {
        let old = r#"{"kind":"settings","theme":"paper"}"#;
        let Item::Settings(mut settings) = serde_json::from_str::<Item>(old).unwrap() else { panic!("settings") };
        assert!(settings.last_place.is_none());
        settings.last_place = Some(LastPlace { kind: "database".into(), id: new_id(), view_id: Some(new_id()) });
        let json = serde_json::to_string(&Item::Settings(settings.clone())).unwrap();
        let Item::Settings(loaded) = serde_json::from_str::<Item>(&json).unwrap() else { panic!("settings") };
        assert_eq!(loaded.last_place, settings.last_place);
        // Like 1.5.1, a reader without this field ignores it and keeps the settings it knows.
        #[derive(Deserialize)]
        struct OldSettings { theme: String }
        assert_eq!(serde_json::from_str::<OldSettings>(&json).unwrap().theme, "paper");
    }

    #[test]
    fn last_place_accepts_only_bounded_identifiers_and_known_kinds() {
        for kind in ["page", "row", "database"] {
            let mut place = LastPlace { kind: kind.into(), id: new_id(), view_id: None };
            assert!(place.valid());
            place.view_id = Some(new_id());
            assert_eq!(place.valid(), kind == "database");
        }
        for (kind, id) in [("settings", "p1"), ("page", ""), ("page", "../note"), ("page", "note title")] {
            assert!(!LastPlace { kind: kind.into(), id: id.into(), view_id: None }.valid());
        }
        assert!(!LastPlace { kind: "page".into(), id: "a".repeat(129), view_id: None }.valid());
    }

    #[test]
    fn sunrise_is_the_default_look_and_both_sunrise_looks_are_kept() {
        assert_eq!(Settings::default().theme, "sunrise");
        for look in ["sunrise", "sunrise-dark", "paper", "system"] {
            let s = Settings { theme: look.into(), ..Settings::default() }.sanitized();
            assert_eq!(s.theme, look);
        }
        let s = Settings { theme: "neon".into(), ..Settings::default() }.sanitized();
        assert_eq!(s.theme, "black", "an unknown look falls back as before");
    }

    #[test]
    fn screenshot_protection_is_off_by_default() {
        assert!(!Settings::default().screen_capture_protection);
    }

    #[test]
    fn a_settings_record_with_the_removed_passwords_switch_still_loads() {
        // Vaults saved by 1.0 and 1.0.1 have `passwordsEnabled` (the KeePassXC switch, now
        // removed). It is ignored, and the other choices are kept.
        let old = r#"{"kind":"settings","theme":"paper","screenCaptureProtection":true,"passwordsEnabled":true}"#;
        match serde_json::from_str::<Item>(old).unwrap() {
            Item::Settings(s) => {
                assert_eq!(s.theme, "paper");
                assert!(s.screen_capture_protection, "a saved choice is kept");
            }
            _ => panic!("expected settings"),
        }
    }

    #[test]
    fn calc_history_defaults_to_empty_and_round_trips() {
        let old = r#"{"kind":"settings","theme":"paper"}"#;
        match serde_json::from_str::<Item>(old).unwrap() {
            Item::Settings(s) => assert!(s.calc_history.is_empty()),
            _ => panic!("expected settings"),
        }
        let mut s = Settings::default();
        s.calc_history.push(CalcEntry { expr: "2+2".into(), result: "4".into() });
        let json = serde_json::to_string(&Item::Settings(s)).unwrap();
        assert!(json.contains(r#""calcHistory":[{"expr":"2+2","result":"4"}]"#));
        match serde_json::from_str::<Item>(&json).unwrap() {
            Item::Settings(s) => assert_eq!(s.calc_history[0].result, "4"),
            _ => panic!("expected settings"),
        }
    }

    #[test]
    fn sanitized_caps_the_calc_history() {
        let mut s = Settings::default();
        assert_eq!(CALC_HISTORY_MAX, 500, "V1.4.0: the history keeps 500 calculations");
        s.calc_history = vec![CalcEntry::default(); CALC_HISTORY_MAX + 20];
        assert_eq!(s.sanitized().calc_history.len(), CALC_HISTORY_MAX);
    }

    #[test]
    fn search_history_defaults_to_empty_and_is_capped() {
        let old = r#"{"kind":"settings","theme":"paper"}"#;
        match serde_json::from_str::<Item>(old).unwrap() {
            Item::Settings(s) => assert!(s.search_history.is_empty()),
            _ => panic!("expected settings"),
        }
        let mut s = Settings::default();
        s.search_history = (0..9).map(|i| format!("term {i}")).collect();
        s.search_history.push("x".repeat(SEARCH_TERM_MAX + 50));
        let s = s.sanitized();
        assert_eq!(s.search_history.len(), SEARCH_HISTORY_MAX);
        let mut t = Settings::default();
        t.search_history = vec!["  ".into(), "y".repeat(SEARCH_TERM_MAX + 50)];
        let t = t.sanitized();
        assert_eq!(t.search_history.len(), 1);
        assert_eq!(t.search_history[0].chars().count(), SEARCH_TERM_MAX);
    }
}
