use crate::{events::conditions::TimeCondition, player::Player};
use chrono::NaiveDateTime;

pub struct TimeSystem;

impl TimeSystem {
    pub fn new() -> Self { Self }
    pub fn init_time() -> NaiveDateTime {
        // 初始化当前时间，可以根据需要调整
        NaiveDateTime::parse_from_str("2024-01-02 07:00", "%Y-%m-%d %H:%M")
            .unwrap()
    }

    pub fn update(&self, _player: &mut Player) {
        // 更新时间逻辑，例如每回合增加一定时间
        // self.current_time = self.current_time + chrono::Duration::minutes(1);
    }

    pub fn get_current_time(player: &Player) -> NaiveDateTime {
        player.game_time
    }

    pub fn check_condition(&self, player: &Player, condition: &TimeCondition) -> bool {
        let current_time: NaiveDateTime = player.game_time;

        // 检查星期
        let day_str = chrono::Datelike::weekday(&current_time).to_string();
        let day_match = condition
            .days
            .iter()
            .any(|day| day.eq_ignore_ascii_case(&day_str));

        if !day_match {
            return false;
        }

        // 检查时间范围
        let start_time = NaiveDateTime::parse_from_str(
            &format!("{} {}", current_time.date(), condition.start),
            "%Y-%m-%d %H:%M",
        )
        .unwrap();
        let end_time = NaiveDateTime::parse_from_str(
            &format!("{} {}", current_time.date(), condition.end),
            "%Y-%m-%d %H:%M",
        )
        .unwrap();

        let in_range = current_time >= start_time && current_time <= end_time;

        // 检查具体时间点
        let times_match = if let Some(times) = &condition.times {
            times.iter().any(|t| {
                let specific_time = NaiveDateTime::parse_from_str(
                    &format!("{} {}", current_time.date(), t),
                    "%Y-%m-%d %H:%M",
                )
                .unwrap();
                current_time == specific_time
            })
        } else {
            true
        };

        in_range && times_match
    }
}
