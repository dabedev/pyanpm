use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub fn run_doctor(state: State<'_, AppState>, write_probe: Option<bool>) -> Result<pyanpm_core::DoctorResult, String> {
    state
        .service()
        .doctor_with_options(write_probe.unwrap_or(false))
        .map_err(|error| error.to_string())
}
