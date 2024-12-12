use serde::Deserialize;
use std::collections::HashMap;

use crate::events::conditions::Condition;

#[derive(Debug, Deserialize, Clone)]
pub struct Connection {
    pub to: String,
    pub time: u32, // travel time in minutes
    pub optional_name: Option<String>,
    pub condition: Option<Condition>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Map {
    pub name: String,
    pub displayed_name: Option<String>,
    pub description: Option<String>,
    pub connections: Vec<Connection>,
}

pub struct MapSystem {
    pub maps: HashMap<String, Map>,
    pub current_location: String,
}

impl MapSystem {
    pub fn new(maps: &Vec<Map>) -> Self {
        let mut map_hash = HashMap::new();
        for map in maps {
            map_hash.insert(map.name.clone(), map.clone());
        }

        Self {
            maps: map_hash,
            current_location: "Town".to_string(), // 默认起始位置
        }
    }

    pub fn get_current_location(&self) -> &str {
        &self.current_location
    }

    pub fn travel(
        &mut self,
        to: &str,
        time_system: &mut super::time_system::TimeSystem,
    ) -> Result<(), String> {
        let current_map = self
            .maps
            .get(&self.current_location)
            .ok_or("当前地图不存在")?;
        if let Some(conn) = current_map.connections.iter().find(|c| c.to == to) {
            // 处理旅行时间
            for _ in 0..conn.time {
                time_system.update();
            }
            self.current_location = to.to_string();
            Ok(())
        } else {
            Err("无法到达目标地图".to_string())
        }
    }

    pub fn get_maps(&self) -> Vec<&Map> {
        self.maps.values().collect()
    }
}
