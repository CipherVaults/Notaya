mod backup;
mod commands;
mod crypto;
mod database;
mod error;
mod export;
mod filenames;
mod importers;
mod generator;
mod import;
mod legacy_guard;
mod logging;
mod model;
mod openvault;
mod portable;
mod restore;
#[cfg(test)]
mod roundtrip_tests;
mod shell;
mod state;
mod system;
mod ui;
mod vault;

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{http, Manager};

use commands::{backup as backup_cmd, calculator, database as database_cmd, database_io, errorlog, export as export_cmd, import_notes as import_notes_cmd, notes, shell as shell_cmd, stickies, tools, vault as vault_cmd};

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

/// The app handle, for the guard against an older Notaya (it may be asked to show the window).
static APP: std::sync::OnceLock<tauri::AppHandle> = std::sync::OnceLock::new();

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // First of all: a panic anywhere after this line is written to the error log before the process aborts.
    logging::install_panic_hook();

    // An older Notaya (V1.3.0 and earlier) and this one must not open the same notes together:
    // take the names the old copy's guard uses, or hand over to a copy that holds them.
    #[cfg(windows)]
    match legacy_guard::claim(legacy_guard::OLD_ID, || {
        if let Some(app) = APP.get() {
            shell::show_main(app);
        }
    }) {
        Ok(legacy_guard::Claim::Held) => {}
        Ok(legacy_guard::Claim::Handed) => {
            // A copy of this version that is already running was asked to show itself. If the copy that
            // holds the names is an OLDER Notaya, say so: its window would otherwise look like the update.
            if !legacy_guard::mutex_exists("app.notaya.notes-sim") {
                legacy_guard::tell(
                    "Notaya",
                    "An older version of Notaya is already running. Close it first (use Quit from its window or its tray icon), then start Notaya again. Your notes were not changed.",
                );
            }
            std::process::exit(0)
        }
        Err(e) => eprintln!("start: could not set up the guard against an older Notaya ({e})"),
    }

    let paths = portable::Paths::resolve();

    tauri::Builder::default()
        // Only one copy may run at a time, so two windows can't edit the vault at once.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            shell::show_main(app);
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
                // Only pictures are shown with their own type. A file attached to a database row is
                // sent as plain data and as a download, so it can never run as a page or a script.
                Ok((mime, bytes)) => {
                    let picture = matches!(mime.as_str(), "image/png" | "image/jpeg" | "image/gif" | "image/webp");
                    let mut b = http::Response::builder()
                        .status(200)
                        .header("Content-Type", if picture { mime.as_str() } else { "application/octet-stream" })
                        .header("Cache-Control", "no-store")
                        .header("X-Content-Type-Options", "nosniff")
                        .header("Content-Security-Policy", "default-src 'none'; sandbox");
                    if !picture {
                        b = b.header("Content-Disposition", "attachment");
                    }
                    b.body(bytes)
                }
                Err(_) => http::Response::builder().status(404).body(Vec::new()),
            };
            response.unwrap_or_else(|_| http::Response::new(Vec::new()))
        })
        .setup(move |app| {
            let _ = APP.set(app.handle().clone());
            // After the single-instance check (a second copy never gets here), so a second start can't look like a crash.
            logging::start(&paths);
            // Plain copies of row files opened in another program are removed at the next start.
            database_io::clean_open_copies();
            if let Err(e) = paths.ensure_dirs() {
                // The folder next to the app can't be written (a read-only drive or folder).
                // The release exe has no console, so without a message it would just vanish.
                logging::start_failed(true, Some(e.kind()));
                report_startup_failure(app.handle().clone(), data_folder_message(&paths, &e));
                return Ok(());
            }
            // Hidden from screenshots only if the vault has a master password: its lock screen
            // comes first. A vault with security off, or none yet, is never hidden (A21).
            let protect = paths.vault.exists()
                && vault::Session::read_header(&paths.vault).map(|h| !h.open).unwrap_or(true);
            if let Err(e) = ui::build_main(app.handle(), &paths, protect) {
                // Usually the Microsoft WebView2 component is missing or its folder can't be used.
                logging::start_failed(false, None);
                report_startup_failure(app.handle().clone(), window_message(&paths, &e));
                return Ok(());
            }
            // The tray icon comes first: the window may only hide itself while there is a way back.
            let state = app.state::<state::AppState>();
            match shell::build_tray(app.handle()) {
                Ok(()) => state.set_tray_ready(true),
                Err(e) => eprintln!("no tray icon: {e}"),
            }
            if shell::started_minimized() && state.tray_ready() {
                state.set_tray_hidden(true);
            }
            state::spawn_monitor(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            // Closing the main window hides Notaya to the tray (the default; locking a vault that
            // has a master password first), or locks the vault and quits, sticky notes included,
            // when "close to tray" is off or there is no tray icon.
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
                            let state = app.state::<state::AppState>();
                            if state.tray_ready() && shell::load_prefs(&state.paths).close_to_tray {
                                let failure = state::hide_to_tray(&app);
                                // The close button works again before the message box is shown (it waits for a click).
                                drop(guard);
                                if let Some(text) = failure {
                                    // Told now; the stored copy would show up as a stale toast later.
                                    let _ = state.take_notice();
                                    state::show_backup_failure(&app, &text);
                                }
                            } else {
                                guard.exiting = state::quit(&app);
                            }
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
            errorlog::log_frontend_error,
            errorlog::log_info,
            errorlog::open_log_folder,
            shell_cmd::get_shell_prefs,
            shell_cmd::set_shell_prefs,
            backup_cmd::list_backups,
            backup_cmd::pick_backup_file,
            backup_cmd::restore_vault,
            backup_cmd::start_over,
            backup_cmd::import_portable_notes,
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
            database_cmd::create_database,
            database_cmd::get_database,
            database_cmd::save_database,
            database_cmd::query_view,
            database_cmd::create_row,
            database_cmd::set_cells,
            database_cmd::duplicate_row,
            database_cmd::reorder_rows,
            database_cmd::search_rows,
            database_cmd::row_titles,
            database_cmd::set_database_look,
            database_cmd::move_database,
            database_cmd::enable_sub_items,
            database_cmd::enable_dependencies,
            database_cmd::chart_data,
            database_io::import_csv,
            database_io::export_csv,
            database_io::export_database,
            database_io::attach_file,
            database_io::open_file,
            database_io::save_file_as,
            export_cmd::export_notes,
            import_notes_cmd::import_notes,
            import_notes_cmd::import_cancel,
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
            calculator::open_calculator,
            calculator::calc_history,
            calculator::add_calc_entry,
            calculator::clear_calc_history,
            tools::password_strength,
            tools::open_url,
            tools::search,
        ])
        .build(tauri::generate_context!())
        .unwrap_or_else(|e| {
            // No window and no console to tell the owner: leave without a crash report.
            eprintln!("Notaya could not start: {e}");
            std::process::exit(1);
        })
        .run(|_app, event| {
            // Every normal end (the Quit button, the tray's Quit, the last window closing) passes here.
            if let tauri::RunEvent::Exit = event {
                logging::clean_exit();
            }
        });
}

/// The text shown when the data folder next to the app cannot be written to.
fn data_folder_message(paths: &portable::Paths, e: &std::io::Error) -> String {
    format!(
        "Notaya could not start because it cannot save its data in {}. The folder may be \
         read-only (a locked USB stick, a CD, or a protected location such as Program Files).\n\n\
         Move the whole Notaya folder somewhere you can write to, for example your \
         Documents folder, and start it again.\n\n({e})",
        paths.data.display()
    )
}

/// The text shown when the window itself could not be opened.
fn window_message(paths: &portable::Paths, e: &tauri::Error) -> String {
    format!(
        "Notaya could not open its window. The usual cause is that the Microsoft WebView2 \
         Runtime is missing (Windows normally has it; if not, install the \"Evergreen\" runtime \
         from Microsoft's website, search for \"WebView2 Runtime\"). Notaya also needs a \
         folder it can write to: {}. If that folder is read-only, move the whole Notaya \
         folder somewhere you can write to, for example your Documents folder.\n\n\
         Then start Notaya again.\n\n({e})",
        paths.data.display()
    )
}

/// Tells the owner, in a native message box, why Notaya cannot start, then exits. It runs
/// on its own thread because the box needs the (already starting) event loop, which setup is
/// still holding.
fn report_startup_failure(app: tauri::AppHandle, text: String) {
    use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
    eprintln!("Notaya could not start: {text}");
    std::thread::spawn(move || {
        let _ = app
            .dialog()
            .message(text)
            .title("Notaya could not start")
            .kind(MessageDialogKind::Error)
            .blocking_show();
        logging::clean_exit();
        std::process::exit(1);
    });
}
