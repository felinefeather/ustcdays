use crate::player::{Player, PlayerAttribute, PlayerItem};
use crate::systems::Systems;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct TimeCondition {
    pub start: String,     // "HH:MM"
    pub end: String,       // "HH:MM"
    pub days: Vec<String>, // ["Monday", "Tuesday", ...]
    #[serde(default)]
    pub times: Option<Vec<String>>, // ["HH:MM", ...]
}

#[derive(Debug, Deserialize, Clone)]
pub struct LocationCondition {
    pub locations: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PlayerAttributeCondition {
    pub attributes: HashMap<String, AttributeCheck>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PlayerItemContition {
    pub items: HashMap<String, ItemCheck>
}


#[derive(Debug, Deserialize, Clone)]
pub struct AttributeCheck {
    pub greater_than: Option<i32>,
    pub less_than: Option<i32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ItemCheck {
    pub expect_existence: Option<bool>,
    pub expect_tags: Option<String>,
    pub more_than: Option<usize>,
    pub less_than: Option<usize>
    // pub tag_check: HashMap<String,ValueCheck>,
}

// #[derive(Debug, Deserialize, Clone)]
// pub enum ValueCheck {
//     Equals(toml::Value),
// }

#[derive(Debug, Deserialize, Clone)]
pub struct ConditionGroup {
    pub conds: Vec<Condition>
}

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(tag = "type")]
pub enum Condition {
    Time(TimeCondition),
    Location(LocationCondition),
    PlayerAttribute(PlayerAttributeCondition),
    PlayerItem(PlayerItemContition),

    RandomCondition(f64),
    // 可以扩展更多条件类型

    // 逻辑条件
    And(ConditionGroup),
    Or(ConditionGroup),
    Xor(ConditionGroup),

    False,
    #[default]
    True,
}

pub type PlayerCond<'a> = (&'a PlayerItem, &'a PlayerAttribute);

impl<'a> Into<PlayerCond<'a>> for &'a Player {
    fn into(self) -> PlayerCond<'a> {
        (&self.items,&self.attributes)
    }
}

impl<'a> Into<PlayerCond<'a>> for &'a mut Player {
    fn into(self) -> PlayerCond<'a> {
        (&self.items,&self.attributes)
    }
}

impl Condition {
    pub fn is_met(
        &self,
        systems: &Systems,
        player: &Player,
    ) -> bool {
        match self {
            Condition::Time(cond) => systems.time.check_condition(player,cond),
            Condition::Location(cond) => cond
                .locations
                .contains(&player.game_map),
            Condition::PlayerAttribute(cond) => {
                for (attr, check) in &cond.attributes {
                    let value = *player.attributes.get(attr).unwrap();
                    if check.greater_than.is_some_and(|v|v>=value)
                        || check.less_than.is_some_and(|v|v<=value) { 
                            return false; 
                    }
                }
                true
            }
            Condition::PlayerItem(cond) => {
                for (item,check) in &cond.items {
                    let item = player.items.get(item);
                    if let Some(exsists) = check.expect_existence {
                        if item.is_some() != exsists { return false; }
                    }
                    let Some((item,num)) = item else { return false; };
                    if check.more_than.is_some_and(|v|v>=*num)
                        || check.less_than.is_some_and(|v|v<=*num) { 
                            return false; 
                    }
                    if !item.is_table() {
                        if check.expect_tags.is_none() { continue; }
                        else { return false; }
                    }
                    let toml::Value::Table(item) = item else { panic!("never") };
                    if check.expect_tags.is_some()
                        &&!item.contains_key(check.expect_tags.as_ref().unwrap()) {
                            return false;
                        }
                    
                }
                true
            },
            Condition::And(vec) => {
                vec.conds.iter().all(|cond| cond.is_met(systems,player))
            },
            Condition::Or(vec) => {
                vec.conds.iter().any(|cond| cond.is_met(systems,player))
            },
            Condition::Xor(vec) => {
                vec.conds.iter().fold(false, |fold,cond| fold^cond.is_met(systems,player))
            },
            Condition::RandomCondition(prop) => {
                rand::random::<f64>()  < *prop
            },
            Condition::False => false,
            Condition::True => true,
        }
    }
}
