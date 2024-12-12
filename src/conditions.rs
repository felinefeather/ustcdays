use crate::time_system::TimeSystem;
use crate::map_system::MapSystem;
use crate::player::Player;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize,Clone)]
pub struct TimeCondition {
    pub start: String,          // "HH:MM"
    pub end: String,            // "HH:MM"
    pub days: Vec<String>,      // ["Monday", "Tuesday", ...]
    #[serde(default)]
    pub times: Option<Vec<String>>, // ["HH:MM", ...]
}

#[derive(Debug, Deserialize,Clone)]
pub struct LocationCondition {
    pub locations: Vec<String>,
}

#[derive(Debug, Deserialize,Clone)]
pub struct PlayerAttributeCondition {
    pub attributes: HashMap<String, AttributeCheck>,
}

#[derive(Debug, Deserialize,Clone)]
pub struct AttributeCheck {
    pub greater_than: Option<i32>,
    pub less_than: Option<i32>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Condition {
    Time(TimeCondition),
    Location(LocationCondition),
    PlayerAttribute(PlayerAttributeCondition),
    // 可以扩展更多条件类型
}

impl Condition {
    pub fn is_met(&self, time_system: &TimeSystem, map_system: &MapSystem, player: &Player) -> bool {
        match self {
            Condition::Time(cond) => time_system.check_condition(cond),
            Condition::Location(cond) => cond.locations.contains(&map_system.get_current_location().to_string()),
            Condition::PlayerAttribute(cond) => {
                for (attr, check) in &cond.attributes {
                    if let Some(value) = player.attributes.get(attr) {
                        if let Some(gt) = check.greater_than {
                            if *value <= gt {
                                return false;
                            }
                        }
                        if let Some(lt) = check.less_than {
                            if *value >= lt {
                                return false;
                            }
                        }
                    } else {
                        return false;
                    }
                }
                true
            },
        }
    }
}