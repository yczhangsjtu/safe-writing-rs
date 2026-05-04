use homedir::my_home;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

fn default_llamacpp_url() -> String {
    "http://localhost:8080".to_string()
}

fn default_font_size() -> f32 {
    24.0
}

fn default_theme() -> String {
    "dark".to_string()
}

fn default_sidebar_width() -> f32 {
    200.0
}

fn default_passage_list_width() -> f32 {
    160.0
}

fn default_copilot_width() -> f32 {
    320.0
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(default = "default_font_size")]
    pub font_size: f32,
    pub data_dir: String,
    #[serde(default = "default_llamacpp_url")]
    pub llamacpp_url: String,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_sidebar_width")]
    pub sidebar_width: f32,
    #[serde(default = "default_passage_list_width")]
    pub passage_list_width: f32,
    #[serde(default = "default_copilot_width")]
    pub copilot_width: f32,
}

impl Default for Config {
    fn default() -> Self {
        let config_dir = get_config_dir();
        let data_dir = config_dir.to_str().unwrap().to_owned();
        Self {
            font_size: 24.0,
            data_dir,
            llamacpp_url: "http://localhost:8080".to_string(),
            theme: "dark".to_string(),
            sidebar_width: 200.0,
            passage_list_width: 160.0,
            copilot_width: 320.0,
        }
    }
}

/// Get the configuration directory path
pub fn get_config_dir() -> PathBuf {
    std::env::var("SAFE_WRITING_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| my_home().unwrap().unwrap().as_path().join(".safe_writing"))
}

/// Get the configuration file path (config.toml inside the config directory)
pub fn get_config_file_path() -> PathBuf {
    get_config_dir().join("config.toml")
}

pub fn load_or_create_config() -> Config {
    let config_dir = get_config_dir();
    let config_file = get_config_file_path();

    // Ensure config directory exists
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir).expect("Failed to create config directory");
    }

    // If config file doesn't exist, create default
    if !config_file.exists() {
        let config = Config::default();
        std::fs::write(&config_file, toml::to_string(&config).expect("Failed to serialize config"))
            .expect("Failed to write config file");
        return config;
    }

    // Load existing config
    let content = std::fs::read_to_string(&config_file).expect("Failed to read config file");
    toml::from_str(&content).unwrap_or_default()
}

pub fn save_config(config: &Config) -> Result<(), String> {
    let config_file = get_config_file_path();
    std::fs::write(&config_file, toml::to_string(config).unwrap())
        .map_err(|e| format!("Failed to save config: {}", e))
}

pub fn ensure_data_dir(data_dir: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(data_dir);
    if !path.exists() {
        std::fs::create_dir_all(&path)
            .map_err(|e| format!("Failed to create data directory: {}", e))?;
    }
    if !path.is_dir() {
        return Err("Data path is not a directory".to_string());
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.font_size, 24.0);
        assert!(!config.data_dir.is_empty());
        assert_eq!(config.llamacpp_url, "http://localhost:8080");
        assert_eq!(config.theme, "dark");
        assert_eq!(config.sidebar_width, 200.0);
        assert_eq!(config.passage_list_width, 160.0);
        assert_eq!(config.copilot_width, 320.0);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("font_size"));
        assert!(toml_str.contains("data_dir"));
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = "font_size = 18.0\ndata_dir = \"/test/path\"\nllamacpp_url = \"http://test\"\ntheme = \"light\"";
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.font_size, 18.0);
        assert_eq!(config.data_dir, "/test/path");
        assert_eq!(config.llamacpp_url, "http://test");
        assert_eq!(config.theme, "light");
    }
}