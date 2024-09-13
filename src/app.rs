use crate::data_structures::PlainText;
use homedir::my_home;
use std::path::PathBuf;

use eframe::egui;
use egui::{Color32, RichText, Vec2, WidgetText};

mod config;
use config::Config;

mod content;
use content::Content;

mod build;

mod macos;
mod windows;

#[derive(Default)]
pub struct MyApp {
    content: Content<String, PlainText>,
    edited_text: String,
    dirty: bool,
    font_size: f32,
    file_names: Vec<String>,
    data_dir: String,
    creating_new_file: Option<String>,
    error_creating_new_file: Option<String>,
    password: String,
    add_new_passage: Option<(String, usize)>,
    editing_passage_name: Option<(String, usize)>,
    confirm_delete_passage: Option<usize>,
    confirm_password: String,
    new_password: String,
    show_passage_operation_buttons: bool,
    appending_another_file: Option<(String, String)>,
    error_appending_another_file: Option<String>,
    waiting_for_password_for_safe_note: Option<PathBuf>,
    imported_file_name: String,
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
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(Color32::BLACK))
            .show(ctx, |ui| {
                ui.label(WidgetText::RichText(
                    RichText::new(self.data_dir.as_str()).color(Color32::WHITE),
                ));
                ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                    self.build_file_list(200.0, ctx, ui);
                    match self.content.clone() {
                        Content::NewFile(filename) => {
                            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                                ui.allocate_space(Vec2::new(0.0, 200.0));
                                self.build_uninitialized_file(filename, ctx, ui);
                            });
                        }
                        Content::Encrypted(ref filename, ref iv, ref data, ref mac) => {
                            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                                ui.allocate_space(Vec2::new(0.0, 200.0));
                                self.build_encrypted_file(filename, ctx, ui, iv, data, mac);
                            });
                        }
                        Content::None => {
                            ui.with_layout(
                                egui::Layout::centered_and_justified(egui::Direction::TopDown),
                                |ui| {
                                    ui.add(egui::Label::new(egui::WidgetText::RichText(
                                        RichText::from("Please select a file to open").size(18.0),
                                    )));
                                },
                            );
                        }
                        Content::PlainText(filename, plaintext, selected_index) => {
                            self.build_editor(&filename, &plaintext, selected_index, ctx, ui);
                        }
                        Content::Error(err) => {
                            ui.with_layout(
                                egui::Layout::centered_and_justified(egui::Direction::TopDown),
                                |ui| {
                                    ui.add(egui::Label::new(egui::WidgetText::RichText(
                                        RichText::from(err).size(18.0).color(Color32::RED),
                                    )));
                                },
                            );
                        }
                        Content::Success(err) => {
                            ui.with_layout(
                                egui::Layout::centered_and_justified(egui::Direction::TopDown),
                                |ui| {
                                    ui.add(egui::Label::new(egui::WidgetText::RichText(
                                        RichText::from(err).size(18.0).color(Color32::GREEN),
                                    )));
                                },
                            );
                        }
                    }
                })
            });
    }
}
