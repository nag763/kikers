use crate::ApplicationError;
use ffb_structs::{navaccess, navaccess::Model as NavAccess};
use std::collections::HashMap;

#[derive(Clone)]
pub struct ApplicationData {
    pub role_navaccess: HashMap<u32, Vec<NavAccess>>,
    pub jwt_path: String,
    pub cookie_approval_path: String,
}

impl ApplicationData {
    pub async fn init() -> Result<ApplicationData, ApplicationError> {
        info!("Begin of init of application data");
        let application_data : ApplicationData = ApplicationData {
            role_navaccess: navaccess::Entity::get_role_navaccess_mapping().await?,
            jwt_path: std::env::var("JWT_TOKEN_PATH")?,
            cookie_approval_path: std::env::var("COOKIE_APPROVAL_PATH")?,
        };
        info!("Application data initialized with succes :)");
        Ok(application_data)
    }

    pub fn get_navaccess_for_role(&self, role_id: &u32) -> Vec<NavAccess> {
        self.role_navaccess.get(role_id).unwrap_or(&vec![]).to_vec()
    }

    pub fn get_jwt_path(&self) -> &str {
        &self.jwt_path
    }

    pub fn get_cookie_approval_path(&self) -> &str {
        &self.cookie_approval_path
    }
}
