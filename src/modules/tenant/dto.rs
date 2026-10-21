use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UpdateTenantRequest {
    pub name: Option<String>,
    pub city: Option<String>,
    pub phone: Option<String>,
}
