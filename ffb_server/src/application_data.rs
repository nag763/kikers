use crate::ApplicationError;
use ffb_structs::{navaccess, navaccess::Model as NavAccess, role::Model as Role};
use std::collections::HashMap;

#[derive(Clone)]
pub struct ApplicationData {
    pub role_navaccess: HashMap<Role, Vec<NavAccess>>,
    pub jwt_path: String,
    pub cookie_approval_path: String,
}

impl ApplicationData {
    pub async fn init() -> Result<ApplicationData, ApplicationError> {
        info!("Begin of init of application data");
        let application_data: ApplicationData = ApplicationData {
            role_navaccess: navaccess::Entity::get_role_navaccess_mapping().await?,
            jwt_path: std::env::var("JWT_TOKEN_PATH")?,
            cookie_approval_path: std::env::var("COOKIE_APPROVAL_PATH")?,
        };
        info!("Application data initialized with succes :)");
        Ok(application_data)
    }

    pub fn get_navaccess_for_role(&self, role_id: &u32) -> Vec<NavAccess> {
        for (role, navaccess) in &self.role_navaccess {
            if &role.id == role_id {
                return navaccess.to_vec();
            }
        }
        vec![]
    }

    pub fn get_jwt_path(&self) -> &str {
        &self.jwt_path
    }

    pub fn get_cookie_approval_path(&self) -> &str {
        &self.cookie_approval_path
    }
}
