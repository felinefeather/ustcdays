use std::{
    collections::HashMap, path::PathBuf, str::FromStr, sync::mpsc::{Receiver, Sender}
};

use crate::{
    events::{
        events::{EventData, EventSystem},
        triggers::{Trigger, TriggerSystem},
    },
    frontend::{DebugFromFrontend, FromFrontend, Frontend, ToFrontend},
    player::{Attribute, Player},
    systems::{
        asset_system::AssetSystem, map_system::{Map, MapSystem}, time_system::TimeSystem
    },
};
use anyhow::Result;
use chrono::NaiveDateTime;
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug, Default)]
pub struct GameData {
    pub maps: Vec<Map>,
    pub events: Vec<EventData>,
    pub player: Vec<Attribute>, // 修改为 Vec<Attribute>
    #[serde(default)]
    pub assets: AssetSystem,
    #[serde(default)]
    pub trigger: Vec<HashMap<String,Trigger>>,
}

pub struct Game {
    time_system: TimeSystem,
    map_system: MapSystem,
    asset_system: AssetSystem,
    event_system: EventSystem,
    player: Player,
    trigger_system: TriggerSystem,
    current_event_and_segment: Option<(String, Option<String>)>,

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
        ret.time_system.current_time = NaiveDateTime::from_str(&ret.player.game_time)?;
        ret.map_system.current_location = ret.player.game_map.clone();
        Ok(ret)
    }

    pub fn new(
        source: DataSource<GameData>,
        frontend: (Sender<ToFrontend>, Receiver<FromFrontend>),
    ) -> Result<Self> {
        let data = source.into_data()?;

        Ok(Game {
            asset_system: data.assets,
            time_system: TimeSystem::new(),
            map_system: MapSystem::new(&data.maps),
            event_system: EventSystem::new(&data.events),
            player: Player::new(&data.player),
            trigger_system: TriggerSystem {
                registed_event: {
                    let ret = data.trigger.iter()
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
            },
            current_event_and_segment: None,

            frontend: Frontend {
                sender: frontend.0,
                cache: ToFrontend::new(),
                receiver: frontend.1,
            },
        })
    }

    pub fn main_loop(&mut self) {
        loop {
            let Self { 
                time_system,
                map_system, 
                asset_system, 
                event_system, 
                player, 
                trigger_system, 
                current_event_and_segment, 
                frontend 
            } = &mut self;
            trigger_system.add(player,time_system,map_system);
            event_system.try_insert(trigger_system.pick_event());
            event_system.run(player,asset_system,frontend);
        }
    }

    pub fn run(mut self) {
        loop {
            let Err(e) = self.game_loop() else { todo!("game over"); }; 
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
                                dbg!(error);
                                todo!("没做怎么处理err红豆私密马赛")
                            });
                        }
                        SetAttribute(str, val) => {
                            if let Some(v) = self.player.attributes.get_mut(&str) {*v = val;}
                        },
                        None => (),
                    }
                }
                GameErr::Error(error) => {
                    dbg!(error);
                    todo!("没做怎么处理err红豆私密马赛")
                }
                GameErr::Default => (),
            }
        }
    }

    pub fn game_loop(&mut self) -> Result<(), GameErr> {
        loop {
            // 检测 dbg 是否发生
            if let Ok(FromFrontend::Debug(dbg)) = self.frontend.receiver.try_recv() {
                return Err(dbg.into());
            }

            // 触发事件
            self.trigger_system.check(
                &self.player.trigger,
                &self.time_system,
                &self.map_system,
                &self.player,
                &mut self.event_system,
            )?; 
            
            self.player.trigger.clear();
            self.player.trigger.insert(Trigger::Always);

            // 处理事件
            self.event_system.process_events(
                &mut self.current_event_and_segment,
                &mut self.player,
                &self.time_system,
                &self.map_system,
                &self.asset_system,
                &mut self.frontend,
            )?;

            // 更新玩家状态
            self.player.game_time = self.time_system.current_time.to_string();
            self.player.game_map = self.map_system.get_current_location().to_string();
            let status = self.player.get_over_under_descriptions();
            self.frontend.cache.display_player_status(&status);
            self.frontend.cache
                .display_player_attributes(&self.player.attributes.val, &self.player.attribute_defs);

            // 示例：玩家选择是否移动地图\
            // 把这个用 event 重写！

            // if !self.player.stuck_in_event {
            //     self.frontend.cache.display_text("你想要移动到其他地点吗？");
            //     let choice = self
            //         .frontend
            //         .display_options(&vec![("是".to_string(),true), ("否".to_string(),true)],false)?;
            //     if choice == 0 {
            //         // 显示可移动的地图
            //         let current_map = self.map_system.get_current_location();
            //         let connections = self
            //             .map_system
            //             .maps
            //             .get(current_map)
            //             .unwrap()
            //             .connections
            //             .clone();

            //         let options: Vec<(String,bool)> = connections
            //             .iter()
            //             .map(|c| (
            //                 format!("{} (需要 {} 分钟)", c.optional_name.clone().unwrap_or(c.to.clone()),c.time ),
            //                 c.condition.is_none() || c.condition.as_ref().unwrap().is_met(&self.time_system, &self.map_system, &self.player)
            //             )).collect();

            //         let map_choice = self.frontend.display_options(&options,false)?;

            //         // 确认目的地，开始尝试移动
            //         let destination = &connections[map_choice].to;
            //         match self.map_system.travel(destination, &mut self.time_system) {
            //             Ok(_) => {
            //                 self.player
            //                     .trigger
            //                     .insert(Trigger::Reached(destination.clone()));
            //                 self.frontend
            //                     .cache
            //                     .display_text(
            //                         self.map_system.maps[destination].description.clone()
            //                             .unwrap_or("已抵达".into()).as_str());
            //             }
            //             Err(e) => self.frontend.cache.display_error(&e),
            //         }
                    
            //     }
            // }
        }
    }
}
