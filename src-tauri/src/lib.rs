mod commands;
mod crypto;
mod embed;
mod error;
mod generator;
mod keepassxc;
mod model;
mod portable;
mod state;
mod system;
mod ui;
mod vault;

use tauri::{http, Manager};

use commands::{keepassxc as keepassxc_cmd, notes, stickies, tools, vault as vault_cmd};

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
            paths.ensure_dirs()?;
            ui::build_main(app.handle(), &paths)?;
            state::spawn_monitor(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            // Closing the main window locks the vault and quits, sticky notes included.
            if window.label() == "main" {
                // Keep the embedded KeePassXC glued to the Passwords area (amendment A10).
                if matches!(event, tauri::WindowEvent::Moved(_) | tauri::WindowEvent::Resized(_)) {
                    embed::follow();
                }
                if let tauri::WindowEvent::CloseRequested { .. } = event {
                    state::lock(window.app_handle(), "quit");
                    keepassxc::shutdown(&window.app_handle().state::<state::AppState>().keepassxc);
                    window.app_handle().exit(0);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            vault_cmd::vault_status,
            vault_cmd::create_vault,
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
            vault_cmd::quit_app,
            vault_cmd::open_data_folder,
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
            keepassxc_cmd::keepassxc_status,
            keepassxc_cmd::keepassxc_open,
            keepassxc_cmd::keepassxc_embed,
            keepassxc_cmd::keepassxc_place,
            keepassxc_cmd::keepassxc_hide,
            keepassxc_cmd::keepassxc_detach,
            keepassxc_cmd::keepassxc_lock,
            tools::generate_passphrase,
            tools::password_strength,
            tools::open_url,
            tools::search,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Cipherleaf");
}
