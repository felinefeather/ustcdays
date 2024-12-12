use anyhow::Result;
use crate::{
    events::{EventData, EventSystem}, frontend::Frontend, map_system::{Map, MapSystem}, player::{Attribute, Player}, time_system::TimeSystem, triggers::TriggerSystem
};
use serde::Deserialize;

#[derive(Deserialize)]
struct GameData {
    maps: Vec<Map>,
    events: Vec<EventData>,
    player: Vec<Attribute>, // 修改为 Vec<Attribute>
}

pub struct Game<F: Frontend> {
    time_system: TimeSystem,
    map_system: MapSystem,
    event_system: EventSystem,
    player: Player,
    trigger_system: TriggerSystem,
    current_event_and_segment: Option<(String,Option<String>)>,
    
    data: GameData,
    frontend: F,
}

impl<F: Frontend> Game<F> {
    pub fn new(frontend: F) -> Result<Self> {
        let data_str = std::fs::read_to_string("C:\\Users\\felin\\ustcdays\\src\\data\\game_data.toml")?;
        let data: GameData = toml::from_str(&data_str)?;

        Ok(Game {
            time_system: TimeSystem::new(),
            map_system: MapSystem::new(&data.maps),
            event_system: EventSystem::new(&data.events),
            player: Player::new(&data.player),
            trigger_system: TriggerSystem::new(),
            current_event_and_segment: None,
            data,
            frontend,
        })
    }

    pub fn run(&mut self) -> Result<()> {
         loop {
            // 更新时间系统
            self.time_system.update();
            self.player.game_time = self.time_system.current_time;
            self.player.game_map = self.map_system.get_current_location().to_string();

            // 触发事件
            self.trigger_system.check(&self.time_system, &self.map_system, &self.player, &mut self.event_system)?;

            // 显示玩家状态
            let status = self.player.get_over_under_descriptions();
            self.frontend.display_player_status(&status);

            // 处理事件
            self.event_system.process_events(&mut self.current_event_and_segment, &mut self.player, &self.time_system, &self.map_system, &self.frontend)?;

            // 示例：玩家选择是否移动地图
            if !self.player.stuck_in_event {
                self.frontend.display_text("你想要移动到其他地点吗？");
                let choice = self.frontend.display_options(&vec!["是".to_string(), "否".to_string()]);
                if choice == 0 {
                    // 显示可移动的地图
                    let current_map = self.map_system.get_current_location();
                    let connections = self.map_system.maps.get(current_map).unwrap().connections.clone();
                    let options: Vec<String> = connections.iter().map(|c| format!("{} (需要 {} 分钟)", c.to, c.time)).collect();
                    let map_choice = self.frontend.display_options(&options);
                    if map_choice < connections.len() {
                        let destination = &connections[map_choice].to;
                        match self.map_system.travel(destination, &mut self.time_system) {
                            Ok(_) => self.frontend.display_text(&format!("你已移动到 {}", destination)),
                            Err(e) => self.frontend.display_error(&e),
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