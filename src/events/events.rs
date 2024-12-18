use crate::frontend::Frontend;
use crate::game;
use crate::player::Player;
use crate::systems::asset_system::AssetSystem;
use crate::systems::map_system::MapSystem;
use crate::systems::time_system::TimeSystem;
use serde::Deserialize;

use super::conditions::Condition;
use super::modifier::Modifier;
use super::triggers::Trigger;
use anyhow::Result;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct EventOption {
    pub text: String,                  // 描述
    pub condition: Option<Condition>,  // 选项的条件——是不是下面那个hide or not得放在这里？

    pub jump_to_event: Option<String>, // 选项下一个event。总是结束当前事件
    pub jump_to: Option<String>,       // 选项下一个segment。如果为空，结束事件
    pub trigger: Option<Vec<Trigger>>, // 选项触发的触发器。好像没什么用，不确定、再看看。
    pub avatar_set: Option<AvatarSet>, // 更换头像……因为Modification不该接触到前端，所以就放在这里了。我们应当认为，所有的前端工作都在 Event 层完成

    #[serde(default)]
    pub modifier: Modifier,             // 默认为不修改

    // #[serde(default)]
    // pub modifications: Option<HashMap<String, i32>>, // 属性修正
    // #[serde(default)]
    // pub item_new: Option<HashMap<String,(toml::Value,usize)>>,
    // pub item_delete: Option<HashMap<String,(Option<toml::Value>,usize)>>, // 属性修正
    
}

#[derive(Debug, Deserialize, Clone)]
pub enum AvatarSet {
    Main(String),
    Deco(String),
    MainKeepingDeco(String),
}

#[derive(Debug, Deserialize, Clone)]
pub struct EventSegment {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub silent: bool,
    #[serde(default)]
    pub options: Vec<EventOption>,
    #[serde(default)]
    pub hide_disabled_options: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EventData {
    pub name: String,
    #[serde(default)]
    pub priority: u32,
    pub force: bool,
    #[serde(default)]
    pub condition: Condition, // 修改为 Condition
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
        mut current_event_and_segment: Option<(String, Option<String>)>,
        player: &mut Player,
        time_system: &mut TimeSystem,
        map_system: &mut MapSystem,
        asset_system: &AssetSystem,
        frontend: &mut Frontend,
    ) -> Result<Option<(String, Option<String>)>, game::GameErr> {
        player.stuck_in_event = false;
        let able_to_stuck;
        // // 从优先级队列中取出优先级最高的事件
        // if current_event_and_segment.is_none() {
        //     current_event_and_segment = self
        //         .registered_events
        //         .pop()
        //         .map(|event| {
        //             if event.force || self.should_trigger_event(&event, player) {
        //                 Some((event.name.clone(), None))
        //             } else { None }
        //         })
        //         .unwrap_or(None);
        // }

        let Some((event_name, segment_name)) = &mut current_event_and_segment else { return Ok(None);};
        println!("{event_name}");
        // 获取当前事件数据
        let Some(event) = self.events.get(event_name) else { return Ok(None);};
        
        able_to_stuck = event.stuck_moving;

        // 获取段落: 如无指定，则发生第一个段落。
        let Some(segment) = segment_name
            .as_ref()
            .and_then(|seg_name| event.segments.iter().find(|seg| seg.name.eq(seg_name)))
            .or(event.segments.first())
        else { return Ok(None);};
        frontend.cache.display_text(&segment.text);

        if segment.options.is_empty() { return Ok(None); }

        // 选项与判定

        let options: Vec<(String,bool)> = segment
            .options.iter().map(|opt| (
                opt.text.clone(),             // 文本
                !opt.condition.as_ref() // 与“没有条件或条件成立”
                    .is_some_and(|c| !c.is_met(time_system, map_system, player))
            )).collect();

        let selected_option = if segment.silent {
            // 如果为无声事件，则自动选择，然后进入下一阶段。有意义吗？我不知道，就这么放着吧。如果无声事件寄了，直接err吧抬走不送
            options.iter().enumerate().filter(|p| p.1.1 )
                .next().map(|(i,_)|&segment.options[i]).unwrap()
        } else { 
            // 前端保证如此；相信前端。
            &segment.options[{
                frontend.display_options(&options,segment.hide_disabled_options)?
            }]
        };

        // 应用属性修改
        selected_option.modifier.modify(time_system, map_system, player);
        // if let Some(ref modifications) = selected_option.modifications {
        //     for (attr, value) in modifications {
        //         // player.modify_attribute(attr, *value);
        //         todo!()
        //     }
        // }

        // if let Some(ref item_new) = selected_option.item_new {
        //     for (str,(val,num)) in item_new {
        //         if !player.items.contains_key(str) {
        //             player.items.insert(str.clone(), (val.clone(),*num));
        //         } else {
        //             let num = num+player.items[str].1;
        //             player.items.insert(str.to_string(), (val.clone(),num));
        //         }
        //     }
        // }

        // if let Some(ref item_delete) = selected_option.item_delete {
        //     for (str,val) in item_delete { 
        //         if let (Some(val),_) = val {
        //             if player.items[str].0 != *val { continue; }
        //         }
        //         if !player.items.contains_key(str) { continue; }
        //         if player.items[str].1 <= val.1 { player.items.remove(str); }
        //         else { player.items.get_mut(str).unwrap().1 -= val.1; }
        //     }
        // }

        if let Some(ref avatar_set) = selected_option.avatar_set {
            match avatar_set {
                AvatarSet::Main(str) => frontend.cache.change_avatar(asset_system.avatar[str].clone()),
                AvatarSet::Deco(str) => frontend.cache.add_avatar_deco(asset_system.avatar_deco[str].clone()),
                AvatarSet::MainKeepingDeco(str) => frontend.cache.change_avatar_keeping_deco(asset_system.avatar[str].clone()),
            }
        }

        match (&selected_option.jump_to_event,&selected_option.jump_to) {
            (None,None) => {current_event_and_segment = None;}
            (Some(evt),seg) => { *event_name = evt.clone(); *segment_name = seg.clone();}
            (None,Some(jump_to)) => {*segment_name = Some(jump_to.clone());}
        }

        // 跳转到指定段落
        

        if let Some(trigger) = selected_option.trigger.clone() {
            for tr in trigger {
                player.trigger.insert(tr);
            }
        }

        player.stuck_in_event = current_event_and_segment.is_some() && able_to_stuck;
        Ok(current_event_and_segment)
    }

    // fn should_trigger_event(&self, _event: &EventData, _player: &Player) -> bool {
    //     // 可以在这里添加更多的事件触发逻辑
    //     true
    // }
}
