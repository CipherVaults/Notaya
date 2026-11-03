//! Commands the UI can call. Each one checks its inputs, and none of them gives
//! the UI general file-system or network access.

pub mod backup;
pub mod calculator;
pub mod database;
pub mod database_io;
pub mod errorlog;
pub mod export;
pub mod import_notes;
pub mod notes;
pub mod shell;
pub mod stickies;
pub mod tools;
pub mod vault;

use crate::error::{Error, Result};
use crate::model::{Item, Notebook, Page, Section, Sticky};
use crate::vault::Session;

macro_rules! typed_getter {
    ($name:ident, $variant:ident, $ty:ty) => {
        pub fn $name(s: &Session, id: &str) -> Result<$ty> {
            match s.get(id) {
                Some(Item::$variant(x)) => Ok(x.clone()),
                _ => Err(Error::NotFound),
            }
        }
    };
}

typed_getter!(notebook, Notebook, Notebook);
typed_getter!(section, Section, Section);
typed_getter!(page, Page, Page);
typed_getter!(sticky, Sticky, Sticky);

/// Rejects oversized text fields from the UI.
pub fn limit(field: &str, value: &str, max_chars: usize) -> Result<()> {
    if value.chars().count() > max_chars {
        Err(Error::invalid(format!("{field} is too long (max {max_chars} characters).")))
    } else {
        Ok(())
    }
}

/// Accepts "#rrggbb" colors or a short named color; anything else is refused.
pub fn check_color(value: &str) -> Result<String> {
    let v = value.trim();
    let hex_ok = v.len() == 7 && v.starts_with('#') && v[1..].chars().all(|c| c.is_ascii_hexdigit());
    let name_ok = !v.is_empty() && v.len() <= 16 && v.chars().all(|c| c.is_ascii_alphabetic());
    if v.is_empty() || hex_ok || name_ok {
        Ok(v.to_string())
    } else {
        Err(Error::invalid("Invalid color."))
    }
}
