mod backup;
mod commands;
mod db;
mod discord;
mod downloads;
mod emulators;
mod error;
mod images;
mod launcher;
mod library;
mod metadata;
mod net;
mod platforms;
mod safe;
mod screenshots;
mod session;
mod settings;
mod sources;
mod stats;
mod steam;
mod wishlist;

use std::collections::HashMap;
use std::sync::Mutex;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, None))
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            let conn = db::open(&dir.join("alexandria.db"))?;
            log::info!("banco aberto em {}", dir.display());
            app.manage(commands::AppState {
                db: Mutex::new(conn),
                data_dir: dir,
                installing: Mutex::new(HashMap::new()),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::library::list_games,
            commands::library::add_game,
            commands::library::update_game,
            commands::library::set_favorite,
            commands::library::remove_game,
            commands::library::clear_image,
            commands::library::set_status,
            commands::library::set_rating,
            commands::library::launch_game,
            commands::steam::steam_status,
            commands::steam::steam_sync,
            commands::metadata::sgdb_search,
            commands::metadata::sgdb_images,
            commands::metadata::apply_image,
            commands::metadata::apply_local_image,
            commands::metadata::enrich_steam_game,
            commands::metadata::igdb_search,
            commands::metadata::igdb_images,
            commands::metadata::igdb_discover,
            commands::extras::wishlist_list,
            commands::extras::wishlist_add,
            commands::extras::wishlist_remove,
            commands::platforms::platform_status,
            commands::platforms::platform_sync,
            commands::backup::export_backup,
            commands::backup::import_backup,
            commands::app::check_update,
            commands::app::open_external,
            commands::extras::year_stats,
            commands::extras::list_screenshots,
            commands::extras::list_emulators,
            commands::extras::add_emulator,
            commands::extras::remove_emulator,
            commands::sources::source_list,
            commands::sources::source_games,
            commands::sources::source_add,
            commands::sources::source_refresh,
            commands::sources::source_remove,
            commands::sources::install_game,
            commands::sources::pause_install,
            commands::sources::uninstall_game,
            commands::settings::get_settings,
            commands::settings::set_setting,
            commands::settings::set_app_background,
        ])
        .run(tauri::generate_context!())
        .expect("falha ao iniciar o ALEXANDRIA");
}
