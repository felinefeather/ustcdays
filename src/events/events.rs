use crate::frontend::Frontend;
use crate::game;
use crate::player::Player;
use crate::systems::map_system::MapSystem;
use crate::systems::time_system::TimeSystem;
use serde::Deserialize;

use super::conditions::Condition;
use super::triggers::Trigger;
use anyhow::Result;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct EventOption {
    pub text: String,
    #[serde(default)]
    pub condition: Option<Vec<Condition>>,
    #[serde(default)]
    pub jump_to: Option<String>,
    #[serde(default)]
    pub trigger: Option<Vec<Trigger>>,
    #[serde(default)]
    pub modifications: Option<HashMap<String, i32>>, // 属性修正
}

#[derive(Debug, Deserialize, Clone)]
pub struct EventSegment {
    pub name: String,
    pub text: String,
    pub options: Vec<EventOption>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EventData {
    pub name: String,
    pub priority: u32,
    pub force: bool,
    pub conditions: Vec<Condition>, // 修改为 Vec<Condition>
    pub segments: Vec<EventSegment>,

    #[serde(default)]
    pub stuck_moving: bool,
}

// 为 EventData 实现 Ord 和 PartialOrd，以便在 BinaryHeap 中按优先级排序
impl PartialEq for EventData {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl Eq for EventData {}

impl PartialOrd for EventData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for EventData {
    fn cmp(&self, other: &Self) -> Ordering {
        // BinaryHeap 是最大堆，因此需要反转比较以实现高优先级优先
        other.priority.cmp(&self.priority)
    }
}

pub struct EventSystem {
    pub events: HashMap<String, EventData>,
    registered_events: BinaryHeap<EventData>,
}

impl EventSystem {
    pub fn new(events: &Vec<EventData>) -> Self {
        Self {
            events: {
                let mut ret = HashMap::new();
                for evt in events {
                    ret.insert(evt.name.clone(), evt.clone());
                }
                ret
            },
            registered_events: BinaryHeap::new(),
        }
    }

    pub fn register_event(&mut self, event: EventData) {
        self.registered_events.push(event);
    }
} // events.rs

impl EventSystem {
    pub fn process_events(
        &mut self,
        current_event_and_segment: &mut Option<(String, Option<String>)>,
        player: &mut Player,
        time_system: &TimeSystem,
        map_system: &MapSystem,
        frontend: &mut Frontend,
    ) -> Result<(), game::GameErr> {
        player.stuck_in_event = false;
        let able_to_stuck;
        // 从优先级队列中取出优先级最高的事件
        if current_event_and_segment.is_none() {
            *current_event_and_segment = self
                .registered_events
                .peek()
                .map(|event| {
                    if event.force || self.should_trigger_event(&event, player) {
                        Some((event.name.clone(), None))
                    } else { None }
                })
                .unwrap_or(None);
        }
        let Some((event_name, segment_name)) = current_event_and_segment else {
            return Ok(());
        };

        // 获取当前事件数据
        let Some(event) = self
            .registered_events
            .iter()
            .find(|e| e.name == *event_name)
        else {
            return Ok(());
        };
        
        able_to_stuck = event.stuck_moving;

        // 获取段落: 如无指定，则发生第一个段落。
        let Some(segment) = segment_name
            .as_ref()
            .and_then(|seg_name| event.segments.iter().find(|seg| seg.name.eq(seg_name)))
            .or(event.segments.first())
        else {
            return Ok(());
        };
        frontend.cache.display_text(&segment.text);

        if segment.options.is_empty() {
            // 无选项，结束事件
            *current_event_and_segment = None;
            return Ok(());
        }

        // 显示选项并获取选择
        let options: Vec<String> = segment.options.iter().map(|opt| opt.text.clone()).collect();
        let selected_option = loop {
            let choice = frontend.display_options(&options)?;

            let Some(selected_option) = segment.options.get(choice) else {
                continue;
            };

            // 检查选项条件
            let Some(ref conditions) = selected_option.condition else {
                break selected_option;
            };
            let mut conditions_met = true;
            for condition in conditions {
                if !condition.is_met(time_system, map_system, player) {
                    conditions_met = false;
                    break;
                }
            }
            if !conditions_met {
                frontend.cache.display_error("选项条件不满足！");
                continue;
            }
            break selected_option;
        };

        // 应用属性修改
        if let Some(ref modifications) = selected_option.modifications {
            for (attr, value) in modifications {
                player.modify_attribute(attr, *value);
            }
        }

        // 跳转到指定段落
        if let Some(ref jump_to) = selected_option.jump_to {
            if let Some(next_segment) = event.segments.iter().find(|s| s.name == *jump_to) {
                // frontend.display_text(&next_segment.text);
                // 可以递归或循环处理后续段落
                *segment_name = Some(next_segment.name.clone());
            }
        } else {
            *segment_name = None
        }

        if let Some(trigger) = selected_option.trigger.clone() {
            for tr in trigger {
                player.trigger.insert(tr);
            }
        }

        // 结束当前事件
        if current_event_and_segment
            .as_ref()
            .is_some_and(|cur| cur.1.is_none())
            { *current_event_and_segment = None; }

        player.stuck_in_event = current_event_and_segment.is_some() && able_to_stuck;
        Ok(())
    }

    fn should_trigger_event(&self, _event: &EventData, _player: &Player) -> bool {
        // 可以在这里添加更多的事件触发逻辑
        true
    }
}
