// triggers.rs

use std::collections::{BinaryHeap, HashMap, HashSet};

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

impl TriggerSystem {
    pub fn new(trigger: &Vec<HashMap<String,Trigger>>) -> Self { Self {
        registed_event: {
            let ret = trigger.iter()
                .fold(HashMap::new(), |mut map: HashMap<Trigger, Vec<String>>,value| {
                    for (key,value) in value {
                        if let Some(map) = map.get_mut(value) {
                            map.push(key.clone());
                        } else { map.insert(value.clone(), vec![key.clone()]);}
                    }
                    map
            });
            ret
        },
    }}
}

#[derive(Hash, Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(tag = "t",content = "c")]
pub enum Trigger {
    Reached(String),
    Stay(String),

    Always,
    Init,
    Custom(String),
}

impl TriggerSystem {
    pub fn get_all_events(&self, triggers: &HashSet<Trigger>) -> HashSet<String> {
        let mut all_events = HashSet::new(); // 使用 HashSet 来自动去重

        // 遍历 registed_event，收集所有事件
        for events in triggers.iter()
            .filter_map(|k| self.registed_event.get(k)) {
                all_events.extend(events.iter().cloned());
        }

        // 转换为 Vec 并返回
        all_events
    }

    pub fn set_default(
        triggers: &mut HashSet<Trigger>
    ) {
        triggers.insert(Trigger::Always);
    }

    pub fn pick_event(
        &mut self,
        player: &Player,
        time_system: &TimeSystem,
        map_system: &MapSystem,
        current_event_and_segment: &Option<(String, Option<String>)>,
        event_system: &mut EventSystem,
    ) -> Option<String> {
        let evt = self.get_all_events(&player.trigger);
        let (cur_priority, cur_force) = current_event_and_segment.as_ref().map(|cur|
            event_system.events.get(&cur.0).map(
                |evt| (evt.priority,evt.force)
            )).unwrap_or(Some((0,false))).unwrap();
        if cur_force { return None; } // 如果硬要执行，我们也不好阻止。
        let mut heap = BinaryHeap::new();
        for event in evt.iter().filter_map(|k| event_system.events.get(k)) {
            // 检查事件条件
            if event.condition.is_met(time_system, map_system, player)
              && (cur_priority == 0 || event.priority > cur_priority) {
                heap.push(event);
            }
        }
        heap.pop().map(|evt| evt.name.clone())
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
            if event.condition.is_met(time_system, map_system, player) {
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
