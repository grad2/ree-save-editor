use std::{collections::{HashMap, HashSet}, sync::{Arc, RwLock, mpsc::{self, Receiver, Sender}}};

use eframe::egui::{Ui};
use egui_dock::tab_viewer::OnCloseResponse;

use ree_lib::language::Language;
use ree_save_core::{
    game_context::{GameConfigs, GameData, load_game_configs}, save::game::Game
};

use crate::{config::Config, tab::{self, TabType}};

pub struct Viewer<'a> {
    pub game_contexts: &'a Arc<RwLock<HashMap<Game, GameData>>>,
    pub config: &'a Config
}

impl<'a> egui_dock::TabViewer for Viewer<'a> {
    type Tab = tab::Tab;

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        ui.push_id(tab.id, |ui| {
            match &mut tab.tab {
                TabType::SaveFile(save_file) => {
                    let game_contexts = self.game_contexts.read().unwrap();
                    save_file.ui(ui, self.config, &game_contexts);
                    // TODO: possibly add a request queue for save files/ different tab types to ask
                    // the viewer to try and load different data in async
                },
            }
        });
    }

    fn title(&mut self, tab: &mut Self::Tab) -> eframe::egui::WidgetText {
        let title: String = match &tab.tab {
            TabType::SaveFile(s) => {
                use std::path::PathBuf;

                let file_str = match &s.file_picker.selected_file {
                    Some(path) => {
                        let count = path.iter().count();
                        let last_two: PathBuf = path.iter().skip(count.saturating_sub(2)).collect();
                        last_two.display().to_string()
                    }
                    None => "(empty)".to_string(),
                };

                format!("{:?}|{}", s.game, file_str).into()
            },
        };
        title.into()
    }

    fn context_menu(
        &mut self,
        ui: &mut Ui,
        tab: &mut Self::Tab,
        _surface: egui_dock::SurfaceIndex,
        _node: egui_dock::NodeIndex,
    ) {
        ui.label(self.title(tab));
    }

    fn on_close(&mut self, _tab: &mut Self::Tab) -> egui_dock::tab_viewer::OnCloseResponse {
        OnCloseResponse::Close
    }
}
