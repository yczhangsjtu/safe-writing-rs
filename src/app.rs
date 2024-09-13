use crate::{
    data_structures::{Passage, PlainText},
    error::Error,
    safe_note::load_safe_note_file,
};
use homedir::my_home;
use std::path::PathBuf;

use eframe::egui;
use egui::{Color32, FontFamily, FontId, FontSelection, Key, RichText, TextEdit, Vec2, WidgetText};

mod config;
use config::Config;

mod content;
use content::Content;

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

impl MyApp {
    fn build_file_list(&mut self, width: f32, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::Frame::none()
            .fill(Color32::GRAY.gamma_multiply(0.2))
            .inner_margin(5.0)
            .show(ui, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    if ui
                        .add(
                            egui::Button::new(egui::WidgetText::RichText(
                                RichText::from("Create New File")
                                    .size(18.0)
                                    .color(Color32::WHITE),
                            ))
                            .min_size(Vec2::new(width, 24.0))
                            .fill(Color32::GRAY.gamma_multiply(0.5)),
                        )
                        .clicked()
                    {
                        if self.creating_new_file.is_none() {
                            self.creating_new_file = Some("".to_string());
                        } else {
                            self.creating_new_file = None;
                        }
                        self.error_creating_new_file = None;
                    }
                    if ui
                        .add(
                            egui::Button::new(egui::WidgetText::RichText(
                                RichText::from("Load Safe Notes File")
                                    .size(18.0)
                                    .color(Color32::WHITE),
                            ))
                            .min_size(Vec2::new(width, 24.0))
                            .fill(Color32::GRAY.gamma_multiply(0.5)),
                        )
                        .clicked()
                    {
                        if self.waiting_for_password_for_safe_note.is_none() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("JSON Files", &["json"])
                                .pick_file()
                            {
                                self.waiting_for_password_for_safe_note = Some(path);
                                self.password = "".to_string();
                                self.imported_file_name = "".to_string();
                            }
                        } else {
                            self.waiting_for_password_for_safe_note = None;
                        }
                    }
                    if ui
                        .add(
                            egui::Button::new(egui::WidgetText::RichText(
                                RichText::from("Refresh").size(18.0).color(Color32::WHITE),
                            ))
                            .min_size(Vec2::new(width, 24.0))
                            .fill(Color32::GRAY.gamma_multiply(0.5)),
                        )
                        .clicked()
                    {
                        let (_, file_names) = Self::get_config_and_filenames();
                        self.file_names = file_names;
                    }
                    if let Some(path) = self.waiting_for_password_for_safe_note.clone() {
                        ui.add(
                            TextEdit::singleline(&mut self.password)
                                .desired_width(width)
                                .font(FontSelection::FontId(FontId::new(
                                    18.0,
                                    FontFamily::Proportional,
                                )))
                                .hint_text("Password")
                                .password(true),
                        );
                        ui.add(
                            TextEdit::singleline(&mut self.imported_file_name)
                                .desired_width(width)
                                .font(FontSelection::FontId(FontId::new(
                                    18.0,
                                    FontFamily::Proportional,
                                )))
                                .hint_text("New Name"),
                        );
                        if ctx.input(|i| i.key_pressed(Key::Enter)) {
                            if !self.imported_file_name.is_empty() {
                                match load_safe_note_file(&self.password, &path) {
                                    Ok(safe_note) => {
                                        let plaintext = safe_note.into_plaintext();

                                        if self.file_names.contains(&self.imported_file_name) {
                                            self.content = Content::Error(format!(
                                                "File with name {} already exists",
                                                &self.imported_file_name
                                            ));
                                        } else {
                                            let content = plaintext.encrypt(&self.password);
                                            let path = PathBuf::from(&self.data_dir)
                                                .join(format!("{}.safe", &self.imported_file_name));
                                            if std::fs::write(path, content).is_ok() {
                                                self.file_names
                                                    .push(self.imported_file_name.clone());
                                                self.file_names.sort();
                                                self.content = Content::PlainText(
                                                    self.imported_file_name.clone(),
                                                    plaintext.clone(),
                                                    0,
                                                );
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        self.content = Content::Error(format!(
                                            "Error loading safenote file: {:?}",
                                            err
                                        ));
                                    }
                                }
                                self.waiting_for_password_for_safe_note = None;
                            }
                        }
                    }
                    if let Some(ref mut filename) = self.creating_new_file {
                        ui.add(
                            egui::TextEdit::singleline(filename)
                                .font(FontSelection::FontId(FontId::new(
                                    18.0,
                                    FontFamily::Proportional,
                                )))
                                .desired_width(width),
                        );
                        if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                            let path = PathBuf::from(self.data_dir.clone())
                                .join(format!("{}.safe", filename));
                            if path.exists() {
                                self.error_creating_new_file =
                                    Some(format!("File {} already exists", filename));
                            } else {
                                std::fs::write(path, "").unwrap();
                                self.file_names.push(filename.clone());
                                self.file_names.sort();
                                self.content = Content::NewFile(filename.clone());
                            }
                            self.creating_new_file = None;
                        }
                    }
                    if let Some(error) = &self.error_creating_new_file {
                        ui.add(egui::Label::new(egui::WidgetText::RichText(
                            RichText::from(error).color(Color32::RED),
                        )));
                    }
                    egui::ScrollArea::vertical()
                        .id_source("file_name_list")
                        .max_height(f32::INFINITY)
                        .auto_shrink([true, false])
                        .max_width(width)
                        .show(ui, |ui| {
                            self.file_names.clone().iter().for_each(|file_name| {
                                if let Err(Error::FailedToOpenFile(s)) =
                                    self.build_filename_button(file_name.clone(), width, ui)
                                {
                                    self.content = Content::Error(s);
                                }
                            });
                        });
                });
            });
    }

    fn clear_editor_input_fields(&mut self) {
        self.password = "".to_string();
        self.confirm_password = "".to_string();
        self.new_password = "".to_string();
        self.edited_text = "".to_string();
    }

    fn build_filename_button(
        &mut self,
        file_name: String,
        width: f32,
        ui: &mut egui::Ui,
    ) -> Result<(), Error> {
        let selected = self.content.get_file_name() == Some(&file_name);
        let disabled = self.dirty || selected;
        if ui
            .add(
                egui::Button::new(egui::WidgetText::RichText(
                    RichText::from(file_name.clone())
                        .size(18.0)
                        .color(if self.dirty {
                            Color32::WHITE.gamma_multiply(0.2)
                        } else if selected {
                            Color32::BLACK
                        } else {
                            Color32::WHITE
                        }),
                ))
                .min_size(Vec2::new(width, 24.0))
                .fill(if selected {
                    Color32::LIGHT_GRAY
                } else {
                    Color32::TRANSPARENT
                }),
            )
            .clicked()
            && !disabled
        {
            self.clear_editor_input_fields();
            let path = PathBuf::from(self.data_dir.clone()).join(format!("{}.safe", file_name));
            let content = std::fs::read(path).map_err(|err| {
                Error::FailedToOpenFile(format!("Failed to open file {}: {:?}", file_name, err))
            })?;

            let content = String::from_utf8(content).unwrap();
            if content.is_empty() {
                self.content = Content::NewFile(file_name);
            } else {
                let content: Vec<_> = content.split("\n").collect();
                if content.len() < 3 {
                    self.content = Content::Error("Invalid format".to_string());
                } else {
                    self.content = Content::Encrypted(
                        file_name.clone(),
                        content[0].to_string(),
                        content[1].to_string(),
                        content[2].to_string(),
                    );
                }
            }
        }
        Ok(())
    }

    fn build_uninitialized_file(
        &mut self,
        filename: String,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) {
        ui.add(
            TextEdit::singleline(&mut self.password)
                .password(true)
                .hint_text("New Password"),
        );
        ui.add(
            TextEdit::singleline(&mut self.confirm_password)
                .password(true)
                .hint_text("Confirm Password"),
        );
        ui.allocate_space(Vec2::new(0.0, 10.0));
        if ui
            .button(egui::WidgetText::RichText(
                RichText::from("Create").size(18.0).color(
                    if self.password == self.confirm_password {
                        Color32::BLACK
                    } else {
                        Color32::WHITE.gamma_multiply(0.3)
                    },
                ),
            ))
            .clicked()
            || ctx.input(|i| i.key_pressed(egui::Key::Enter))
        {
            if self.password.len() > 0 && self.password == self.confirm_password {
                self.content = Content::PlainText(filename, PlainText::empty(), 0);
                self.show_passage_operation_buttons = false;
            }
        }
    }

    fn build_encrypted_file(
        &mut self,
        filename: &String,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        iv: &str,
        data: &str,
        mac: &str,
    ) {
        ui.add(
            TextEdit::singleline(&mut self.password)
                .password(true)
                .hint_text("Password"),
        );
        ui.allocate_space(Vec2::new(0.0, 10.0));
        if ui
            .button(egui::WidgetText::RichText(
                RichText::from("Decrypt").size(18.0),
            ))
            .clicked()
            || ctx.input(|i| i.key_pressed(egui::Key::Enter))
        {
            match PlainText::decrypt(&self.password, iv, data, mac) {
                Ok(plaintext) => {
                    if let Some(text) = plaintext.get_first_passage_text() {
                        self.edited_text = text;
                    }
                    self.content = Content::PlainText(filename.clone(), plaintext, 0);
                    self.dirty = false;
                    self.add_new_passage = None;
                    self.editing_passage_name = None;
                    self.confirm_delete_passage = None;
                    self.show_passage_operation_buttons = false;
                }
                Err(err) => {
                    self.content = Content::Error(format!("{:?}", err));
                }
            }
        }
        ui.allocate_space(Vec2::new(0.0, 100.0));
        ui.add(
            TextEdit::singleline(&mut self.new_password)
                .password(true)
                .hint_text("New Password"),
        );
        ui.add(
            TextEdit::singleline(&mut self.confirm_password)
                .password(true)
                .hint_text("Confirm Password"),
        );
        if ui
            .button(
                egui::WidgetText::RichText(RichText::from("Change Password").size(18.0)).color(
                    if self.new_password == self.confirm_password {
                        Color32::BLACK
                    } else {
                        Color32::WHITE.gamma_multiply(0.3)
                    },
                ),
            )
            .clicked()
        {
            if self.new_password == self.confirm_password {
                match PlainText::decrypt(&self.password, iv, data, mac) {
                    Ok(plaintext) => {
                        let ciphertext = plaintext.encrypt(&self.new_password);
                        let path =
                            PathBuf::from(self.data_dir.clone()).join(format!("{}.safe", filename));
                        std::fs::write(path, &ciphertext).unwrap();
                        self.content =
                            Content::Success("Password changed successfully".to_string());
                        self.password = "".to_string();
                        self.new_password = "".to_string();
                        self.confirm_password = "".to_string();
                    }
                    Err(err) => {
                        self.content = Content::Error(format!("{:?}", err));
                    }
                }
            }
        }
    }

    fn build_editor(
        &mut self,
        filename: &String,
        plaintext: &PlainText,
        selected_index: usize,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) {
        egui::Frame::none()
            .fill(Color32::LIGHT_GRAY.gamma_multiply(0.1))
            .inner_margin(5.0)
            .show(ui, |ui| {
                self.build_passage_list(150.0, filename, plaintext, selected_index, ctx, ui);
            });
        if plaintext.is_empty() {
            ui.with_layout(
                egui::Layout::centered_and_justified(egui::Direction::TopDown),
                |ui| {
                    ui.add(egui::Label::new(egui::WidgetText::RichText(
                        RichText::from("Empty file").size(18.0),
                    )));
                },
            );
        } else if let Some(to_delete_passage_index) = self.confirm_delete_passage {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.allocate_space(Vec2::new(0.0, 200.0));
                ui.label(
                    egui::WidgetText::from(format!(
                        "Sure to delete this passage? Titled: {:?}",
                        plaintext.title_of_passage(to_delete_passage_index)
                    ))
                    .color(Color32::LIGHT_RED),
                );
                if ui
                    .add(
                        egui::Button::new(egui::WidgetText::RichText(
                            RichText::from("Delete").size(18.0).color(Color32::WHITE),
                        ))
                        .fill(Color32::RED),
                    )
                    .clicked()
                {
                    let mut plaintext = plaintext.clone();
                    plaintext.remove_passage(to_delete_passage_index);
                    let new_selected_index = if plaintext.is_empty() {
                        0
                    } else if selected_index >= plaintext.num_passages() {
                        selected_index - 1
                    } else {
                        selected_index
                    };
                    self.content =
                        Content::PlainText(filename.clone(), plaintext.clone(), new_selected_index);
                    if plaintext.num_passages() == 0 {
                        self.edited_text = "".to_string();
                    } else {
                        self.edited_text =
                            plaintext.content_of_passage(new_selected_index).unwrap();
                    }
                    self.confirm_delete_passage = None;
                    self.dirty = true;
                }
            });
        } else {
            egui::ScrollArea::vertical()
                .id_source(format!(
                    "editor:{}:{}",
                    filename,
                    plaintext.id_of_passage(selected_index).unwrap()
                ))
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if ui
                        .add(
                            TextEdit::multiline(&mut self.edited_text)
                                .frame(false)
                                .desired_width(f32::INFINITY)
                                .desired_rows(50)
                                .font(FontSelection::FontId(FontId::new(
                                    self.font_size,
                                    FontFamily::Proportional,
                                )))
                                .text_color(Color32::WHITE),
                        )
                        .changed()
                    {
                        self.content.get_plaintext_mut().map(|plaintext| {
                            plaintext.set_content(selected_index, self.edited_text.clone());
                            self.dirty = true;
                        });
                    }
                });
        }
    }

    fn build_passage_list(
        &mut self,
        width: f32,
        filename: &String,
        plaintext: &PlainText,
        selected_index: usize,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) {
        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
            if ui
                .add(
                    egui::Button::new(
                        egui::WidgetText::RichText(RichText::from("...").size(18.0))
                            .color(Color32::WHITE),
                    )
                    .min_size(Vec2::new(width, 24.0))
                    .fill(Color32::LIGHT_GREEN.gamma_multiply(0.3)),
                )
                .clicked()
            {
                self.show_passage_operation_buttons = !self.show_passage_operation_buttons;
            }
            if ctx.input(|i| i.key_pressed(Key::S) && i.modifiers.command) {
                self.save(filename.clone(), plaintext);
            }
            if ctx.input(|i| i.key_pressed(Key::L) && i.modifiers.command) {
                self.save_and_lock(filename.clone(), plaintext);
            }
            if self.show_passage_operation_buttons {
                if ui
                    .add(
                        egui::Button::new(
                            egui::WidgetText::RichText(RichText::from("Add").size(18.0))
                                .color(Color32::WHITE),
                        )
                        .min_size(Vec2::new(width, 24.0))
                        .fill(Color32::LIGHT_GREEN.gamma_multiply(0.3)),
                    )
                    .clicked()
                {
                    self.add_new_passage = Some(("".to_string(), selected_index + 1));
                    self.editing_passage_name = None;
                }
                if ui
                    .add(
                        egui::Button::new(egui::WidgetText::RichText(
                            RichText::from("Save").size(18.0).color(if self.dirty {
                                Color32::WHITE
                            } else {
                                Color32::LIGHT_GRAY.gamma_multiply(0.3)
                            }),
                        ))
                        .min_size(Vec2::new(width, 24.0))
                        .fill(Color32::LIGHT_GREEN.gamma_multiply(0.3)),
                    )
                    .clicked()
                    && self.dirty
                {
                    self.save(filename.clone(), plaintext);
                }
                if ui
                    .add(
                        egui::Button::new(egui::WidgetText::RichText(
                            RichText::from("Save & Lock")
                                .size(18.0)
                                .color(Color32::WHITE),
                        ))
                        .min_size(Vec2::new(width, 24.0))
                        .fill(Color32::LIGHT_RED.gamma_multiply(0.3)),
                    )
                    .clicked()
                {
                    self.save_and_lock(filename.clone(), plaintext);
                }
                if ui
                    .add(
                        egui::Button::new(egui::WidgetText::RichText(
                            RichText::from("Move Up").size(18.0).color(Color32::WHITE),
                        ))
                        .min_size(Vec2::new(width, 24.0))
                        .fill(Color32::LIGHT_GREEN.gamma_multiply(0.3)),
                    )
                    .clicked()
                {
                    if self
                        .content
                        .get_plaintext_mut()
                        .and_then(|plaintext| {
                            if selected_index > 0 {
                                plaintext.swap(selected_index, selected_index - 1);
                                self.dirty = true;
                                Some(())
                            } else {
                                None
                            }
                        })
                        .is_some()
                    {
                        self.content.decrease_selected_index();
                    };
                }
                if ui
                    .add(
                        egui::Button::new(egui::WidgetText::RichText(
                            RichText::from("Move Down").size(18.0).color(Color32::WHITE),
                        ))
                        .min_size(Vec2::new(width, 24.0))
                        .fill(Color32::LIGHT_GREEN.gamma_multiply(0.3)),
                    )
                    .clicked()
                {
                    if self
                        .content
                        .get_plaintext_mut()
                        .and_then(|plaintext| {
                            if selected_index < plaintext.num_passages() - 1 {
                                plaintext.swap(selected_index, selected_index + 1);
                                self.dirty = true;
                                Some(())
                            } else {
                                None
                            }
                        })
                        .is_some()
                    {
                        self.content.increase_selected_index();
                    };
                }
                if ui
                    .add(
                        egui::Button::new(egui::WidgetText::RichText(
                            RichText::from("Rename").size(18.0).color(Color32::WHITE),
                        ))
                        .min_size(Vec2::new(width, 24.0))
                        .fill(Color32::LIGHT_GREEN.gamma_multiply(0.3)),
                    )
                    .clicked()
                {
                    self.add_new_passage = None;
                    self.editing_passage_name = Some((
                        plaintext.title_of_passage(selected_index).unwrap(),
                        selected_index,
                    ));
                }
                if ui
                    .add(
                        egui::Button::new(egui::WidgetText::RichText(
                            RichText::from("Delete").size(18.0).color(Color32::WHITE),
                        ))
                        .min_size(Vec2::new(width, 24.0))
                        .fill(Color32::LIGHT_RED.gamma_multiply(0.3)),
                    )
                    .clicked()
                {
                    self.confirm_delete_passage = Some(selected_index);
                }
                if ui
                    .add(
                        egui::Button::new(egui::WidgetText::RichText(
                            RichText::from("Read Temp").size(18.0).color(Color32::WHITE),
                        ))
                        .min_size(Vec2::new(width, 24.0))
                        .fill(Color32::LIGHT_GREEN.gamma_multiply(0.3)),
                    )
                    .clicked()
                {
                    let temp_file_path = PathBuf::from(self.data_dir.clone()).join("temp.txt");
                    if let Ok(temp_content) = std::fs::read_to_string(&temp_file_path) {
                        self.content.get_plaintext_mut().map(|plaintext| {
                            self.edited_text += &format!("\n\n{}", temp_content.trim());
                            plaintext.set_content(selected_index, self.edited_text.clone());
                            self.dirty = true;
                        });
                        if let Err(err) = std::fs::remove_file(&temp_file_path) {
                            println!("Failed to remove temp file: {}", err);
                        }
                    }
                }
                if ui
                    .add(
                        egui::Button::new(egui::WidgetText::RichText(
                            RichText::from("Append File")
                                .size(18.0)
                                .color(Color32::WHITE),
                        ))
                        .min_size(Vec2::new(width, 24.0))
                        .fill(Color32::LIGHT_GREEN.gamma_multiply(0.3)),
                    )
                    .clicked()
                {
                    if self.appending_another_file.is_some() {
                        self.appending_another_file = None;
                    } else {
                        self.appending_another_file = Some((String::new(), String::new()));
                    }
                    self.error_appending_another_file = None;
                }
                if let Some((ref mut filename, ref mut password)) = self.appending_another_file {
                    ui.add(
                        egui::TextEdit::singleline(filename)
                            .font(FontSelection::FontId(FontId::new(
                                18.0,
                                FontFamily::Proportional,
                            )))
                            .hint_text("Name")
                            .desired_width(width),
                    );
                    ui.add(
                        egui::TextEdit::singleline(password)
                            .font(FontSelection::FontId(FontId::new(
                                18.0,
                                FontFamily::Proportional,
                            )))
                            .password(true)
                            .hint_text("Password")
                            .desired_width(width),
                    );
                    if ctx.input(|i| i.key_pressed(egui::Key::Enter)) && !filename.is_empty() {
                        if let Some(current_file_name) = self.content.get_file_name() {
                            if filename == current_file_name {
                                self.error_appending_another_file =
                                    Some("Cannot append to self".to_string());
                            }
                        }

                        let path =
                            PathBuf::from(self.data_dir.clone()).join(format!("{}.safe", filename));
                        if !path.exists() {
                            self.error_appending_another_file =
                                Some(format!("File {}.safe not exists", filename));
                        } else {
                            match std::fs::read(path) {
                                Ok(data) => {
                                    let content = String::from_utf8(data).unwrap();
                                    if content.is_empty() {
                                        self.error_appending_another_file =
                                            Some(format!("File {}.safe is empty", filename));
                                    } else {
                                        let content: Vec<_> = content.split("\n").collect();
                                        if content.len() < 3 {
                                            self.error_appending_another_file = Some(format!(
                                                "File {}.safe has invalid format",
                                                filename
                                            ));
                                        } else {
                                            match PlainText::decrypt(
                                                password, content[0], content[1], content[2],
                                            ) {
                                                Ok(appended_plaintext) => {
                                                    self.content.get_plaintext_mut().map(
                                                        |plaintext| {
                                                            plaintext.append_plaintext(
                                                                appended_plaintext,
                                                            );
                                                            self.dirty = true;
                                                            self.appending_another_file = None;
                                                            self.error_appending_another_file =
                                                                None;
                                                        },
                                                    );
                                                }
                                                Err(err) => {
                                                    self.error_appending_another_file =
                                                        Some(format!(
                                                            "Failed to decrypt file {}.safe: {:?}",
                                                            filename, err
                                                        ));
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(err) => {
                                    self.error_appending_another_file = Some(format!(
                                        "Failed to read file {}.safe: {}",
                                        filename, err
                                    ));
                                }
                            }
                        }
                        self.creating_new_file = None;
                    }
                    if let Some(error) = &self.error_appending_another_file {
                        ui.add(egui::Label::new(egui::WidgetText::RichText(
                            RichText::from(error).color(Color32::RED),
                        )));
                    }
                }
            }
            egui::ScrollArea::vertical()
                .id_source("passage_list")
                .max_height(f32::INFINITY)
                .auto_shrink([true, false])
                .max_width(width)
                .show(ui, |ui| {
                    if plaintext.is_empty() {
                        self.build_new_passage_add(0, width, ctx, ui);
                    }
                    plaintext
                        .passages()
                        .iter()
                        .enumerate()
                        .for_each(|(i, passage)| {
                            if self.editing_passage_name.clone().map(|(_, index)| index) == Some(i)
                            {
                                self.build_passage_rename(selected_index, width, ctx, ui);
                            } else {
                                self.build_passage_button(
                                    i,
                                    selected_index,
                                    passage,
                                    width,
                                    filename,
                                    plaintext,
                                    ctx,
                                    ui,
                                );
                            }
                        });
                });
        });
    }

    fn save(&mut self, filename: String, plaintext: &PlainText) {
        let path = PathBuf::from(self.data_dir.clone()).join(format!("{}.safe", filename));
        std::fs::write(path, plaintext.encrypt(&self.password)).unwrap();
        self.dirty = false;
    }

    fn save_and_lock(&mut self, filename: String, plaintext: &PlainText) {
        let path = PathBuf::from(self.data_dir.clone()).join(format!("{}.safe", filename));
        let ciphertext = plaintext.encrypt(&self.password);
        std::fs::write(path, &ciphertext).unwrap();
        self.dirty = false;
        let ciphertext: Vec<_> = ciphertext.split("\n").collect();
        self.content = Content::Encrypted(
            filename.clone(),
            ciphertext[0].to_string(),
            ciphertext[1].to_string(),
            ciphertext[2].to_string(),
        );
        self.clear_editor_input_fields();
    }

    fn build_new_passage_add(
        &mut self,
        current_index: usize,
        width: f32,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) {
        if let Some((ref mut title, to_insert_index)) = self.add_new_passage {
            if current_index > 0 && current_index != to_insert_index {
                // When current_index is 0, this function must be called when the plaintext is
                // empty. In this case, we will build this text field.
                // When current_index is not zero, this function is called after building the
                // passage at index current_index-1. In this case, we must check if this is
                // indeed the target index to insert.
                return;
            }
            ui.add(
                egui::TextEdit::singleline(title)
                    .font(FontSelection::FontId(FontId::new(
                        18.0,
                        FontFamily::Proportional,
                    )))
                    .desired_width(width)
                    .text_color(Color32::BLACK)
                    .hint_text("Passage Title"),
            );
            if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.content
                    .get_plaintext_mut()
                    .unwrap()
                    .insert_new_passage(current_index, title.clone());
                self.add_new_passage = None;
                self.dirty = true;
            }
        }
    }

    fn build_passage_button(
        &mut self,
        curr_index: usize,
        selected_index: usize,
        passage: &Passage,
        width: f32,
        filename: &String,
        plaintext: &PlainText,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) {
        if ui
            .add(
                egui::Button::new(egui::WidgetText::RichText(
                    RichText::from(passage.title().clone()).size(18.0).color(
                        if curr_index == selected_index {
                            Color32::BLACK
                        } else {
                            Color32::WHITE
                        },
                    ),
                ))
                .min_size(Vec2::new(width, 24.0))
                .fill(if curr_index == selected_index {
                    Color32::WHITE.gamma_multiply(0.5)
                } else {
                    Color32::TRANSPARENT
                }),
            )
            .clicked()
        {
            if curr_index != selected_index {
                self.content = Content::PlainText(filename.clone(), plaintext.clone(), curr_index);
                self.edited_text = plaintext.content_of_passage(curr_index).unwrap();
            }
        }
        self.build_new_passage_add(curr_index + 1, width, ctx, ui);
    }

    fn build_passage_rename(
        &mut self,
        selected_index: usize,
        width: f32,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) {
        ui.add(
            egui::TextEdit::singleline(&mut self.editing_passage_name.as_mut().unwrap().0)
                .min_size(Vec2::new(width, 24.0))
                .text_color(Color32::BLACK)
                .font(FontSelection::FontId(FontId::new(
                    18.0,
                    FontFamily::Proportional,
                ))),
        );
        if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            self.content.get_plaintext_mut().unwrap().set_title(
                selected_index,
                self.editing_passage_name.as_ref().unwrap().0.clone(),
            );
            self.editing_passage_name = None;
            self.dirty = true;
        }
    }
}
