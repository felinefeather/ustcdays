use std::collections::HashMap;

use serde::Deserialize;

#[derive(Default,Deserialize,Clone,Debug)]
pub struct AssetSystem {
    pub avatar: HashMap<String,ImageData>,
    #[serde(default)]
    pub avatar_deco: HashMap<String,ImageData>,
}

#[derive(Default, Deserialize, Clone,Debug)]
pub struct ImageData {
    pub size: Option<(f32,f32)>,
    #[serde(default)]
    pub position: (f32,f32),
    pub path: String,
}