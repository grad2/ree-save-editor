use std::{collections::HashMap, path::{Path, PathBuf}};
use ree_lib::{assets::bundle::Bundle, enums::{EnumMap, load_enum_map}, rsz::RszMap};
use serde::Deserialize;

use crate::{save::{game::Game, remap::Remap}};

#[derive(Deserialize, Debug, Clone)]
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
    pub rsz: RszMap,
    pub enums: EnumMap,
    pub remaps: HashMap<String, Remap>,
    pub bundle: Bundle
}

impl TryFrom<&GamePaths> for GameData {
    type Error = anyhow::Error;
    fn try_from(value: &GamePaths) -> Result<Self, Self::Error> {
        let rsz = if let Some(path) = &value.rsz {
            let data = std::fs::read(path)?;
            serde_json::from_slice(&data)?
        } else {
            log::info!("Default rszmap");
            RszMap::default()
        };

        let enums = if let Some(ref path) = value.enums {
            load_enum_map(&PathBuf::from(&path))?
        } else {
            log::info!("Default enums loaded");
            EnumMap::default()
        };

        let remaps: HashMap<String, Remap> = if let Some(path) = &value.remaps {
            let data = std::fs::read(path)?;
            serde_json::from_slice(&data)?
        } else {
            log::info!("Default remap loaded");
            HashMap::default()
        };

        let bundle = if let Some(ref _path) = value.enums {
            //let data = std::fs::read(path)?;
            Bundle::default()
        } else {
            Bundle::default()
        };

        Ok(Self {
            rsz,
            enums,
            remaps,
            bundle,
        })
    }
}
