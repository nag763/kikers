use crate::error::ApplicationError;
use serde::{Deserialize, Serialize};
use crate::{translation::Model as Translation, translation};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Eq, Hash)]
struct TranslationManagerKey {
    label_name: String,
    locale_id: u32
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Model {
    mapped_translation : HashMap<TranslationManagerKey, String>
}

impl Model {
    pub fn translate(&self, label_name: &str, locale_id: u32) -> Result<&str, ApplicationError> {
        let translation : &String = self.mapped_translation.get(
            &TranslationManagerKey
                {
                    label_name: label_name.to_string(),
                    locale_id
                }
        ).ok_or(ApplicationError::TranslationError(label_name.into(), locale_id))?;
        Ok(translation.as_str())
    }
}

pub struct Entity;

impl Entity {
    pub async fn init() -> Result<Model, ApplicationError> {
        let translations : Vec<Translation> = translation::Entity::get_all().await?;
        let mut mapped_translation : HashMap<TranslationManagerKey, String> = HashMap::new();
        for translation in translations {
            mapped_translation.insert(TranslationManagerKey{label_name: translation.label_name, locale_id: translation.locale_id}, translation.translation);
        }
        Ok(Model { mapped_translation } )
    }
}
