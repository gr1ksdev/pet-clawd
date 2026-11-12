mod ai;
mod brain;
mod commands;
mod store;

use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            let state = store::load::<brain::PetState>(&handle, "state.json");
            let ai_cfg = store::load::<ai::AiConfig>(&handle, "settings.json");
            app.manage(commands::AppCtx {
                state: Mutex::new(state),
                ai: Mutex::new(ai_cfg),
                counted_session: AtomicBool::new(false),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_state,
            commands::act,
            commands::get_settings,
            commands::set_settings,
            commands::ai_talk,
            commands::backup_export,
            commands::backup_import,
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o Pet Clawd");
}
