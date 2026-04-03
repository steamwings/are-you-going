use rand::Rng;
use sea_orm::{entity::prelude::*, ActiveValue};
use serde::{Deserialize, Serialize};

use super::_entities::events::{self, ActiveModel, Entity};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FieldConfig {
    #[serde(default = "default_true")]
    pub show_name: bool,
    #[serde(default = "default_true")]
    pub show_party_size: bool,
    #[serde(default)]
    pub show_kids_count: bool,
    #[serde(default)]
    pub show_allergies: bool,
    #[serde(default)]
    pub custom_prompt: Option<String>,
}

fn default_true() -> bool {
    true
}

impl Default for FieldConfig {
    fn default() -> Self {
        Self {
            show_name: true,
            show_party_size: true,
            show_kids_count: false,
            show_allergies: false,
            custom_prompt: None,
        }
    }
}

impl events::Model {
    pub fn field_config(&self) -> FieldConfig {
        serde_json::from_str(&self.field_config).unwrap_or_default()
    }
}

pub fn generate_slug() -> String {
    let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyz0123456789".chars().collect();
    let mut rng = rand::thread_rng();
    (0..6).map(|_| chars[rng.gen_range(0..chars.len())]).collect()
}

pub fn validate_custom_slug(slug: &str) -> bool {
    let len = slug.len();
    (3..=32).contains(&len)
        && slug
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
}

impl ActiveModel {
    pub fn new(
        name: &str,
        description: &str,
        slug: &str,
        field_config: &FieldConfig,
    ) -> Self {
        let config_json = serde_json::to_string(field_config).unwrap_or_default();
        Self {
            name: ActiveValue::set(name.to_string()),
            description: ActiveValue::set(description.to_string()),
            slug: ActiveValue::set(slug.to_string()),
            field_config: ActiveValue::set(config_json),
            ..Default::default()
        }
    }
}

pub async fn find_by_slug(db: &DatabaseConnection, slug: &str) -> Result<Option<events::Model>, DbErr> {
    Entity::find()
        .filter(events::Column::Slug.eq(slug))
        .one(db)
        .await
}

pub async fn slug_exists(db: &DatabaseConnection, slug: &str) -> Result<bool, DbErr> {
    let count = Entity::find()
        .filter(events::Column::Slug.eq(slug))
        .count(db)
        .await?;
    Ok(count > 0)
}
