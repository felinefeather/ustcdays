use serde::Deserialize;

use crate::{player::Player, systems::Systems};

use super::conditions::Condition;

#[derive(Default,Deserialize,Clone,Debug)]
#[serde(untagged)]
pub enum Modifier {
    Attribute { attr: Identity, val: ValModifier },
    Item { item: String, modify: ItemModifier },
    Position { towards: String, #[serde(default)] check: bool },

    Group(Vec<Modifier>),
    Condition{group: Vec<Modifier>,cond: Option<Condition>},
    #[default]
    None
}

#[derive(Default,Deserialize,Clone,Debug)]
#[serde(untagged)]
pub enum Identity {
    Str(String),
    Index(usize),
    #[default]
    None
}

#[derive(Default,Deserialize,Clone,Debug)]
pub enum ValModifier {
    Add(i32),
    Mul(f32),
    Sqrt10, // 钱学森先生发明的计分法，再次呈现！
    #[default]
    None
}

impl ValModifier {
    pub fn apply(&self, val: &mut i32) {
        match self {
            Self::Add(add) => {* val += add; }
            Self::Mul(mul) => {*val = (*val as f32 * mul) as i32}
            Self::Sqrt10 => {*val = ((*val as f32).sqrt()*10.) as i32}
            Self::None => ()
        }
    }
}

#[derive(Default,Deserialize,Clone,Debug)]
#[serde(untagged)]
pub enum ItemModifier {
    Add { add: usize, val: Option<toml::Value> },
    Sub { sub: usize, val: Option<toml::Value> },
    ModifyVal { val: toml::Value},
    #[default]
    None
}

impl ItemModifier {
    pub fn apply(&self, (value,num): &mut (toml::Value,usize)) {
        match self {
            ItemModifier::Add { add, val } => {
                *num += add;
                if let Some(val) = val { *value = val.clone(); }
            },
            ItemModifier::Sub { sub, val } => {
                if val.is_none() || val.as_ref().unwrap().eq(&value) {
                    if *num > *sub { *num -= sub; } else { *num = 0 }
                }
            },
            ItemModifier::ModifyVal { val } => { *value = val.clone() },
            ItemModifier::None => (),
        }
    }
}

impl Modifier {
    pub fn modify(
        &self, 
        systems: &Systems,
        player: &mut Player
    ) -> anyhow::Result<()> {
        // let trigger = &mut player.trigger;
        match &self {
            Modifier::Attribute { attr, val } => {
                player.modify_attribute(attr, val); 
            },
            Modifier::Item { item, modify } => {
                let Some(val) = player.items.get_mut(item) else { return Ok(()); };
                modify.apply(val); if val.1 == 0 { player.items.remove(item); }
            },
            Modifier::Group(group) => {
                for modifier in group {
                    modifier.modify(systems, player)?;
                }
            },
            Modifier::Condition { group, cond } => {
                if cond.is_none() || cond.as_ref().unwrap().is_met(systems, player) {
                    for modifier in group {
                        modifier.modify(systems, player)?;
                    }
                }
            },
            Modifier::None => (),
            Modifier::Position { towards, check } => {
                if *check {
                    systems.map.travel(player, &towards, &systems.time)?;
                } else { player.game_map = towards.clone() }
            },
        };
        Ok(())
    }
}