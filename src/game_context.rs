use std::{collections::HashMap};
use ree_lib::{assets::bundle::Bundle, enums::EnumMap, rsz::RszMap};
use serde::Deserialize;

use crate::{save::{game::Game, remap::Remap}};

#[derive(Deserialize)]
pub struct GamePaths {
    pub rsz: Option<String>,
    pub enums: Option<String>,
    pub remaps: Option<String>,
    pub bundle: Option<String>,
}

pub type GameConfigs = HashMap<Game, GamePaths>;

pub fn load_game_configs(path: &str) -> anyhow::Result<GameConfigs> {
    let data = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&data)?)
}


pub struct GameData {
    rsz: RszMap,
    enums: EnumMap,
    remaps: HashMap<String, Remap>,
    bundle: Bundle
}

impl TryFrom<GamePaths> for GameData {
    type Error = anyhow::Error;
    fn try_from(value: GamePaths) -> Result<Self, Self::Error> {

        todo!()
    }
}
