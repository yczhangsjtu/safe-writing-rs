use homedir::my_home;
use std::path::PathBuf;

use eframe::egui;
use egui::Color32;

mod config;
use config::Config;

mod content;
use content::Content;

mod build;

mod macos;
mod windows;

#[derive(Default)]
pub struct MyApp {
    content: Content,
    next_content: Option<Content>,
    font_size: f32,
    file_names: Vec<String>,
    data_dir: String,
    creating_new_file: Option<String>,
    password: String,
    new_password: (String, String),
    waiting_for_password_for_safe_note: Option<(PathBuf, String, String)>,
}

impl MyApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (config, file_names) = Self::get_config_and_filenames();

        let mut fonts = egui::FontDefinitions::default();

        #[cfg(target_os = "macos")]
        {
            Self::load_font_and_insert("Hei", "heiti", 0, &mut fonts);
            Self::load_font_and_insert("Heiti SC", "heiti sc", 0, &mut fonts);
            Self::load_font_and_insert("PingFang SC", "pingfang sc", 0, &mut fonts);
            Self::load_font_and_insert("Songti SC", "songti sc", 1, &mut fonts);
            Self::load_font_and_insert("SimSong", "simsong", 1, &mut fonts);
        }
        #[cfg(target_os = "windows")]
        {
            Self::load_font_and_insert("Microsoft YaHei UI", "yahei", 0, &mut fonts);
        }

        // Tell egui to use these fonts:
        cc.egui_ctx.set_fonts(fonts);

        Self {
            font_size: config.font_size,
            file_names,
            data_dir: config.data_dir,
            ..Default::default()
        }
    }

    fn get_config_and_filenames() -> (Config, Vec<String>) {
        let config_path = std::env::var("SAFE_WRITING_CONFIG_DIR")
            .map(|p| std::path::PathBuf::from(p))
            .unwrap_or(my_home().unwrap().unwrap().as_path().join(".safe_writing"));
        if !config_path.is_dir() {
            if config_path.exists() {
                panic!("Config path {} is not a directory", config_path.display());
            }
            std::fs::create_dir_all(&config_path).unwrap();
        }

        let config_file = config_path.join("config.toml");
        let config = if !config_file.exists() {
            let config = Config {
                font_size: 24.0,
                data_dir: config_path.to_str().unwrap().to_owned(),
            };
            std::fs::write(config_file.clone(), toml::to_string(&config).unwrap()).unwrap();
            config
        } else {
            toml::from_str(&std::fs::read_to_string(config_file).unwrap()).unwrap()
        };

        let data_dir = std::path::PathBuf::from(&config.data_dir);
        if !data_dir.is_dir() {
            if data_dir.exists() {
                panic!("Data dir {} is not a directory", data_dir.display());
            }
            std::fs::create_dir_all(&data_dir).unwrap();
        }
        let mut file_names = std::fs::read_dir(&data_dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .filter_map(|filename| {
                if filename.ends_with(".safe") {
                    Some(filename[0..filename.len() - 5].to_string())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        file_names.sort();
        (config, file_names)
    }

    fn is_dirty(&self) -> bool {
        match &self.content {
            Content::PlainText(ref editor_state) => editor_state.is_dirty(),
            _ => false,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(content) = self.next_content.take() {
            self.content = content;
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(Color32::BLACK))
            .show(ctx, |ui| self.main_layout(ctx, ui));
    }
}
