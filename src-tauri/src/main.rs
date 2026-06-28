mod commands;
mod preferences;
mod state;

use commands::doctor::run_doctor;
use commands::activity::list_activity;
use commands::cache::{evict_cache, list_cache, prune_cache};
use commands::plugins::{
    add_plugin, diff_plugins, init_manifest, install_plugins, list_plugins, reinstall_plugin, remove_plugin,
    update_plugins, validate_source,
};
use commands::settings::{get_settings, save_settings};
use state::AppState;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            init_manifest,
            list_plugins,
            add_plugin,
            install_plugins,
            validate_source,
            diff_plugins,
            remove_plugin,
            reinstall_plugin,
            update_plugins,
            run_doctor,
            list_activity,
            list_cache,
            evict_cache,
            prune_cache
        ])
        .run(tauri::generate_context!())
        .expect("failed to run pyanPM");
}
