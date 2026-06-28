use std::sync::RwLock;

use serde::Serialize;

use crate::preferences::{detected_plugins_dir, DesktopPreferences};

pub struct AppState {
    preferences: RwLock<DesktopPreferences>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopSettingsPayload {
    pub state_dir: String,
    pub default_state_dir: Option<String>,
    pub plugins_dir_override: Option<String>,
    pub detected_plugins_dir: Option<String>,
    pub last_route: String,
    pub notifications_enabled: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            preferences: RwLock::new(DesktopPreferences::load()),
        }
    }

    pub fn settings_payload(&self) -> DesktopSettingsPayload {
        let preferences = self.preferences.read().expect("preferences read lock");
        let detected_plugins_dir = detected_plugins_dir();
        let default_state_dir = pyanpm_core::config::global_state_dir()
            .ok()
            .map(|path| path.display().to_string());
        let state_dir = preferences
            .state_dir_override
            .clone()
            .or_else(|| default_state_dir.clone())
            .unwrap_or_else(|| "Unavailable".to_owned());
        DesktopSettingsPayload {
            state_dir,
            default_state_dir,
            plugins_dir_override: preferences
                .plugins_dir_override
                .clone()
                .or_else(|| detected_plugins_dir.clone()),
            detected_plugins_dir,
            last_route: preferences.last_route.clone(),
            notifications_enabled: preferences.notifications_enabled,
        }
    }

    pub fn update_preferences(
        &self,
        state_dir_override: Option<String>,
        plugins_dir_override: Option<String>,
        last_route: String,
        notifications_enabled: bool,
    ) -> Result<DesktopSettingsPayload, String> {
        let normalized_state_dir_override = state_dir_override.and_then(|path| {
            let trimmed = path.trim().to_owned();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        });
        let detected_plugins_dir = detected_plugins_dir();
        let normalized_plugins_dir_override = plugins_dir_override
            .and_then(|path| {
                let trimmed = path.trim().to_owned();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                }
            })
            .or_else(|| detected_plugins_dir.clone());
        let default_state_dir = pyanpm_core::config::global_state_dir()
            .ok()
            .map(|path| path.display().to_string());
        let state_dir = normalized_state_dir_override
            .clone()
            .or_else(|| default_state_dir.clone())
            .ok_or_else(|| "Failed to resolve the Global Config path.".to_owned())?;

        let payload = DesktopSettingsPayload {
            state_dir,
            default_state_dir,
            plugins_dir_override: normalized_plugins_dir_override,
            detected_plugins_dir,
            last_route,
            notifications_enabled,
        };

        let mut preferences = self.preferences.write().expect("preferences write lock");
        preferences.state_dir_override = normalized_state_dir_override;
        preferences.plugins_dir_override = payload.plugins_dir_override.clone();
        preferences.last_route = payload.last_route.clone();
        preferences.notifications_enabled = payload.notifications_enabled;
        preferences.save()?;
        Ok(payload)
    }

    pub fn service(&self) -> pyanpm_core::PyanpmService {
        let preferences = self.preferences.read().expect("preferences read lock");
        let state_dir = preferences
            .state_dir_override
            .clone()
            .or_else(|| {
                pyanpm_core::config::global_state_dir()
                    .ok()
                    .map(|path| path.display().to_string())
            })
            .expect("global config path");
        pyanpm_core::PyanpmService::new(state_dir, preferences.plugins_dir_override.as_ref().map(Into::into))
    }
}
