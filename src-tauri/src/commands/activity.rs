use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub fn list_activity(state: State<'_, AppState>) -> Result<pyanpm_core::ActivityListResult, String> {
    state.service().activity_list().map_err(|error| error.to_string())
}
