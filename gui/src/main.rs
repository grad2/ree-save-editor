#[cfg(not(target_arch = "wasm32"))]
mod native {
    use clap::Parser;
    use eframe::egui;
    use ree_lib::{
        game_context::{AssetPaths, GameCtx},
        save::game::Game,
    };
    use ree_save_editor::{Config, app::TameApp, configure_fonts};
    use std::{
        collections::HashMap,
        sync::{Arc, RwLock},
    };

    fn parse_game(value: &str) -> Result<Game, String> {
        Game::from_string(value).ok_or_else(|| {
            format!(
                "unknown game '{value}'. Valid games: {}",
                Game::valid_names()
            )
        })
    }

    #[derive(Parser, Debug)]
    #[command(name = "ree-save-editor")]
    #[command(version, about, long_about = None)]
    struct GuiArgs {
        #[arg(short('f'), long)]
        file_name: Option<String>,

        #[arg(short('o'), long, default_value_t = String::from("outputs"))]
        out_dir: String,

        #[arg(long)]
        steamid: Option<String>,

        #[arg(short('g'), long, value_parser = parse_game, help = "Game profile to use, e.g. MHWILDS, RE9, or \"MH Wilds\"")]
        game: Option<Game>,

        #[arg(
            long,
            value_name = "LUA_FILE",
            help = "Run a Lua script and exit without launching the UI"
        )]
        run_script: Option<String>,

        #[arg(long)]
        rsz_path: Option<String>,
        #[arg(long)]
        enums_path: Option<String>,
        #[arg(long)]
        msgs_path: Option<String>,
        #[arg(long)]
        mappings_path: Option<String>,
        #[arg(long)]
        remap_path: Option<String>,

        #[cfg(target_os = "linux")]
        #[arg(long, default_value_t = shellexpand::full("~/.local/share/Steam/").unwrap_or_default().to_string())]
        steam_path: String,
        #[cfg(target_os = "windows")]
        #[arg(long, default_value_t = String::from("C:\\Program Files (x86)\\Steam"))]
        steam_path: String,
    }

    fn asset_paths_from_config(config: &Config, game: Game) -> AssetPaths {
        let mut asset_paths = AssetPaths::from_game(game);
        if config.rsz_path.is_some() {
            asset_paths.rsz = config.rsz_path.clone();
        }
        if config.enums_path.is_some() {
            asset_paths.enums = config.enums_path.clone();
        }
        if config.msgs_path.is_some() {
            asset_paths.msgs = config.msgs_path.clone();
        }
        if config.mappings_path.is_some() {
            asset_paths.mappings = config.mappings_path.clone();
        }
        if config.remap_path.is_some() {
            asset_paths.remap = config.remap_path.clone();
        }
        asset_paths
    }

    fn run_script_headless(config: &Config, script_path: &str) -> eframe::Result<()> {
        let game = config.game.unwrap_or(Game::MHWILDS);
        let mut contexts = HashMap::new();
        contexts.insert(game, GameCtx::new(&asset_paths_from_config(config, game)));

        let mut script_runner = ree_lib::bindings::runner::ScriptRunner::new();
        script_runner
            .lua
            .set_app_data(Arc::new(RwLock::new(contexts)));
        script_runner
            .load_and_execute_from_file(script_path)
            .map_err(|err| {
                eframe::Error::AppCreation(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    err.to_string(),
                )))
            })?;
        Ok(())
    }

    pub fn main() -> eframe::Result<()> {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
        let args = GuiArgs::parse();
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_drag_and_drop(true),
            ..Default::default()
        };

        let config = Config {
            file_name: args.file_name,
            out_dir: args.out_dir,
            steamid: args.steamid,
            game: args.game,
            rsz_path: args.rsz_path,
            enums_path: args.enums_path,
            msgs_path: args.msgs_path,
            mappings_path: args.mappings_path,
            steam_path: args.steam_path,
            remap_path: args.remap_path,
        };

        if let Some(script_path) = args.run_script {
            return run_script_headless(&config, &script_path);
        }

        eframe::run_native(
            "ree-save-editor",
            options,
            Box::new(|_cc| {
                configure_fonts(&_cc.egui_ctx);
                egui_extras::install_image_loaders(&_cc.egui_ctx);
                //Ok(Box::new(TameApp::new(config, _cc)))
                Ok(Box::new(TameApp::new(config)))
            }),
        )
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    native::main()
}

#[cfg(target_arch = "wasm32")]
fn main() {
    panic!("This binary cannot be run on WASM. Use the library entry point.");
}
