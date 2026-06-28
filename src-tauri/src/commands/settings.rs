use serde::Deserialize;
use tauri::State;

use crate::state::{AppState, DesktopSettingsPayload};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSettingsInput {
    pub state_dir_override: Option<String>,
    pub plugins_dir_override: Option<String>,
    pub last_route: String,
    pub notifications_enabled: bool,
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> DesktopSettingsPayload {
    state.settings_payload()
}

#[tauri::command]
pub fn save_settings(
    state: State<'_, AppState>,
    input: SaveSettingsInput,
) -> Result<DesktopSettingsPayload, String> {
    state.update_preferences(
        normalize_optional_string(input.state_dir_override),
        normalize_optional_string(input.plugins_dir_override),
        input.last_route,
        input.notifications_enabled,
    )
}

fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value.and_then(|entry| {
        let trimmed = entry.trim().to_owned();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}
