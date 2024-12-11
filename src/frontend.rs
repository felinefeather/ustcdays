use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, Sender},
};

use crate::{
    game::{GameDataSource, GameErr},
    player::Attribute,
};

// frontend.rs
#[derive(Clone, Default, Debug)]
pub struct ToFrontend {
    pub main_area: String,
    pub option_area: Vec<String>,
    pub player_status: Vec<String>,

    pub player_attribute: Vec<(String, i32, i32)>,
    pub debug: DebugToFrontend,
}

pub struct Frontend {
    pub receiver: Receiver<FromFrontend>,
    pub sender: Sender<ToFrontend>,
    pub cache: ToFrontend,
}

#[derive(Clone, Default, Debug)]
pub struct DebugToFrontend;

#[derive(Clone, Default, Debug)]
pub enum FromFrontend {
    Choice(usize),
    Debug(DebugFromFrontend),
    #[default]
    None,
}

impl FromFrontend {
    pub fn into_choice(self) -> Result<usize, DebugFromFrontend> {
        if let FromFrontend::Choice(u) = self {
            Ok(u)
        } else if let FromFrontend::Debug(dbg) = self {
            Err(dbg)
        } else {
            Err(DebugFromFrontend::default())
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct DebugFromFrontend {
    pub sign: DebugSign,
}

#[derive(Clone, Default, Debug)]
pub enum DebugSign {
    ReloadData(GameDataSource),
    SetAttribute(String, i32),
    #[default]
    None,
}

impl Frontend {
    pub fn display_options(&mut self, options: &[String]) -> Result<usize, GameErr> {
        self.cache.display_options(options);
        self.sender.send(self.cache.clone_and_clear())?;
        Ok(self.receiver.recv()?.into_choice()?)
    }
}

impl ToFrontend {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
    /// 显示一段文本
    pub fn display_text(&mut self, text: &str) {
        self.main_area.push_str(text);
    }

    /// 显示选项并获取玩家的选择
    /// 返回玩家选择的选项索引
    /// Blocking => ?
    pub fn display_options(&mut self, options: &[String]) {
        options
            .iter()
            .for_each(|opt| self.option_area.push(opt.clone()));
    }

    /// 显示玩家属性的过高或过低描述

    pub fn display_player_status(&mut self, descriptions: &[String]) {
        descriptions
            .iter()
            .for_each(|des| self.player_status.push(des.clone()));
    }

    pub fn display_player_attributes(
        &mut self,
        attributes: &HashMap<String, i32>,
        attr_def: &HashMap<String, Attribute>,
    ) {
        for (name, attr) in attr_def {
            if attr.invisible { continue; }
            self.player_attribute
                .push((name.clone(), attributes[name], attr.max));
        }
    }

    /// 显示错误信息
    pub fn display_error(&mut self, message: &str) {
        println!("ERROR: {message}");
    }

    pub fn clone_and_clear(&mut self) -> Self {
        let ret = self.clone();
        *self = Self::default();
        ret
    }
}
