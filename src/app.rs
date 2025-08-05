use homedir::my_home;
use std::path::PathBuf;

use eframe::egui;
use egui::{Color32, FontDefinitions, Theme};

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
    file_names: Vec<String>,
    creating_new_file: Option<String>,
    waiting_for_password_for_safe_note: Option<(PathBuf, String, String)>,
    config: Config,
}

impl MyApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (config, file_names) = Self::get_config_and_filenames();

        let mut fonts = egui::FontDefinitions::default();

        Self::load_local_font_and_insert("LXGW", 0, &mut fonts);
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
            file_names,
            config,
            ..Default::default()
        }
    }

    fn load_local_font_and_insert(name: &'static str, index: usize, fonts: &mut FontDefinitions) {
        // SystemSource::new()
        //     .all_families()
        //     .unwrap()
        //     .iter()
        //     .for_each(|name| println!("Family: {}", name));
        let font_data = if name == "LXGW" {
            egui::FontData::from_static(include_bytes!("../assets/LXGWWenKaiGB-Regular.ttf"))
        } else {
            panic!("Unknown font name: {}", name);
        };

        fonts
            .font_data
            .insert(name.to_owned(), std::sync::Arc::new(font_data));

        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(index, name.to_owned());
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .push(name.to_owned());
        fonts
            .families
            .entry(egui::FontFamily::Name(name.into()))
            .or_default()
            .push(name.to_owned());
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

    fn data_dir(&self) -> &String {
        &self.config.data_dir
    }

    fn formatted_data_dir(&self) -> String {
        // If the prefix is home directory of the current user, replace it
        // with "$HOME"
        if self
            .data_dir()
            .starts_with(my_home().unwrap().unwrap().to_str().unwrap())
        {
            let folder_icon = egui_material_icons::icons::ICON_FOLDER;
            return format!(
                "{} $HOME{}",
                folder_icon,
                &self.config.data_dir[my_home().unwrap().unwrap().to_str().unwrap().len()..]
            );
        }
        self.data_dir().to_string()
    }

    #[allow(unused)]
    fn font_size(&self) -> f32 {
        self.config.font_size
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_theme(Theme::Light);
        if let Some(content) = self.next_content.take() {
            self.content = content;
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(Color32::BLACK))
            .show(ctx, |ui| self.main_layout(ctx, ui));
    }
}
