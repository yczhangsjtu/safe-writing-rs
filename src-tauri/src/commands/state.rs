use crate::state::{AppState, AppStateResponse};
use tauri::Emitter;

#[tauri::command]
pub fn get_app_state(state: tauri::State<'_, AppState>) -> AppStateResponse {
    state.get_state()
}

/// Emit state change event to frontend
pub fn emit_state_change(app: &tauri::AppHandle, state: &AppState) {
    let state_response = state.get_state();
    app.emit("state-changed", state_response).ok();
}