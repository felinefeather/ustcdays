use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, Sender}, vec,
};

use crate::{
    game::{DataSource, GameData, GameErr}, player::Attribute, frontend::assets::ImageData
};

use super::assets::Assets;

// frontend.rs
#[derive(Clone, Default, Debug)]
pub struct ToFrontend {
    pub main_area: Option<String>,
    pub option_area: Option<Vec<(String,bool)>>,
    pub option_display_disabled: Option<bool>,
    pub player_status: Option<Vec<String>>,

    pub player_attribute: Option<Vec<(String, i32, i32)>>,
    pub avatar_image: (Option<ImageData>,Option<Vec<ImageData>>),
    pub debug: Option<DebugToFrontend>,
}

impl ToFrontend {
    pub fn merge(&mut self, target: ToFrontend) {
        if let Some(main_area) = target.main_area { self.main_area = Some(main_area); }
        if let Some(option_area) = target.option_area { self.option_area = Some(option_area); }
        if let Some(option_display_disabled) = target.option_display_disabled { self.option_display_disabled = Some(option_display_disabled); }
        if let Some(player_status) = target.player_status { self.player_status = Some(player_status); }
        if let Some(player_attribute) = target.player_attribute { 
            self.player_attribute = Some(player_attribute); 
            println!("attr set");
        }
        if let Some(avatar_image) = target.avatar_image.0 { self.avatar_image.0 = Some(avatar_image); }
        if let Some(avatar_image) = target.avatar_image.1 { self.avatar_image.1 = Some(avatar_image); }
        if let Some(debug) = target.debug { self.debug = Some(debug); }
    }
}

pub struct Frontend {
    pub receiver: Receiver<FromFrontend>,
    pub sender: Sender<ToFrontend>,
    pub cache: ToFrontend,
    pub assets: Assets
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
    ReloadData(DataSource<GameData>),
    SetAttribute(String, i32),
    #[default]
    None,
}

impl Frontend {
    pub fn display_options(&mut self, options: &[(String,bool)], display_disabled: bool) -> Result<usize, GameErr> {
        self.cache.display_options(options,display_disabled);
        self.sender.send(self.cache.clone_and_clear())?;
        Ok(self.receiver.recv()?.into_choice()?)
    }

    pub fn display_all_options(&mut self, options: &[String]) -> Result<usize, GameErr> {
        self.cache.display_options(
            &options.iter().map(|s|(s.clone(),false)).collect::<Vec<_>>(),false);
        self.sender.send(self.cache.clone_and_clear())?;
        Ok(self.receiver.recv()?.into_choice()?)
    }

    pub fn change_avatar(&mut self, avatar: &String) {
        self.cache.change_avatar(
            self.assets.avatar[avatar].clone()
        );
    }

    pub fn change_avatar_keeping_deco(&mut self, avatar: &String) {
        self.cache.change_avatar_keeping_deco(
            self.assets.avatar[avatar].clone()
        );
    }

    pub fn add_avatar_deco(&mut self, deco: &String) {
        self.cache.add_avatar_deco(
            self.assets.avatar_deco[deco].clone()
        );
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
        self.main_area.get_or_insert(String::new()).push_str(text);
    }

    /// 显示选项并获取玩家的选择
    /// 返回玩家选择的选项索引
    /// Blocking => ?
    pub fn display_options(&mut self, options: &[(String,bool)], display_disabled: bool) {
        options
            .iter()
            .for_each(|opt| self.option_area.get_or_insert(vec![]).push(opt.clone()));
        self.option_display_disabled = Some(display_disabled);
    }

    /// 显示玩家属性的过高或过低描述

    pub fn display_player_status(&mut self, descriptions: &[String]) {
        descriptions
            .iter()
            .for_each(|des| self.player_status.get_or_insert(vec![]).push(des.clone()));
    }

    pub fn display_player_attributes(
        &mut self,
        attributes: &Vec<(String, i32)>,
        attr_def: &HashMap<String, Attribute>,
    ) {
        for (name, val) in attributes {
            let attr = &attr_def[name];
            if attr.invisible { continue; }
            self.player_attribute.get_or_insert(vec![])
                .push((name.clone(), *val, attr.max));
        }
    }

    /// 显示错误信息
    pub fn display_error(&mut self, message: &str) {
        println!("ERROR: {message}");
    }

    pub fn change_avatar(&mut self, avatar: ImageData) {
        self.avatar_image = (Some(avatar),Some(vec![]))
    }

    pub fn change_avatar_keeping_deco(&mut self, avatar: ImageData) {
        self.avatar_image.0 = Some(avatar);
    }

    pub fn add_avatar_deco(&mut self, deco: ImageData) {
        self.avatar_image.1.get_or_insert(vec![]).push(deco);
    }

    pub fn clone_and_clear(&mut self) -> Self {
        let ret = self.clone();
        *self = Self::default();
        ret
    }
}
