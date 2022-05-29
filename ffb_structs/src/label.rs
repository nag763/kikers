use crate::database::Database;
use crate::error::ApplicationError;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
enum Model {
    MENU_WELCOME_BACK,
    MENU_ROOT
}

