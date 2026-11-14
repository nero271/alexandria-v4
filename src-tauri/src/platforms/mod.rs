//! Outras lojas além da Steam. Cada plataforma só precisa devolver uma lista de `ExternalApp`.
pub mod epic;
pub mod gog;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalApp {
    pub app_id: String,
    pub name: String,
    pub launch_target: Option<String>,
    pub playtime_minutes: Option<i64>,
}
