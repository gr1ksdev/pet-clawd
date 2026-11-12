// Comandos Tauri expostos à UI (ponte substitui o hack document.title).
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use crate::ai::{self, AiConfig};
use crate::brain::{ActResult, PetState};
use crate::store;

pub struct AppCtx {
    pub state: Mutex<PetState>,
    pub ai: Mutex<AiConfig>,
    pub counted_session: AtomicBool,
}

#[derive(Serialize)]
pub struct BackupResult {
    pub path: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct StateResult {
    pub state: PetState,
    pub speech: String,
    pub mood_label: String,
}

fn save_all(app: &tauri::AppHandle, ctx: &AppCtx) {
    if let Ok(s) = ctx.state.lock() {
        let _ = store::save(app, "state.json", &*s);
    }
}

#[tauri::command]
pub fn get_state(app: tauri::AppHandle, ctx: tauri::State<AppCtx>) -> Result<StateResult, String> {
    let mut s = ctx.state.lock().map_err(|e| e.to_string())?;
    s.tick();
    if !ctx.counted_session.swap(true, Ordering::Relaxed) {
        s.stats.sessions += 1;
    }
    let speech = s.idle_speech();
    let mood_label = s.mood_label().to_string();
    let state = s.clone();
    drop(s);
    save_all(&app, &ctx);
    Ok(StateResult {
        state,
        speech,
        mood_label,
    })
}

#[tauri::command]
pub fn act(
    app: tauri::AppHandle,
    ctx: tauri::State<AppCtx>,
    kind: String,
    item: Option<String>,
) -> Result<ActResult, String> {
    let mut s = ctx.state.lock().map_err(|e| e.to_string())?;
    let result = s.act(&kind, item.as_deref());
    drop(s);
    save_all(&app, &ctx);
    Ok(result)
}

#[tauri::command]
pub fn get_settings(ctx: tauri::State<AppCtx>) -> Result<AiConfig, String> {
    let cfg = ctx.ai.lock().map_err(|e| e.to_string())?;
    Ok(cfg.clone())
}

#[tauri::command]
pub fn set_settings(
    app: tauri::AppHandle,
    ctx: tauri::State<AppCtx>,
    cfg: AiConfig,
) -> Result<(), String> {
    {
        let mut guard = ctx.ai.lock().map_err(|e| e.to_string())?;
        *guard = cfg;
    }
    if let Ok(c) = ctx.ai.lock() {
        let _ = store::save(&app, "settings.json", &*c);
    }
    Ok(())
}

#[tauri::command]
pub async fn ai_talk(
    ctx: tauri::State<'_, AppCtx>,
    text: Option<String>,
) -> Result<String, String> {
    let cfg = {
        let guard = ctx.ai.lock().map_err(|e| e.to_string())?;
        guard.clone()
    };
    let prompt = text.unwrap_or_else(|| "Reaja a algo fofo que aconteceu hoje".into());
    ai::talk(&cfg, &prompt).await
}

#[tauri::command]
pub fn backup_export(
    app: tauri::AppHandle,
    ctx: tauri::State<AppCtx>,
) -> Result<BackupResult, String> {
    use tauri::Manager;
    let state = {
        let s = ctx.state.lock().map_err(|e| e.to_string())?;
        s.clone()
    };
    let content =
        serde_json::to_string_pretty(&state).map_err(|e| e.to_string())?;
    let dir = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("backups");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let ts = PetState::now();
    let path = dir.join(format!("ClawdBackup-{}.clawd", ts));
    std::fs::write(&path, &content).map_err(|e| e.to_string())?;
    Ok(BackupResult {
        path: path.display().to_string(),
        content,
    })
}

#[tauri::command]
pub fn backup_import(
    app: tauri::AppHandle,
    ctx: tauri::State<AppCtx>,
    content: String,
) -> Result<PetState, String> {
    let state: PetState =
        serde_json::from_str(&content).map_err(|_| "Backup inválido (.clawd)".to_string())?;
    {
        let mut s = ctx.state.lock().map_err(|e| e.to_string())?;
        *s = state.clone();
    }
    save_all(&app, &ctx);
    Ok(state)
}
