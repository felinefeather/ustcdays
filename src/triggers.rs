// triggers.rs

use crate::time_system::TimeSystem;
use crate::map_system::MapSystem;
use crate::player::Player;
use crate::events::EventSystem;
use anyhow::Result;

pub struct TriggerSystem;

impl TriggerSystem {
    pub fn new() -> Self {
        Self
    }

    pub fn check(&self, time_system: &TimeSystem, map_system: &MapSystem, player: &Player, event_system: &mut EventSystem) -> Result<()> {
        let mut to_reg = vec![];
        for event in &event_system.events {
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
            event_system.register_event(i);
        }
        Ok(())
    }
}
