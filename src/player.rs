use serde::Deserialize;
use std::collections::{HashMap, HashSet};

use crate::events::triggers::Trigger;

#[derive(Debug, Deserialize, Clone)]
pub struct Attribute {
    pub name: String,
    pub max: i32,
    pub min: i32,

    #[serde(default)]
    pub default: i32,
    #[serde(default)]
    pub over_max: i32,
    #[serde(default)]
    pub under_min: i32,
    #[serde(default)]
    pub over_max_desc: String,
    #[serde(default)]
    pub under_min_desc: String,

    #[serde(default)]
    pub invisible: bool,
}

pub struct Player {
    pub attributes: HashMap<String, i32>,
    pub attribute_defs: HashMap<String, Attribute>,

    pub items: HashMap<String,(toml::Value,usize)>,
    pub game_time: chrono::NaiveDateTime,
    pub game_map: String,
    pub trigger: HashSet<Trigger>,

    pub stuck_in_event: bool,
}

impl Player {
    pub fn new(attributes: &Vec<Attribute>) -> Self {
        let mut attr_map = HashMap::new();
        let mut defs_map = HashMap::new();
        for attr in attributes {
            attr_map.insert(attr.name.clone(), attr.default);
            defs_map.insert(attr.name.clone(), (*attr).clone());
        }

        Self {
            attributes: attr_map,
            attribute_defs: defs_map,
            items: HashMap::new(),
            game_time: chrono::NaiveDateTime::parse_from_str("2024-01-01 00:00", "%Y-%m-%d %H:%M")
                .unwrap(),
            game_map: "Town".to_string(),

            trigger: HashSet::new(),
            stuck_in_event: false,
        }
    }

    pub fn modify_attribute(&mut self, attr: &str, value: i32) {
        if let Some(current) = self.attributes.get_mut(attr) {
            *current += value;
            // 检查属性上限和下限
            if *current > self.attribute_defs.get(attr).unwrap().max {
                *current = self.attribute_defs.get(attr).unwrap().max;
            }
            if *current < self.attribute_defs.get(attr).unwrap().min {
                *current = self.attribute_defs.get(attr).unwrap().min;
            }
        }
    }

    pub fn get_over_under_descriptions(&self) -> Vec<String> {
        let mut descriptions = Vec::new();
        for (name, value) in &self.attributes {
            if let Some(def) = self.attribute_defs.get(name) {
                if def.invisible { continue; }
                if *value > def.over_max {
                    descriptions.push(def.over_max_desc.clone());
                }
                if *value < def.under_min {
                    descriptions.push(def.under_min_desc.clone());
                }
            }
            if descriptions.len() >= 2 {
                break;
            }
        }
        descriptions
    }
}
