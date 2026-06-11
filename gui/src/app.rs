use std::{collections::HashMap, sync::{Arc, RwLock}};

use eframe::{
    self,
    egui::{Align, CentralPanel, Layout, MenuBar, TopBottomPanel},
};
use egui_dock::{DockArea, DockState};
use ree_lib::language::Language;
use ree_save_core::{game_context::{GameConfigs, GameData, load_game_configs}, save::game::Game};


use crate::{config::{Config, load_config, load_config_checked}, tab::{SaveFileView, Tab}, viewer::Viewer};

pub struct App {
    tree: DockState<Tab>,
    config: Config,
    config_path: String,
    game_configs: GameConfigs,
    game_contexts: Arc<RwLock<HashMap<Game, GameData>>>,
}

impl App {
    pub fn new(config_path: String, config: Config) -> Self {
        let dock_state = DockState::new(vec![]);
        let game_configs = load_game_configs("game_configs.json")
            .unwrap_or_default();
        Self {
            tree: dock_state,
            game_configs,
            config_path,
            game_contexts: Arc::new(RwLock::new(HashMap::new())),
            config
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("Menu Bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("REE Save Editor");

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.hyperlink_to("GitHub", "https://github.com/kvasszn/ree-save-editor");
                    ui.separator();
                });
            });

            MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Empty Save File").clicked() {
                        let file_view = SaveFileView::new(&self.config);
                        let surface = self.tree.main_surface_mut();
                        surface.push_to_focused_leaf(Tab::from(file_view));
                    }
                });

                // TODO: add a live update type thing that looks at file modification time from last
                // reloaded
                if ui.button("Reload Config").clicked() {
                    match load_config_checked(&self.config_path) {
                        Ok(config) => self.config = config,
                        Err(e) => log::error!("Error: {e}. Could not load config from path {}", self.config_path),
                    }
                }

                ui.menu_button("Options", |ui| {
                    ui.style_mut().wrap_mode = Some(eframe::egui::TextWrapMode::Extend);
                    ui.menu_button(self.config.language.to_string(), |ui| {
                        use strum::IntoEnumIterator;
                        for option in Language::iter().filter(|x| INGAME_LANGUAGES.contains(x)) {
                                ui.selectable_value(
                                    &mut self.config.language,
                                    option,
                                    option.to_string(),
                                );
                            }
                    });
                });
            });
        });

        CentralPanel::default()
            //.frame(egui::Frame::central_panel(style)).inner_margin(0.))
            .show(ctx, |ui| {
                let mut viewer = Viewer {
                    game_contexts: &self.game_contexts,
                    config: &self.config,
                };
                DockArea::new(&mut self.tree)
                    .show_close_buttons(true)
                    .tab_context_menus(true)
                    .draggable_tabs(true)
                    .show_tab_name_on_hover(true)
                    .show_leaf_close_all_buttons(true)
                    .show_secondary_button_hint(true)
                    .secondary_button_context_menu(true)
                    .secondary_button_on_modifier(true)
                    .show_inside(ui, &mut viewer);
                });
    }
}

const INGAME_LANGUAGES: [Language; 15] = [
    Language::Japanese,
    Language::English,
    Language::French,
    Language::German,
    Language::Italian,
    Language::Spanish,
    Language::Russian,
    Language::Polish,
    Language::PortugueseBr,
    Language::Korean,
    Language::TransitionalChinese,
    Language::SimplelifiedChinese,
    Language::Arabic,
    Language::Thai,
    Language::LatinAmericanSpanish,
];
