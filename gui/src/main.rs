#[cfg(not(target_arch = "wasm32"))]
mod native {
    use clap::Parser;
    use ree_lib::save::{SaveFile, SaveOptions, game::Game};
    use ree_save_editor::Config;
    use std::{error::Error, fs, path::PathBuf};

    fn parse_game(value: &str) -> Result<Game, String> {
        Game::from_string(value).ok_or_else(|| {
            format!(
                "unknown game '{value}'. Valid games: {}",
                Game::valid_names()
            )
        })
    }

    fn parse_steamid(value: &str) -> Result<u64, String> {
        if let Some(hex) = value.strip_prefix("0x") {
            u64::from_str_radix(hex, 16).map_err(|err| err.to_string())
        } else {
            value.parse::<u64>().map_err(|err| err.to_string())
        }
    }

    #[derive(Parser, Debug)]
    #[command(name = "ree-save-editor")]
    #[command(version, about = "Headless RE Engine save conversion tool", long_about = None)]
    struct ConvertArgs {
        #[arg(
            short('f'),
            long,
            value_name = "SAVE_FILE",
            help = "Save file to convert"
        )]
        file_name: String,

        #[arg(short('o'), long, default_value_t = String::from("outputs"), help = "Directory for the converted save")]
        out_dir: String,

        #[arg(long, help = "Steam ID to write into the converted save")]
        steamid: Option<String>,

        #[arg(short('g'), long, value_parser = parse_game, help = "Game profile to use, e.g. MHWILDS, RE9, or \"MH Wilds\"")]
        game: Option<Game>,

        #[arg(long, help = "Citrus/Lime curve index to use for conversion")]
        curve_index: Option<usize>,

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

    fn convert_save(
        config: &Config,
        curve_index: Option<usize>,
    ) -> Result<PathBuf, Box<dyn Error>> {
        let input_path = config.file_name.as_ref().ok_or("--file-name is required")?;
        let game = config.game.unwrap_or(Game::MHWILDS);
        let expanded =
            shellexpand::full(input_path).unwrap_or(std::borrow::Cow::Borrowed(input_path));
        let input_path = PathBuf::from(expanded.as_ref());
        let data = fs::read(&input_path)
            .map_err(|err| format!("failed to read {}: {err}", input_path.display()))?;

        let mut options = SaveOptions::new(game);
        if let Some(steamid) = &config.steamid {
            options = options
                .id(parse_steamid(steamid).map_err(|err| format!("invalid --steamid: {err}"))?);
        }
        if let Some(curve_index) = curve_index {
            options = options.curve_index(curve_index);
        }

        let save = SaveFile::read_save(data, &mut options)
            .map_err(|err| format!("failed to load save: {err}"))?;
        let data = save
            .write_save(&options)
            .map_err(|err| format!("failed to write save: {err}"))?;

        let mut output_path = PathBuf::from(&config.out_dir);
        fs::create_dir_all(&output_path)
            .map_err(|err| format!("failed to create {}: {err}", output_path.display()))?;
        output_path.push(
            input_path
                .file_name()
                .unwrap_or_else(|| "data.bin".as_ref()),
        );
        fs::write(&output_path, data)
            .map_err(|err| format!("failed to save {}: {err}", output_path.display()))?;
        Ok(output_path)
    }

    pub fn main() -> Result<(), Box<dyn Error>> {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
        let args = ConvertArgs::parse();
        let config = Config {
            file_name: Some(args.file_name),
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

        let output_path = convert_save(&config, args.curve_index)?;
        println!("Converted save written to {}", output_path.display());
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    native::main()
}

#[cfg(target_arch = "wasm32")]
fn main() {
    panic!("This binary is headless and cannot be run on WASM.");
}
