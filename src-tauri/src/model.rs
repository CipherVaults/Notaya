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

/// The settings record always uses this id.
pub const SETTINGS_ID: &str = "00000000000000000000000000000000";

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Item {
    Notebook(Notebook),
    Section(Section),
    Page(Page),
    Sticky(Sticky),
    Login(Login),
    Settings(Settings),
}

impl Item {
    pub fn id(&self) -> &str {
        match self {
            Item::Notebook(x) => &x.id,
            Item::Section(x) => &x.id,
            Item::Page(x) => &x.id,
            Item::Sticky(x) => &x.id,
            Item::Login(x) => &x.id,
            Item::Settings(_) => SETTINGS_ID,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(rename_all = "camelCase")]
pub struct Notebook {
    pub id: String,
    pub name: String,
    pub color: String,
    pub icon: String,
    pub order: i64,
    pub created: i64,
    pub updated: i64,
}

#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(rename_all = "camelCase")]
pub struct Section {
    pub id: String,
    pub notebook_id: String,
    pub name: String,
    pub color: String,
    pub order: i64,
    pub created: i64,
    pub updated: i64,
}

#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    pub id: String,
    pub section_id: String,
    pub title: String,
    /// TipTap document as JSON text.
    pub content: String,
    /// Plain-text copy for search and word count (encrypted like everything else).
    pub text: String,
    pub order: i64,
    pub favorite: bool,
    pub trashed_at: Option<i64>,
    pub created: i64,
    pub updated: i64,
}

/// Page without its body, used for lists and the tree.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageMeta {
    pub id: String,
    pub section_id: String,
    pub title: String,
    pub preview: String,
    pub words: usize,
    pub order: i64,
    pub favorite: bool,
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
}

#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(rename_all = "camelCase")]
pub struct CustomField {
    pub name: String,
    pub value: String,
    pub secret: bool,
}

#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(rename_all = "camelCase")]
pub struct PasswordHistory {
    pub password: String,
    pub changed: i64,
}

#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(rename_all = "camelCase")]
pub struct Login {
    pub id: String,
    pub title: String,
    pub username: String,
    pub password: String,
    pub urls: Vec<String>,
    pub notes: String,
    /// Base32 secret or `otpauth://` URI; empty when unused.
    pub totp: String,
    pub fields: Vec<CustomField>,
    pub tags: Vec<String>,
    pub folder: String,
    pub favorite: bool,
    pub color: String,
    pub created: i64,
    pub updated: i64,
    pub password_changed: i64,
    pub history: Vec<PasswordHistory>,
}

/// What the list view needs. Contains no secrets.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginSummary {
    pub id: String,
    pub title: String,
    pub username: String,
    pub url: String,
    pub folder: String,
    pub tags: Vec<String>,
    pub favorite: bool,
    pub color: String,
    pub has_totp: bool,
    pub updated: i64,
    pub password_changed: i64,
}

impl From<&Login> for LoginSummary {
    fn from(l: &Login) -> Self {
        LoginSummary {
            id: l.id.clone(),
            title: l.title.clone(),
            username: l.username.clone(),
            url: l.urls.first().cloned().unwrap_or_default(),
            folder: l.folder.clone(),
            tags: l.tags.clone(),
            favorite: l.favorite,
            color: l.color.clone(),
            has_totp: !l.totp.trim().is_empty(),
            updated: l.updated,
            password_changed: l.password_changed,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Zeroize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// "dark" | "light" | "system"
    pub theme: String,
    pub accent: String,
    /// "mica" | "acrylic" | "none"
    pub backdrop: String,
    /// "inter" | "system" | "serif" | "mono"
    pub font: String,
    pub font_size: u32,
    /// "narrow" | "normal" | "wide"
    pub editor_width: String,
    /// "comfortable" | "compact"
    pub density: String,
    /// "aurora" | "mesh" | "midnight" | "plain"
    pub unlock_background: String,
    pub idle_lock_minutes: u32,
    pub lock_on_minimize: bool,
    pub clipboard_clear_seconds: u32,
    pub screen_capture_protection: bool,
    pub breach_check_enabled: bool,
    pub sticky_default_color: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            theme: "dark".into(),
            accent: "#8b5cf6".into(),
            backdrop: "mica".into(),
            font: "inter".into(),
            font_size: 15,
            editor_width: "normal".into(),
            density: "comfortable".into(),
            unlock_background: "aurora".into(),
            idle_lock_minutes: 5,
            lock_on_minimize: false,
            clipboard_clear_seconds: 20,
            screen_capture_protection: true,
            breach_check_enabled: false,
            sticky_default_color: "yellow".into(),
        }
    }
}

impl Settings {
    /// Clamps values coming from the UI into safe ranges.
    pub fn sanitized(mut self) -> Settings {
        self.font_size = self.font_size.clamp(12, 22);
        self.idle_lock_minutes = self.idle_lock_minutes.clamp(1, 240);
        self.clipboard_clear_seconds = self.clipboard_clear_seconds.clamp(5, 300);
        self
    }
}
