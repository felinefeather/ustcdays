use std::{
    collections::HashMap,
    path::PathBuf,
    sync::mpsc::{Receiver, Sender},
};

use crate::{
    events::{
        events::{EventData, EventSystem},
        triggers::{Trigger, TriggerSystem},
    },
    frontend::{DebugFromFrontend, FromFrontend, Frontend, ToFrontend},
    player::{Attribute, Player},
    systems::{
        map_system::{Map, MapSystem},
        time_system::TimeSystem,
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
    pub trigger: HashMap<Trigger, Vec<String>>,
}

pub struct Game {
    time_system: TimeSystem,
    map_system: MapSystem,
    event_system: EventSystem,
    player: Player,
    trigger_system: TriggerSystem,
    current_event_and_segment: Option<(String, Option<String>)>,

    pub frontend: Frontend,
}

#[derive(Clone, Default, Debug)]
pub enum GameDataSource {
    Path(PathBuf),
    Raw(String),
    Inbuilt(Box<GameData>),
    #[default]
    None,
}

impl GameDataSource {
    fn into_gamedata(self) -> Result<GameData> {
        let str = match self {
            GameDataSource::Path(path_buf) => std::fs::read_to_string(path_buf)?,
            GameDataSource::Raw(str) => str,
            GameDataSource::Inbuilt(gamedata) => return Ok(*gamedata),
            GameDataSource::None => 
                return GameDataSource::Inbuilt(Box::new(GameData::default())).into_gamedata(),
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
    pub fn new(
        source: GameDataSource,
        frontend: (Sender<ToFrontend>, Receiver<FromFrontend>),
    ) -> Result<Self> {
        let data = source.into_gamedata()?;

        Ok(Game {
            time_system: TimeSystem::new(),
            map_system: MapSystem::new(&data.maps),
            event_system: EventSystem::new(&data.events),
            player: Player::new(&data.player),
            trigger_system: TriggerSystem {
                registed_event: data.trigger.clone(),
            },
            current_event_and_segment: None,

            frontend: Frontend {
                sender: frontend.0,
                cache: ToFrontend::new(),
                receiver: frontend.1,
            },
        })
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

            // 更新时间系统
            self.time_system.update();

            self.player.game_time = self.time_system.current_time;
            self.player.game_map = self.map_system.get_current_location().to_string();

            // 触发事件
            self.trigger_system.check(
                &self.player.trigger,
                &self.time_system,
                &self.map_system,
                &self.player,
                &mut self.event_system,
            )?;

            // 显示玩家状态
            let status = self.player.get_over_under_descriptions();
            self.frontend.cache.display_player_status(&status);
            self.frontend.cache
                .display_player_attributes(&self.player.attributes, &self.player.attribute_defs);

            // 处理事件
            self.event_system.process_events(
                &mut self.current_event_and_segment,
                &mut self.player,
                &self.time_system,
                &self.map_system,
                &mut self.frontend,
            )?;

            // 示例：玩家选择是否移动地图
            if !self.player.stuck_in_event {
                self.frontend.cache.display_text("你想要移动到其他地点吗？");
                let choice = self
                    .frontend
                    .display_options(&vec!["是".to_string(), "否".to_string()])?;
                if choice == 0 {
                    // 显示可移动的地图
                    let current_map = self.map_system.get_current_location();
                    let connections = self
                        .map_system
                        .maps
                        .get(current_map)
                        .unwrap()
                        .connections
                        .clone();
                    let options: Vec<String> = connections
                        .iter()
                        .map(|c| format!("{} (需要 {} 分钟)", c.to, c.time))
                        .collect();
                    let map_choice = self.frontend.display_options(&options)?;
                    if map_choice < connections.len() {
                        let destination = &connections[map_choice].to;
                        match self.map_system.travel(destination, &mut self.time_system) {
                            Ok(_) => {
                                self.player
                                    .trigger
                                    .insert(Trigger::Reached(destination.clone()));
                                self.frontend
                                    .cache
                                    .display_text(&format!("你已移动到 {}", destination));
                            }
                            Err(e) => self.frontend.cache.display_error(&e),
                        }
                    }
                }
            }

            // 添加退出条件，例如玩家输入特定命令
            // 例如，每1000回合自动退出

            // 被我删了——这是我唯一写过的注释
        }
    }
}
