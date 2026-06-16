use ree_lib::save::game::Game;

// Shared configuration for the headless save conversion binary and external wrappers.
#[derive(Debug, Clone)]
pub struct Config {
    pub file_name: Option<String>,
    pub out_dir: String,
    pub steamid: Option<String>,
    pub game: Option<Game>,
    pub rsz_path: Option<String>,
    pub enums_path: Option<String>,
    pub msgs_path: Option<String>,
    pub mappings_path: Option<String>,
    pub remap_path: Option<String>,
    #[cfg(not(target_arch = "wasm32"))]
    pub steam_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            file_name: None,
            out_dir: "outputs".to_string(),
            steamid: None,
            game: None,
            rsz_path: None,
            enums_path: None,
            msgs_path: None,
            mappings_path: None,
            remap_path: None,
            #[cfg(target_os = "windows")]
            steam_path: "C:\\Program Files (x86)\\Steam".to_string(),
            #[cfg(target_os = "linux")]
            steam_path: shellexpand::full("~/.local/share/Steam")
                .unwrap_or_default()
                .to_string(),
        }
    }
}
