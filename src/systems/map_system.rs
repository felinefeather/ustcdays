use serde::Deserialize;
use anyhow::{Result,anyhow};
use std::collections::HashMap;

use crate::{events::conditions::Condition, player::Player};

#[allow(dead_code)]

#[derive(Debug, Deserialize, Clone)]
pub struct Connection {
    pub to: String,
    pub time: u32, // travel time in minutes
    pub optional_name: Option<String>,
    pub condition: Option<Condition>,
}
#[allow(dead_code)]

#[derive(Debug, Deserialize, Clone)]
pub struct Map {
    pub name: String,
    pub displayed_name: Option<String>,
    pub description: Option<String>,
    pub connections: Vec<Connection>,
}

pub struct MapSystem {
    pub maps: HashMap<String, Map>,
}

impl MapSystem {
    pub fn new(maps: &Vec<Map>) -> Self {
        let mut map_hash = HashMap::new();
        for map in maps {
            map_hash.insert(map.name.clone(), map.clone());
        }

        Self {
            maps: map_hash,
        }
    }

    pub fn travel(
        &self, player: &mut Player,
        to: &str,
        time_system: &super::time_system::TimeSystem,
    ) -> Result<()> {
        let current_map = self
            .maps
            .get(&player.game_map)
            .ok_or(anyhow!("当前地图不存在"))?;
        if let Some(conn) = current_map.connections.iter().find(|c| c.to == to) {
            // 处理旅行时间
            for _ in 0..conn.time {
                time_system.update(player);
            }
            player.game_map = to.to_string();
            Ok(())
        } else {
            Err(anyhow!("无法到达目标地图"))
        }
    }

    pub fn get_maps(&self) -> Vec<&Map> {
        self.maps.values().collect()
    }
}
