// Persistência JSON (state.json / settings.json) no app data dir.
use serde::{de::DeserializeOwned, Serialize};
use std::path::PathBuf;

fn data_dir(app: &tauri::AppHandle) -> PathBuf {
    use tauri::Manager;
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn path_for(app: &tauri::AppHandle, name: &str) -> PathBuf {
    data_dir(app).join(name)
}

pub fn load<T: DeserializeOwned + Default>(app: &tauri::AppHandle, name: &str) -> T {
    let p = path_for(app, name);
    match std::fs::read(&p) {
        Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
        Err(_) => T::default(),
    }
}

pub fn save<T: Serialize>(app: &tauri::AppHandle, name: &str, value: &T) -> Result<(), String> {
    let dir = data_dir(app);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let p = path_for(app, name);
    let tmp = p.with_extension("tmp");
    let bytes = serde_json::to_vec(value).map_err(|e| e.to_string())?;
    std::fs::write(&tmp, bytes).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, &p).map_err(|e| e.to_string())
}
