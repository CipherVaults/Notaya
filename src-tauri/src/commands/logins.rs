use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;
use zeroize::Zeroizing;

use super::{check_color, limit, login};
use crate::error::{Error, Result};
use crate::generator;
use crate::model::{new_id, now_ms, CustomField, Item, Login, LoginSummary, PasswordHistory};
use crate::state::AppState;

const MAX_HISTORY: usize = 20;
const OLD_AFTER_DAYS: i64 = 365;

#[tauri::command]
pub fn list_logins(state: State<'_, AppState>) -> Result<Vec<LoginSummary>> {
    state.with(|s| {
        let mut list: Vec<LoginSummary> = s
            .items()
            .filter_map(|i| match i {
                Item::Login(l) => Some(LoginSummary::from(l)),
                _ => None,
            })
            .collect();
        list.sort_by_key(|l| l.title.to_lowercase());
        Ok(list)
    })
}

#[tauri::command]
pub fn get_login(state: State<'_, AppState>, id: String) -> Result<Login> {
    state.with(|s| login(s, &id))
}

#[derive(Deserialize, zeroize::Zeroize, zeroize::ZeroizeOnDrop)]
#[serde(rename_all = "camelCase")]
pub struct LoginInput {
    id: Option<String>,
    title: String,
    username: String,
    password: String,
    urls: Vec<String>,
    notes: String,
    totp: String,
    fields: Vec<CustomField>,
    tags: Vec<String>,
    folder: String,
    favorite: bool,
    color: String,
}

fn validate(input: &LoginInput) -> Result<()> {
    if input.title.trim().is_empty() {
        return Err(Error::invalid("Give this login a name."));
    }
    limit("Name", &input.title, 200)?;
    limit("Username", &input.username, 500)?;
    limit("Password", &input.password, 4096)?;
    limit("Notes", &input.notes, 100_000)?;
    limit("Folder", &input.folder, 100)?;
    if input.urls.len() > 20 || input.tags.len() > 50 || input.fields.len() > 50 {
        return Err(Error::invalid("Too many URLs, tags or fields."));
    }
    for u in &input.urls {
        limit("URL", u, 2048)?;
    }
    for t in &input.tags {
        limit("Tag", t, 60)?;
    }
    for f in &input.fields {
        limit("Field name", &f.name, 100)?;
        limit("Field value", &f.value, 10_000)?;
    }
    if !input.totp.trim().is_empty() {
        build_totp(&input.totp)?;
    }
    Ok(())
}

#[tauri::command]
pub fn save_login(state: State<'_, AppState>, input: LoginInput) -> Result<LoginSummary> {
    validate(&input)?;
    let color = check_color(&input.color)?;
    state.with_mut(|s| {
        let now = now_ms();
        let existing = match &input.id {
            Some(id) => Some(login(s, id)?),
            None => None,
        };
        let (id, created, mut history, mut password_changed) = match &existing {
            Some(e) => (e.id.clone(), e.created, e.history.clone(), e.password_changed),
            None => (new_id(), now, Vec::new(), now),
        };
        if let Some(e) = &existing {
            if e.password != input.password {
                if !e.password.is_empty() {
                    history.insert(0, PasswordHistory { password: e.password.clone(), changed: now });
                    history.truncate(MAX_HISTORY);
                }
                password_changed = now;
            }
        }
        let l = Login {
            id,
            title: input.title.trim().to_string(),
            username: input.username.clone(),
            password: input.password.clone(),
            urls: input.urls.iter().map(|u| u.trim().to_string()).filter(|u| !u.is_empty()).collect(),
            notes: input.notes.clone(),
            totp: input.totp.trim().to_string(),
            fields: input.fields.clone(),
            tags: input.tags.iter().map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect(),
            folder: input.folder.trim().to_string(),
            favorite: input.favorite,
            color,
            created,
            updated: now,
            password_changed,
            history,
        };
        let summary = LoginSummary::from(&l);
        s.put(Item::Login(l))?;
        Ok(summary)
    })
}

#[tauri::command]
pub fn delete_login(state: State<'_, AppState>, id: String) -> Result<()> {
    state.with_mut(|s| {
        login(s, &id)?;
        s.remove_many(&[id])
    })
}

#[tauri::command]
pub fn set_login_favorite(state: State<'_, AppState>, id: String, favorite: bool) -> Result<LoginSummary> {
    state.with_mut(|s| {
        let mut l = login(s, &id)?;
        l.favorite = favorite;
        let summary = LoginSummary::from(&l);
        s.put(Item::Login(l))?;
        Ok(summary)
    })
}

/// Copies one field of a login straight from Rust to the clipboard, so the
/// secret never has to pass through the UI. Returns seconds until it is cleared.
#[tauri::command]
pub fn copy_login_field(state: State<'_, AppState>, id: String, field: String) -> Result<u32> {
    let (value, secs) = state.with(|s| {
        let l = login(s, &id)?;
        let value: Zeroizing<String> = Zeroizing::new(match field.as_str() {
            "username" => l.username.clone(),
            "password" => l.password.clone(),
            "url" => l.urls.first().cloned().unwrap_or_default(),
            "totp" => build_totp(&l.totp)?.generate_current().to_string(),
            other => match other.strip_prefix("field:").and_then(|i| i.parse::<usize>().ok()) {
                Some(i) => l.fields.get(i).map(|f| f.value.clone()).ok_or(Error::NotFound)?,
                None => return Err(Error::invalid("Unknown field.")),
            },
        });
        Ok((value, s.settings().clipboard_clear_seconds))
    })?;
    if value.is_empty() {
        return Err(Error::invalid("That field is empty."));
    }
    state.copy_secret(&value, secs)?;
    Ok(secs)
}

/// Copies arbitrary text (e.g. a generated password) with the same protections.
#[tauri::command]
pub fn copy_secret_text(state: State<'_, AppState>, text: String) -> Result<u32> {
    let text = Zeroizing::new(text);
    let secs = state.with(|s| Ok(s.settings().clipboard_clear_seconds)).unwrap_or(20);
    state.copy_secret(&text, secs)?;
    Ok(secs)
}

pub fn build_totp(secret: &str) -> Result<totp_rs::Totp> {
    let s = secret.trim();
    let bad = || Error::invalid("That 2FA secret isn't valid. Paste the setup key or otpauth:// link.");
    if s.starts_with("otpauth://") {
        return totp_rs::Totp::from_url_unchecked(s).map_err(|_| bad());
    }
    let cleaned: Zeroizing<String> = Zeroizing::new(
        s.chars().filter(|c| !c.is_whitespace() && *c != '-').collect::<String>().to_uppercase(),
    );
    let secret = totp_rs::Secret::try_from_base32(cleaned.as_str()).map_err(|_| bad())?;
    if secret.as_ref().len() < 10 {
        return Err(bad());
    }
    Ok(totp_rs::Builder::new().with_secret(secret).build_noncompliant())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TotpCode {
    code: String,
    remaining: u64,
    period: u64,
}

fn code_for(t: &totp_rs::Totp) -> TotpCode {
    TotpCode { code: t.generate_current().to_string(), remaining: t.ttl(), period: t.step() }
}

#[tauri::command]
pub fn totp_code(state: State<'_, AppState>, id: String) -> Result<TotpCode> {
    state.with(|s| Ok(code_for(&build_totp(&login(s, &id)?.totp)?)))
}

/// Previews a code while the user is typing a secret in the edit form.
#[tauri::command]
pub fn totp_preview(secret: String) -> Result<TotpCode> {
    let secret = Zeroizing::new(secret);
    Ok(code_for(&build_totp(&secret)?))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthIssue {
    id: String,
    title: String,
    username: String,
    detail: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Health {
    total: usize,
    score: u8,
    weak: Vec<HealthIssue>,
    reused: Vec<HealthIssue>,
    old: Vec<HealthIssue>,
}

#[tauri::command(async)]
pub fn password_health(state: State<'_, AppState>) -> Result<Health> {
    state.with(|s| {
        let logins: Vec<&Login> = s
            .items()
            .filter_map(|i| match i {
                Item::Login(l) if !l.password.is_empty() => Some(l),
                _ => None,
            })
            .collect();
        let issue = |l: &Login, detail: String| HealthIssue {
            id: l.id.clone(),
            title: l.title.clone(),
            username: l.username.clone(),
            detail,
        };
        let mut flagged = std::collections::HashSet::new();

        let mut weak = Vec::new();
        for l in &logins {
            let st = generator::strength(&l.password, &[&l.username, &l.title]);
            if st.score < 3 {
                flagged.insert(l.id.clone());
                weak.push(issue(l, format!("Could be cracked in {}", st.crack_time)));
            }
        }

        let mut by_password: HashMap<[u8; 32], Vec<&Login>> = HashMap::new();
        for l in &logins {
            by_password.entry(crate::crypto::sha256(l.password.as_bytes())).or_default().push(l);
        }
        let mut reused = Vec::new();
        for group in by_password.values().filter(|g| g.len() > 1) {
            for l in group {
                flagged.insert(l.id.clone());
                reused.push(issue(l, format!("Same password as {} other login(s)", group.len() - 1)));
            }
        }

        let cutoff = now_ms() - OLD_AFTER_DAYS * 24 * 60 * 60 * 1000;
        let mut old = Vec::new();
        for l in &logins {
            if l.password_changed < cutoff {
                flagged.insert(l.id.clone());
                let days = (now_ms() - l.password_changed) / (24 * 60 * 60 * 1000);
                old.push(issue(l, format!("Not changed in {days} days")));
            }
        }

        let total = logins.len();
        let score = if total == 0 { 100 } else { (100 * (total - flagged.len()) / total) as u8 };
        Ok(Health { total, score, weak, reused, old })
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    imported: usize,
    skipped: usize,
    path: String,
}

/// Imports logins from a CSV export (Chrome, Edge, Firefox, Bitwarden, 1Password …).
/// Columns are matched by name.
#[tauri::command(async)]
pub fn import_csv(app: AppHandle, state: State<'_, AppState>) -> Result<Option<ImportResult>> {
    if !state.is_unlocked() {
        return Err(Error::Locked);
    }
    let Some(path) = app
        .dialog()
        .file()
        .set_title("Import passwords from a CSV file")
        .add_filter("CSV", &["csv"])
        .blocking_pick_file()
        .and_then(|f| f.into_path().ok())
    else {
        return Ok(None);
    };
    let data = Zeroizing::new(std::fs::read(&path)?);
    let mut reader = csv::ReaderBuilder::new().flexible(true).from_reader(data.as_slice());
    let headers: Vec<String> = reader
        .headers()
        .map_err(|_| Error::invalid("That file isn't a readable CSV."))?
        .iter()
        .map(|h| h.trim().trim_start_matches('\u{feff}').to_lowercase())
        .collect();
    let col = |names: &[&str]| headers.iter().position(|h| names.contains(&h.as_str()));
    let c_title = col(&["name", "title", "account"]);
    let c_url = col(&["url", "login_uri", "website", "web site", "uri"]);
    let c_user = col(&["username", "login_username", "user", "email", "login"]);
    let c_pass = col(&["password", "login_password"]);
    let c_notes = col(&["note", "notes", "extra"]);
    let c_totp = col(&["login_totp", "otpauth", "totp", "otp"]);
    let c_folder = col(&["folder", "group", "grouping"]);
    if c_pass.is_none() {
        return Err(Error::invalid("No password column was found in that CSV file."));
    }

    let now = now_ms();
    let mut items = Vec::new();
    let mut skipped = 0;
    for record in reader.records() {
        let Ok(record) = record else {
            skipped += 1;
            continue;
        };
        let get = |c: Option<usize>| c.and_then(|i| record.get(i)).unwrap_or("").trim().to_string();
        let url = get(c_url);
        let mut title = get(c_title);
        if title.is_empty() {
            title = url.clone();
        }
        let password = get(c_pass);
        if title.is_empty() && password.is_empty() {
            skipped += 1;
            continue;
        }
        let totp = get(c_totp);
        let totp = if !totp.is_empty() && build_totp(&totp).is_ok() { totp } else { String::new() };
        items.push(Item::Login(Login {
            id: new_id(),
            title: if title.is_empty() { "Imported login".into() } else { title },
            username: get(c_user),
            password,
            urls: if url.is_empty() { vec![] } else { vec![url] },
            notes: get(c_notes),
            totp,
            fields: vec![],
            tags: vec!["imported".into()],
            folder: get(c_folder),
            favorite: false,
            color: String::new(),
            created: now,
            updated: now,
            password_changed: now,
            history: vec![],
        }));
    }
    let imported = items.len();
    state.with_mut(|s| s.put_many(items))?;
    Ok(Some(ImportResult { imported, skipped, path: path.display().to_string() }))
}

/// Opt-in Have I Been Pwned check using k-anonymity: only the first 5 hex
/// characters of the password's SHA-1 hash leave this PC, never the password.
async fn pwned_count(password: &str) -> Result<u64> {
    let hash = Zeroizing::new(hex::encode_upper(Sha1::digest(password.as_bytes())));
    let (prefix, suffix) = hash.split_at(5);
    let client = reqwest::Client::builder()
        .https_only(true)
        .timeout(std::time::Duration::from_secs(15))
        .user_agent("Cipherleaf-password-manager")
        .build()
        .map_err(|e| Error::Other(e.to_string()))?;
    let body = client
        .get(format!("https://api.pwnedpasswords.com/range/{prefix}"))
        .header("Add-Padding", "true")
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|_| Error::Other("Couldn't reach the breach database. Check your connection.".into()))?
        .text()
        .await
        .map_err(|e| Error::Other(e.to_string()))?;
    for line in body.lines() {
        if let Some((s, count)) = line.trim().split_once(':') {
            if s.eq_ignore_ascii_case(suffix) {
                return Ok(count.trim().parse().unwrap_or(0));
            }
        }
    }
    Ok(0)
}

fn breach_enabled(state: &AppState) -> Result<()> {
    if state.with(|s| Ok(s.settings().breach_check_enabled))? {
        Ok(())
    } else {
        Err(Error::invalid("Turn on breach checking in Settings first."))
    }
}

#[tauri::command]
pub async fn check_breach(state: State<'_, AppState>, id: String) -> Result<u64> {
    breach_enabled(&state)?;
    let password = state.with(|s| Ok(Zeroizing::new(login(s, &id)?.password.clone())))?;
    pwned_count(&password).await
}

#[tauri::command]
pub async fn check_breach_text(state: State<'_, AppState>, password: String) -> Result<u64> {
    let password = Zeroizing::new(password);
    breach_enabled(&state)?;
    pwned_count(&password).await
}

#[tauri::command]
pub fn open_url(app: AppHandle, url: String) -> Result<()> {
    use tauri_plugin_opener::OpenerExt;
    let u = url.trim();
    let u = if u.contains("://") { u.to_string() } else { format!("https://{u}") };
    if !(u.starts_with("https://") || u.starts_with("http://")) {
        return Err(Error::invalid("Only web links can be opened."));
    }
    app.opener().open_url(u, None::<&str>).map_err(|e| Error::Other(e.to_string()))
}
