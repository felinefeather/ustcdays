use std::{
    collections::HashMap, path::PathBuf, sync::mpsc::{Receiver, Sender}
};

use crate::{
    events::{
        events::{EventData, EventSystem},
        triggers::{Trigger, TriggerSystem},
    },
    frontend::{assets::Assets, DebugFromFrontend, FromFrontend, Frontend, ToFrontend},
    player::{Attribute, Player},
    systems::{
        map_system::{Map, MapSystem}, time_system::TimeSystem, Systems
    },
};
use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug, Default)]
pub struct GameData {
    pub maps: Vec<Map>,
    pub events: Vec<EventData>,
    pub player: Vec<Attribute>, // 修改为 Vec<Attribute>
    #[serde(default)]
    pub assets: Assets,
    #[serde(default)]
    pub trigger: Vec<HashMap<String,Trigger>>,
}

pub struct Game {
    systems: Systems,
    player: Player,

    pub frontend: Frontend,
}

#[derive(Clone, Default, Debug)]
pub enum DataSource<T> {
    Path(PathBuf),
    Raw(String),
    Inbuilt(Box<T>),
    #[default]
    None,
}

impl<T: serde::de::DeserializeOwned + Default> DataSource<T> {
    fn into_data(self) -> Result<T> {
        let str = match self {
            DataSource::Path(path_buf) => std::fs::read_to_string(path_buf)?,
            DataSource::Raw(str) => str,
            DataSource::Inbuilt(gamedata) => return Ok(*gamedata),
            DataSource::None => 
                return DataSource::Inbuilt(Box::new(T::default())).into_data(),
        };
        Ok(toml::from_str(&str)?)
    }
}

#[derive(Debug, Default)]
pub enum GameErr {
    DebugEscape(DebugFromFrontend),
    Error(anyhow::Error),
    #[default]
    Default,
}

impl<T: Into<anyhow::Error>> From<T> for GameErr {
    fn from(value: T) -> Self {
        Self::Error(value.into())
    }
}

impl From<DebugFromFrontend> for GameErr {
    fn from(value: DebugFromFrontend) -> Self {
        Self::DebugEscape(value)
    }
}

impl Game {
    pub fn new_with_player(
        source: DataSource<GameData>,
        player_source: DataSource<Player>,
        frontend: (Sender<ToFrontend>, Receiver<FromFrontend>),
    )-> Result<Self> {
        let mut ret = Self::new(source, frontend)?;
        ret.player = player_source.into_data()?;
        Ok(ret)
    }

    pub fn new(
        source: DataSource<GameData>,
        frontend: (Sender<ToFrontend>, Receiver<FromFrontend>),
    ) -> Result<Self> {
        let data = source.into_data()?;

        Ok(Game {
            systems: Systems { 
                time: TimeSystem::new(), 
                map: MapSystem::new(&data.maps),
                trigger: TriggerSystem::new(&data.trigger), 
                event: EventSystem::new(&data.events) 
            },

            player: Player::new(&data.player),

            frontend: Frontend {
                sender: frontend.0,
                cache: ToFrontend::new(),
                receiver: frontend.1,
                assets: data.assets
            },
        })
    }

    pub fn main_loop(&mut self) -> Result<(),GameErr> {
        loop {
            let Self { 
                systems, player, frontend 
            } = self;

            TriggerSystem::set_default(&mut player.trigger);

            if let Some(evt) = systems.trigger.pick_event(&player, systems) {
                player.cur_evt_seg = Some((evt.clone(),None));
            } player.trigger.clear();

            player.cur_evt_seg = systems.event.process_events(
                player, systems, frontend,
            )?;
        }
    }

    pub fn run(mut self) {
        loop {
            let Err(e) = self.main_loop() else { todo!("game over"); }; 
            match e {
                GameErr::DebugEscape(frontend_debug_input) => {
                    use crate::frontend::DebugSign::*;
                    match frontend_debug_input.sign {
                        ReloadData(source) => {
                            self = Self::new(
                                source,
                                (self.frontend.sender, self.frontend.receiver),
                            )
                            .unwrap_or_else(|error| {
                                panic!("{}", error);
                            });
                        }
                        SetAttribute(str, val) => {
                            if let Some(v) = self.player.attributes.get_mut(&str) {
                                *v = val;
                            }
                        },
                        None => (),
                    }
                }
                GameErr::Error(error) => {
                    if error.to_string().contains("SendError") {
                        println!("Fronted disconnected");
                        return;
                    }
                    dbg!(error);
                }
                GameErr::Default => (),
            }
        }
    }

    // pub fn game_loop(&mut self) -> Result<(), GameErr> {
    //     loop {
    //         // 检测 dbg 是否发生
    //         if let Ok(FromFrontend::Debug(dbg)) = self.frontend.receiver.try_recv() {
    //             return Err(dbg.into());
    //         }

    //         // 触发事件
    //         self.trigger_system.check(
    //             &self.player.trigger,
    //             &self.time_system,
    //             &self.map_system,
    //             &self.player,
    //             &mut self.event_system,
    //         )?; 
            
    //         self.player.trigger.clear();
    //         self.player.trigger.insert(Trigger::Always);

    //         // 处理事件
    //         self.event_system.process_events(
    //             &mut self.current_event_and_segment,
    //             &mut self.player,
    //             &self.time_system,
    //             &self.map_system,
    //             &self.asset_system,
    //             &mut self.frontend,
    //         )?;

    //         // 更新玩家状态
    //         self.player.game_time = self.time_system.current_time.to_string();
    //         self.player.game_map = self.map_system.get_current_location().to_string();
    //         let status = self.player.get_over_under_descriptions();
    //         self.frontend.cache.display_player_status(&status);
    //         self.frontend.cache
    //             .display_player_attributes(&self.player.attributes.val, &self.player.attribute_defs);

    //         // 示例：玩家选择是否移动地图\
    //         // 把这个用 event 重写！

    //         // if !self.player.stuck_in_event {
    //         //     self.frontend.cache.display_text("你想要移动到其他地点吗？");
    //         //     let choice = self
    //         //         .frontend
    //         //         .display_options(&vec![("是".to_string(),true), ("否".to_string(),true)],false)?;
    //         //     if choice == 0 {
    //         //         // 显示可移动的地图
    //         //         let current_map = self.map_system.get_current_location();
    //         //         let connections = self
    //         //             .map_system
    //         //             .maps
    //         //             .get(current_map)
    //         //             .unwrap()
    //         //             .connections
    //         //             .clone();

    //         //         let options: Vec<(String,bool)> = connections
    //         //             .iter()
    //         //             .map(|c| (
    //         //                 format!("{} (需要 {} 分钟)", c.optional_name.clone().unwrap_or(c.to.clone()),c.time ),
    //         //                 c.condition.is_none() || c.condition.as_ref().unwrap().is_met(&self.time_system, &self.map_system, &self.player)
    //         //             )).collect();

    //         //         let map_choice = self.frontend.display_options(&options,false)?;

    //         //         // 确认目的地，开始尝试移动
    //         //         let destination = &connections[map_choice].to;
    //         //         match self.map_system.travel(destination, &mut self.time_system) {
    //         //             Ok(_) => {
    //         //                 self.player
    //         //                     .trigger
    //         //                     .insert(Trigger::Reached(destination.clone()));
    //         //                 self.frontend
    //         //                     .cache
    //         //                     .display_text(
    //         //                         self.map_system.maps[destination].description.clone()
    //         //                             .unwrap_or("已抵达".into()).as_str());
    //         //             }
    //         //             Err(e) => self.frontend.cache.display_error(&e),
    //         //         }
                    
    //         //     }
    //         // }
    //     }
    // }
}
