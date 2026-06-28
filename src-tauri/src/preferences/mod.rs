use std::path::PathBuf;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use pyanpm_core::install::resolve_plugins_dir;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopPreferences {
    #[serde(default)]
    pub state_dir_override: Option<String>,
    #[serde(default)]
    pub plugins_dir_override: Option<String>,
    #[serde(default)]
    pub last_route: String,
    #[serde(default = "default_notifications_enabled")]
    pub notifications_enabled: bool,
}

impl DesktopPreferences {
    pub fn load() -> Self {
        let path = preferences_path();
        let content = std::fs::read_to_string(&path).ok();
        let mut preferences = content
            .and_then(|raw| toml::from_str::<DesktopPreferences>(&raw).ok())
            .unwrap_or_else(Self::default_from_env);

        if preferences.last_route.is_empty() {
            preferences.last_route = "/".to_owned();
        }
        if preferences
            .state_dir_override
            .as_ref()
            .is_some_and(|path| path.trim().is_empty())
        {
            preferences.state_dir_override = None;
        }
        if preferences
            .plugins_dir_override
            .as_ref()
            .is_none_or(|path| path.trim().is_empty())
        {
            preferences.plugins_dir_override = detected_plugins_dir();
        }

        preferences
    }

    pub fn save(&self) -> Result<(), String> {
        let path = preferences_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let serialized = toml::to_string_pretty(self).map_err(|error| error.to_string())?;
        std::fs::write(path, serialized).map_err(|error| error.to_string())
    }

    fn default_from_env() -> Self {
        Self {
            state_dir_override: None,
            plugins_dir_override: detected_plugins_dir(),
            last_route: "/".to_owned(),
            notifications_enabled: default_notifications_enabled(),
        }
    }
}

fn default_notifications_enabled() -> bool {
    true
}

fn preferences_path() -> PathBuf {
    if let Some(project_dirs) = ProjectDirs::from("dev", "bblocks", "pyanpm") {
        return project_dirs.config_dir().join("desktop.toml");
    }

    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".pyanpm-app.toml")
}

pub fn detected_plugins_dir() -> Option<String> {
    resolve_plugins_dir(None)
        .ok()
        .map(|path| path.display().to_string())
}
