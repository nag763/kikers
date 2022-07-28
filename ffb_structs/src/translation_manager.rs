//! The translation manager is a model that is used to translate the labels
//! within the app.
//!
//! When inited, it provides a blazing fast way to translate the labels used
//! within the application templates.
//!
//! It is recommanded to init it at the startup and pass it as application data
//! to the different endpoints.

use crate::error::ApplicationError;
use crate::{translation, translation::Model as Translation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The keys to request a translation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Eq, Hash)]
struct TranslationManagerKey {
    /// The name of the label to request, has to exist within the database
    /// otherwise it will panic.
    ///
    /// ie. HOME_WELCOME_LABEL
    label_name: String,
    /// The MySQL's locale id we want the translation for.
    locale_id: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Model {
    /// The list of translations has an hashmap in order to earn as much time
    /// possible.
    mapped_translation: HashMap<TranslationManagerKey, String>,
}

impl Model {
    /// Translates one label for the given locale.
    ///
    /// If the label isn't found, an error will be returned.
    ///
    /// # Arguments
    ///
    /// * label_name : the name of the label.
    /// * locale_id : the MySQL id of the locale.
    pub fn translate(&self, label_name: &str, locale_id: u32) -> Result<&str, ApplicationError> {
        let translation: &String = self
            .mapped_translation
            .get(&TranslationManagerKey {
                label_name: label_name.to_string(),
                locale_id,
            })
            .ok_or_else(|| ApplicationError::TranslationError(label_name.into(), locale_id))?;
        trace!("Label {} has been found for locale id {}", label_name, locale_id);
        Ok(translation.as_str())
    }
}

pub struct Entity;

impl Entity {

    /// Initialize the structure from the database.
    pub async fn init() -> Result<Model, ApplicationError> {
        let translations: Vec<Translation> = translation::Entity::get_all().await?;
        let mut mapped_translation: HashMap<TranslationManagerKey, String> = HashMap::new();
        for translation in translations {
            mapped_translation.insert(
                TranslationManagerKey {
                    label_name: translation.label_name,
                    locale_id: translation.locale_id,
                },
                translation.translation,
            );
        }
        debug!("The translation manager has been initialized with success");
        Ok(Model { mapped_translation })
    }
}
