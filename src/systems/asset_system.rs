use std::{collections::HashMap, path::{Path, PathBuf}};

use egui::{ImageSize, Pos2};
use serde::Deserialize;

#[derive(Default,Deserialize,Clone,Debug)]
pub struct AssetSystem {
    pub avatar: HashMap<Vec<String>,ImageData>,
    pub avatar_deco: HashMap<Vec<String>,ImageData>,
}

#[derive(Default, Deserialize, Clone,Debug)]
pub struct ImageData {
    pub size: Option<(f32,f32)>,
    pub position: (f32,f32),
    pub path: String,
}