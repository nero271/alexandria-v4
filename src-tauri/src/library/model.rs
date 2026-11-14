use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallState {
    Installed,
    NotInstalled,
    Unknown,
}

impl InstallState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Installed => "installed",
            Self::NotInstalled => "not_installed",
            Self::Unknown => "unknown",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "installed" => Self::Installed,
            "not_installed" => Self::NotInstalled,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub id: i64,
    pub name: String,
    pub platform: String,
    pub platform_id: Option<String>,
    pub launch_target: Option<String>,
    pub category: Option<String>,
    pub install_state: InstallState,
    pub favorite: bool,
    pub last_played_at: Option<i64>,
    pub description: Option<String>,
    pub cover_path: Option<String>,
    pub hero_path: Option<String>,
    pub background_path: Option<String>,
    pub logo_path: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub playtime_minutes: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewGame {
    pub name: String,
    pub platform: Option<String>,
    pub platform_id: Option<String>,
    pub launch_target: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,
}

/// Campos ausentes não mudam; texto vazio limpa campos opcionais.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameUpdate {
    pub name: Option<String>,
    pub platform: Option<String>,
    pub launch_target: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncSummary {
    pub added: usize,
    pub updated: usize,
    pub uninstalled: usize,
}
