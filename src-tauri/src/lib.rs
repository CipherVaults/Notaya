mod backup;
mod commands;
mod crypto;
mod error;
mod generator;
mod model;
mod openvault;
mod portable;
mod restore;
mod state;
mod system;
mod ui;
mod vault;

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{http, Manager};

use commands::{backup as backup_cmd, notes, stickies, tools, vault as vault_cmd};

/// Set once the main window's close has started, so a second request is ignored.
static CLOSING: AtomicBool = AtomicBool::new(false);

/// Resets `CLOSING` when the close thread ends, unless the app is exiting. It also
/// runs if the thread panics, so a failed close can never block later ones.
struct ClosingGuard {
    exiting: bool,
}

impl Drop for ClosingGuard {
    fn drop(&mut self) {
        if !self.exiting {
            CLOSING.store(false, Ordering::SeqCst);
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let paths = portable::Paths::resolve();

    tauri::Builder::default()
        // Only one copy may run at a time, so two windows can't edit the vault at once.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.unminimize();
                let _ = w.show();
                let _ = w.set_focus();
            }
        }))
        // Dialog and opener are used only from Rust. The UI gets no permission for them.
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(state::AppState::new(paths.clone()))
        // Serves decrypted images to the editor at http://clatt.localhost/<id>,
        // only while the vault is unlocked. Nothing is written to disk.
        .register_uri_scheme_protocol("clatt", |ctx, request| {
            let state = ctx.app_handle().state::<state::AppState>();
            let id = request.uri().path().trim_start_matches('/').to_string();
            let response = match state.with(|s| s.get_attachment(&id)) {
                Ok((mime, bytes)) => http::Response::builder()
                    .status(200)
                    .header("Content-Type", mime)
                    .header("Cache-Control", "no-store")
                    .header("X-Content-Type-Options", "nosniff")
                    .body(bytes),
                Err(_) => http::Response::builder().status(404).body(Vec::new()),
            };
            response.unwrap_or_else(|_| http::Response::new(Vec::new()))
        })
        .setup(move |app| {
            if let Err(e) = paths.ensure_dirs() {
                // The folder next to the app can't be written (a read-only drive or folder).
                // The release exe has no console, so without a message it would just vanish.
                report_startup_failure(app.handle().clone(), data_folder_message(&paths, &e));
                return Ok(());
            }
            // Hidden from screenshots only if the vault has a master password: its lock screen
            // comes first. A vault with security off, or none yet, is never hidden (A21).
            let protect = paths.vault.exists()
                && vault::Session::read_header(&paths.vault).map(|h| !h.open).unwrap_or(true);
            if let Err(e) = ui::build_main(app.handle(), &paths, protect) {
                // Usually the Microsoft WebView2 component is missing or its folder can't be used.
                report_startup_failure(app.handle().clone(), window_message(&paths, &e));
                return Ok(());
            }
            state::spawn_monitor(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            // Closing the main window locks the vault and quits, sticky notes included.
            if let tauri::WindowEvent::Destroyed = event {
                // A closed window can't answer a flush request any more.
                window.app_handle().state::<state::AppState>().unmark_ready(window.label());
            }
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    // Don't close yet: the windows must save their edits first, and that
                    // needs this (event-loop) thread to stay free. Finish on another thread.
                    api.prevent_close();
                    if !CLOSING.swap(true, Ordering::SeqCst) {
                        let app = window.app_handle().clone();
                        std::thread::spawn(move || {
                            // Locks and exits. If this thread panics before the exit, the app
                            // stays open and the next close request may try again (see the guard).
                            let mut guard = ClosingGuard { exiting: false };
                            guard.exiting = state::quit(&app);
                        });
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            vault_cmd::vault_status,
            vault_cmd::start_notes,
            vault_cmd::enable_security,
            vault_cmd::disable_security,
            vault_cmd::unlock_vault,
            vault_cmd::lock_vault,
            vault_cmd::change_master,
            vault_cmd::pick_keyfile,
            vault_cmd::create_keyfile,
            vault_cmd::backup_now,
            vault_cmd::get_settings,
            vault_cmd::save_settings,
            vault_cmd::set_backdrop,
            vault_cmd::window_ready,
            vault_cmd::flush_done,
            vault_cmd::quit_app,
            vault_cmd::take_notice,
            vault_cmd::open_data_folder,
            backup_cmd::list_backups,
            backup_cmd::pick_backup_file,
            backup_cmd::restore_vault,
            backup_cmd::start_over,
            notes::list_notes,
            notes::create_notebook,
            notes::update_notebook,
            notes::delete_notebook,
            notes::create_section,
            notes::update_section,
            notes::delete_section,
            notes::create_page,
            notes::get_page,
            notes::save_page,
            notes::set_page_favorite,
            notes::set_page_look,
            notes::create_subpage,
            notes::move_page,
            notes::reorder,
            notes::trash_page,
            notes::restore_page,
            notes::delete_page,
            notes::empty_trash,
            notes::add_attachment,
            stickies::list_stickies,
            stickies::create_sticky,
            stickies::get_sticky,
            stickies::save_sticky,
            stickies::set_sticky_style,
            stickies::set_sticky_geometry,
            stickies::open_sticky,
            stickies::close_sticky,
            stickies::delete_sticky,
            stickies::delete_stickies,
            tools::password_strength,
            tools::open_url,
            tools::search,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            // No window and no console to tell the owner: leave without a crash report.
            eprintln!("Cipherleaf could not start: {e}");
            std::process::exit(1);
        });
}

/// The text shown when the data folder next to the app cannot be written to.
fn data_folder_message(paths: &portable::Paths, e: &std::io::Error) -> String {
    format!(
        "Cipherleaf could not start because it cannot save its data in {}. The folder may be \
         read-only (a locked USB stick, a CD, or a protected location such as Program Files).\n\n\
         Move the whole Cipherleaf folder somewhere you can write to, for example your \
         Documents folder, and start it again.\n\n({e})",
        paths.data.display()
    )
}

/// The text shown when the window itself could not be opened.
fn window_message(paths: &portable::Paths, e: &tauri::Error) -> String {
    format!(
        "Cipherleaf could not open its window. The usual cause is that the Microsoft WebView2 \
         Runtime is missing (Windows normally has it; if not, install the \"Evergreen\" runtime \
         from Microsoft's website, search for \"WebView2 Runtime\"). Cipherleaf also needs a \
         folder it can write to: {}. If that folder is read-only, move the whole Cipherleaf \
         folder somewhere you can write to, for example your Documents folder.\n\n\
         Then start Cipherleaf again.\n\n({e})",
        paths.data.display()
    )
}

/// Tells the owner, in a native message box, why Cipherleaf cannot start, then exits. It runs
/// on its own thread because the box needs the (already starting) event loop, which setup is
/// still holding.
fn report_startup_failure(app: tauri::AppHandle, text: String) {
    use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
    eprintln!("Cipherleaf could not start: {text}");
    std::thread::spawn(move || {
        let _ = app
            .dialog()
            .message(text)
            .title("Cipherleaf could not start")
            .kind(MessageDialogKind::Error)
            .blocking_show();
        std::process::exit(1);
    });
}
