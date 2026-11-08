//! Attribute validation never coerces or rewrites a stored string.
use super::ids;
use serde_json::Value;

fn string(v: &Value, max: usize, nullable: bool) -> bool {
    (nullable && v.is_null()) || v.as_str().is_some_and(|s| s.encode_utf16().count() <= max)
}
fn choice(v: &Value, choices: &[&str], nullable: bool) -> bool {
    (nullable && v.is_null()) || v.as_str().is_some_and(|s| choices.contains(&s))
}
fn number(v: &Value, min: f64, max: f64, nullable: bool) -> bool {
    (nullable && v.is_null())
        || v.as_f64()
            .is_some_and(|n| n.is_finite() && n >= min && n <= max)
}

pub(super) fn valid(node: &Value, mark: bool) -> bool {
    let Some(attrs) = node.get("attrs") else {
        return true;
    };
    let Some(attrs) = attrs.as_object() else {
        return false;
    };
    let kind = node["type"].as_str().unwrap_or("");
    attrs.iter().all(|(key, v)| {
        let key = key.as_str();
        if mark {
            return match (kind, key) {
                ("link", "href" | "target" | "rel" | "class" | "title") => string(v, 8192, true),
                ("highlight", "color") | ("textStyle", "color" | "fontSize" | "fontFamily") => {
                    string(v, 1024, true)
                }
                ("comment", "text") => string(v, crate::importers::doc::MAX_RENDER_BYTES, false),
                _ => false,
            };
        }
        if key == "bid" {
            return ids::addressable(node)
                && (v.is_null() || v.as_str().is_some_and(ids::valid_id));
        }
        match (kind, key) {
            ("paragraph" | "heading", "dir") => choice(v, &["rtl", "ltr", "auto"], true),
            ("paragraph" | "heading", "textAlign") => {
                choice(v, &["left", "center", "right", "justify"], true)
            }
            ("heading", "level") => v.as_u64().is_some_and(|n| (1..=4).contains(&n)),
            ("heading", "anchor") => string(v, 128, true),
            ("codeBlock", "language") => string(v, 32, true),
            ("orderedList", "start") => v.as_u64().is_some_and(|n| n > 0 && n <= u32::MAX as u64),
            ("orderedList", "type") => choice(v, &["1", "a", "A", "i", "I"], true),
            ("taskItem", "checked")
            | ("toggle", "open" | "shelf")
            | ("mermaidBlock", "showCode") => v.is_boolean(),
            ("callout", "icon") => string(v, 40, false),
            ("mathInline" | "mathBlock", "latex") => string(v, 20_000, false),
            ("mermaidBlock", "code") => string(v, 50_000, false),
            ("pageLink", "pageId") | ("databaseBlock", "databaseId" | "viewId") => {
                string(v, 128, false)
            }
            ("image", "src") => string(v, 8192, true),
            ("image", "alt" | "title") => string(v, crate::importers::doc::MAX_RENDER_BYTES, true),
            ("image", "caption") => string(v, crate::importers::doc::MAX_RENDER_BYTES, false),
            ("image", "width") => number(v, f64::MIN_POSITIVE, 100.0, true),
            ("image", "height") => number(v, f64::MIN_POSITIVE, 1_000_000.0, true),
            ("image", "x" | "y") => number(v, -1_000_000.0, 1_000_000.0, true),
            ("image", "align") => choice(v, &["left", "center", "right"], false),
            ("image", "layer") => choice(v, &["front", "back"], false),
            ("image", "ratio") => choice(v, &["", "1:1", "4:3", "3:2", "16:9", "9:16"], false),
            ("image", "float" | "free") => v.is_boolean(),
            ("tableCell" | "tableHeader", "colspan" | "rowspan") => {
                v.as_u64().is_some_and(|n| n > 0 && n <= 20_000)
            }
            ("tableCell" | "tableHeader", "colwidth") => {
                v.is_null()
                    || v.as_array().is_some_and(|widths| {
                        widths.len() == node["attrs"]["colspan"].as_u64().unwrap_or(1) as usize
                            && widths
                                .iter()
                                .all(|w| w.as_u64().is_some_and(|n| n <= 1_000_000))
                    })
            }
            ("tableCell" | "tableHeader", "align") => {
                choice(v, &["left", "center", "right", "justify"], true)
            }
            _ => false,
        }
    })
}
