mod commands;
mod db;
mod error;
mod images;
mod launcher;
mod library;
mod metadata;
mod net;
mod settings;
mod steam;

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
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            let conn = db::open(&dir.join("alexandria.db"))?;
            log::info!("banco aberto em {}", dir.display());
            app.manage(commands::AppState {
                db: Mutex::new(conn),
                data_dir: dir,
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
            commands::library::launch_game,
            commands::steam::steam_status,
            commands::steam::steam_sync,
            commands::metadata::sgdb_search,
            commands::metadata::sgdb_images,
            commands::metadata::apply_image,
            commands::metadata::apply_local_image,
            commands::metadata::enrich_steam_game,
            commands::metadata::igdb_search,
            commands::settings::get_settings,
            commands::settings::set_setting,
            commands::settings::set_app_background,
        ])
        .run(tauri::generate_context!())
        .expect("falha ao iniciar o ALEXANDRIA");
}
