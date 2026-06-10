use std::{cell::RefCell, collections::HashMap, error::Error, ops::Range, path::PathBuf};

use eframe::egui::{DragValue, TextEdit, Ui};
use ree_lib::{assets::bundle::Bundle, context::EngineContext, language::Language, rsz::RszMap};
use ree_save_core::{edit::{EditContext, Editable, EditorConfig}, game_context::GameData, save::{SaveFile, SaveOptions, game::Game}};
use uuid::Uuid;

use crate::{config::Config, steam::{self, Steam}};

pub struct Tab {
    pub tab: TabType,
    pub id: Uuid,
    pub name: Option<String>,
}

pub enum TabType {
    SaveFile(SaveFileView)
}

impl From<SaveFileView> for Tab {
    fn from(value: SaveFileView) -> Self {
        Self {
            tab: TabType::SaveFile(value),
            id: Uuid::new_v4(),
            name: None
        }
    }
}

pub struct SaveFileView {
    pub path: String,
    output: Option<String>,
    save_file: Option<SaveFile>,
    game: Game,
    steam: Steam,
    last_error: Option<Box<dyn Error>>,

    // Save Options
    brute_force: bool,
    brute_force_range: Range<u64>,
    curve_index: Option<usize>,
    dump: bool,
}

impl SaveFileView {
    pub fn new(config: &Config) -> Self {
        Self {
            path: "".to_string(),
            output: config.output_dir.clone(),
            save_file: None,
            game: config.game,
            steam: Steam::new(config.id, config.steam_path.clone()),
            last_error: None,
            brute_force: false,
            brute_force_range: (0x0110000100000000u64..0x01100001ffffffffu64),
            curve_index: None,
            dump: false,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, config: &Config, game_contexts: &HashMap<Game, GameData>) {
        let game_context = game_contexts.get(&self.game);

        ui.horizontal(|ui| {
            ui.add(TextEdit::singleline(&mut self.path).clip_text(false));

            if ui.button("Save").clicked() {

            }

            if ui.button("Load").clicked() {
                self.load_save(game_context);
            }
        });

        ui.horizontal(|ui| {
            self.steam.edit_steam_id(ui);
            if let Some(path) = self.steam.found_file(ui, self.game) {
                self.path = path;
                self.load_save(game_context);
            };
            self.steam.edit_steam_path(ui);
        });

        ui.horizontal(|ui| {
            if let Some(val) = self.curve_index.as_mut() {
                ui.label("Citrus Curve Index");
                ui.add(DragValue::new(val).speed(1.0));
            }
        });
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.brute_force, "Brute Force SteamID");
            ui.checkbox(&mut self.dump, "Dump Bytes");
        });

        if self.brute_force {
            ui.horizontal(|ui| {
                ui.label("Base:");
                ui.add(DragValue::new(&mut self.brute_force_range.start).speed(1.0).hexadecimal(8, true, false));
                ui.label("Count:");
                ui.add(DragValue::new(&mut self.brute_force_range.end).speed(1.0).hexadecimal(8, true, false));
            });
        }

        if let Some(last_error) = &self.last_error {
            ui.label(format!("Error: {}", last_error));
        }

        if let Some(save_file) = &mut self.save_file {
            let rsz_map = RszMap::default();
            let assets = Bundle::default();
            //let remaps = HashMap::default();
            let engine = EngineContext::new(config.language, &rsz_map, &assets);
            let path = Vec::with_capacity(50);
            let ctx = EditContext {
                engine_context: &engine,
                path: RefCell::new(path),
                config: &config.editor
            };
            save_file.ui(ui, &ctx);
        }
    }

    fn load_save(&mut self, game_context: Option<&GameData>) {
        let expanded = shellexpand::full(&self.path)
            .unwrap_or(std::borrow::Cow::Borrowed(&self.path));

        let path = PathBuf::from(expanded.as_ref());
        if path.exists() {
            match std::fs::read(&path) {
                Ok(data) => {
                    let mut options = SaveOptions::new(self.game);

                    if let Some(id) = self.steam.steam_id {
                        options = options.id(id);
                    }
                    if self.brute_force {
                        options = options.brute_force(self.brute_force_range.start, self.brute_force_range.end);
                    }
                    if let Some(curve_index) = self.curve_index {
                        options = options.curve_index(curve_index);
                    }
                    if self.dump {
                        options = options.debug_dump();
                    }

                    let save_file = SaveFile::read_save(data, &mut options);
                    match save_file {
                        Ok(save_file) => self.save_file = Some(save_file),
                        Err(e) => self.last_error = Some(Box::new(e)),
                    }

                    if self.brute_force {
                        self.steam.set_id(options.id);
                    }
                    self.curve_index = options.curve_index;
                }
                Err(e) => {
                    self.last_error = Some(Box::new(e));
                },
            }

        }
    }
}
