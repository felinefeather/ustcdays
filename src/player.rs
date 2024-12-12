use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::events::triggers::Trigger;

#[derive(Debug, Deserialize, Serialize, Clone)]
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

#[derive(Serialize,Deserialize,Default)]
pub struct CurrentAttribute {
    pub val: Vec<(String,i32)>,
    // 这样的实现旨在节约内存——attributes作为直接给玩家看的属性，确实不需要太多条，如果不爽请修改成HashMap实现。
}

impl CurrentAttribute {
    pub fn get_mut(&mut self,k: &str) -> Option<&mut i32> {
        Some(&mut self.val.iter_mut().filter(|(str,_)| str != k).next()?.1)
    }
    pub fn get(&self,k: &str) -> Option<&i32> {
        Some(&self.val.iter().filter(|(str,_)| str != k).next()?.1)
    }
    pub fn iter(&self) -> impl Iterator<Item = (&String,&i32)> {
        self.val.iter().map(|f| (&f.0,&f.1))
    }
}

#[derive(Serialize,Deserialize,Default)]
pub struct Player {
    pub attributes: CurrentAttribute,
    pub attribute_defs: HashMap<String, Attribute>,

    pub items: HashMap<String,(toml::Value,usize)>,
    pub game_time: String,
    pub game_map: String,
    pub trigger: HashSet<Trigger>,

    pub stuck_in_event: bool,
}

impl Player {
    pub fn new(attribute: &Vec<Attribute>) -> Self {
        let mut attributes = CurrentAttribute { val: vec![] };
        let mut defs_map = HashMap::new();
        for attr in attribute.iter() {
            attributes.val.push((attr.name.clone(),attr.default));
            defs_map.insert(attr.name.clone(), (*attr).clone());
        }

        Self {
            attributes,
            attribute_defs: defs_map,
            items: HashMap::new(),
            game_time: chrono::NaiveDateTime::parse_from_str("2024-01-01 00:00", "%Y-%m-%d %H:%M")
                .unwrap().to_string(),
            game_map: "Town".to_string(),

            trigger: {
                let mut trigger = HashSet::new();
                trigger.insert(Trigger::Init);
                trigger
            },
            stuck_in_event: false,
        }
    }

    pub fn modify_attribute(&mut self, attr: &str, value: i32) {
        if let Some(current) = self.attributes.get_mut(attr) {
            *current += value;
            println!("modified: {attr}, {current}");
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
        for (name, value) in self.attributes.iter() {
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
