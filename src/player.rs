use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::events::{modifier::{Identity, ValModifier}, triggers::Trigger};

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
pub struct PlayerAttribute {
    pub val: Vec<(String,i32)>,
    // 这样的实现旨在节约内存——attributes作为直接给玩家看的属性，确实不需要太多条，如果不爽请修改成HashMap实现。
}

impl PlayerAttribute {
    pub fn get_mut(&mut self,k: &str) -> Option<&mut i32> {
        Some(&mut self.val.iter_mut().filter(|(str,_)| str != k).next()?.1)
    }
    pub fn get(&self,k: &str) -> Option<&i32> {
        Some(&self.val.iter().filter(|(str,_)| str != k).next()?.1)
    }

    pub fn id_mut(&mut self,k: &crate::events::modifier::Identity) -> Option<&mut i32> {
        match k {
            Identity::Str(k) => self.get_mut(k),
            Identity::Index(i) => self.val.get_mut(*i).map(|v| &mut v.1),
            Identity::None => None,
        }
    }

    pub fn id_mut_with_name<'a>(&'a mut self,k: &'a crate::events::modifier::Identity) -> Option<(&mut i32,&'a String)> {
        match k {
            Identity::Str(k) => self.get_mut(k).map(|i|(i,k)),
            Identity::Index(i) => self.val.get_mut(*i).map(|v| (&mut v.1,&v.0)),
            Identity::None => None,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String,&i32)> {
        self.val.iter().map(|f| (&f.0,&f.1))
    }
}

pub type PlayerItem = HashMap<String,(toml::Value,usize)>;

#[derive(Serialize,Deserialize,Default)]
pub struct Player {
    pub attributes: PlayerAttribute,
    pub attribute_defs: HashMap<String, Attribute>,

    pub items: PlayerItem,
    pub game_time: NaiveDateTime,
    pub game_map: String,
    pub cur_evt_seg: Option<(String, Option<String>)>,
    pub trigger: HashSet<Trigger>,
}

impl Player {
    pub fn new(attribute: &Vec<Attribute>) -> Self {
        let mut attributes = PlayerAttribute { val: vec![] };
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
                .unwrap(),
            game_map: "Town".to_string(),

            trigger: {
                let mut trigger = HashSet::new();
                trigger.insert(Trigger::Init);
                trigger
            },
            cur_evt_seg: None,
        }
    }

    pub fn modify_attribute(&mut self, attr: &Identity, value: &ValModifier) {
        if let Some((current,k)) = self.attributes.id_mut_with_name(attr) {
            value.apply(current);
            // 检查属性上限和下限
            if *current > self.attribute_defs.get(k).unwrap().max {
                *current = self.attribute_defs.get(k).unwrap().max;
            }
            if *current < self.attribute_defs.get(k).unwrap().min {
                *current = self.attribute_defs.get(k).unwrap().min;
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
