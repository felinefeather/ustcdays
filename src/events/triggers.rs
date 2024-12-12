// triggers.rs

use std::collections::{HashMap, HashSet};

use crate::player::Player;
use crate::systems::map_system::MapSystem;
use crate::systems::time_system::TimeSystem;

use super::events::EventSystem;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct TriggerSystem {
    pub registed_event: HashMap<Trigger, Vec<String>>,
}

#[derive(Hash, Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub enum Trigger {
    Reached(String),
    Stay(String),

    Always,
    Custom(String),
}

impl TriggerSystem {
    pub fn get_all_events(&self, triggers: &HashSet<Trigger>) -> HashSet<String> {
        let mut all_events = HashSet::new(); // 使用 HashSet 来自动去重

        // 遍历 registed_event，收集所有事件
        for events in triggers.iter().filter_map(|k| self.registed_event.get(k)) {
            all_events.extend(events.iter().cloned());
        }

        // 转换为 Vec 并返回
        all_events
    }

    pub fn check(
        &mut self,
        triggers: &HashSet<Trigger>,
        time_system: &TimeSystem,
        map_system: &MapSystem,
        player: &Player,
        event_system: &mut EventSystem,
    ) -> Result<()> {
        let events = self.get_all_events(triggers);
        let mut to_reg = vec![];
        for event in events.iter().filter_map(|k| event_system.events.get(k)) {
            // 检查事件条件
            let mut conditions_met = true;
            for condition in &event.conditions {
                if !condition.is_met(time_system, map_system, player) {
                    conditions_met = false;
                    break;
                }
            }
            if conditions_met {
                to_reg.push(event.clone());
            }
        }
        for i in to_reg {
            dbg!(i.clone());
            event_system.register_event(i);
        }
        Ok(())
    }
}
