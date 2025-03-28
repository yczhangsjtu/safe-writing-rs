use super::{
    editor::EditorState,
    locked::{EncryptedFileState, NewFileState},
    MyApp,
};
use crate::{
    app::content::Content,
    consts::{FILE_LIST_BUTTON_WIDTH, FILE_LIST_WIDTH},
    error::Error,
    safe_note::load_safe_note_file,
};
use std::{ffi::OsStr, path::PathBuf};

use eframe::egui;
use egui::{Color32, FontFamily, FontId, FontSelection, Key, RichText, TextEdit, Vec2};

impl MyApp {
    fn build_create_new_file_button(&mut self, ui: &mut egui::Ui) {
        if ui
            .add(Self::make_file_list_top_button(
                egui_material_icons::icons::ICON_ADD,
                self.is_dirty(),
            ))
            .clicked()
            && !self.is_dirty()
        {
            if self.creating_new_file.is_none() {
                self.creating_new_file = Some("".to_string());
            } else {
                self.creating_new_file = None;
            }
        }
    }

    fn build_file_list_menu_button(&mut self, ui: &mut egui::Ui) {
        let data_dir = self.data_dir().clone();
        egui::menu::menu_custom_button(
            ui,
            Self::make_file_list_top_button("...", self.is_dirty()),
            |ui| {
                if ui
                    .add(
                        egui::Button::new(egui::WidgetText::RichText(
                            RichText::from("Load Safe Notes File").size(18.0).color(
                                if self.is_dirty() {
                                    Color32::GRAY.gamma_multiply(0.2)
                                } else {
                                    Color32::BLACK
                                },
                            ),
                        ))
                        .min_size(Vec2::new(24.0, 24.0))
                        .fill(Color32::WHITE),
                    )
                    .clicked()
                    && !self.is_dirty()
                {
                    if self.waiting_for_password_for_safe_note.is_none() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("JSON Files", &["json"])
                            .pick_file()
                        {
                            let default_name = path
                                .file_stem()
                                .unwrap_or(OsStr::new(""))
                                .to_string_lossy()
                                .to_string();
                            self.waiting_for_password_for_safe_note =
                                Some((path, default_name, "".to_string()));
                        }
                    } else {
                        self.waiting_for_password_for_safe_note = None;
                    }
                }

                if let Some((path, ref mut new_file_name, ref mut password)) =
                    &mut self.waiting_for_password_for_safe_note
                {
                    ui.add(
                        TextEdit::singleline(new_file_name)
                            .desired_width(FILE_LIST_BUTTON_WIDTH)
                            .font(FontSelection::FontId(FontId::new(
                                18.0,
                                FontFamily::Proportional,
                            )))
                            .hint_text("New Name"),
                    );
                    ui.add(
                        TextEdit::singleline(password)
                            .desired_width(FILE_LIST_BUTTON_WIDTH)
                            .font(FontSelection::FontId(FontId::new(
                                18.0,
                                FontFamily::Proportional,
                            )))
                            .hint_text("Password")
                            .password(true),
                    );
                    if ui.ctx().input(|i| i.key_pressed(Key::Enter)) && !new_file_name.is_empty() {
                        match load_safe_note_file(password, &path) {
                            Ok(safe_note) => {
                                let plaintext = safe_note.into_plaintext();
                                if self.file_names.contains(new_file_name) {
                                    self.content = Content::Error(format!(
                                        "File with name {} already exists",
                                        new_file_name
                                    ));
                                } else {
                                    let content = plaintext.encrypt(password);
                                    let path = PathBuf::from(&data_dir)
                                        .join(format!("{}.safe", new_file_name));
                                    if std::fs::write(path, content).is_ok() {
                                        self.file_names.push(new_file_name.clone());
                                        self.file_names.sort();
                                        self.content = Content::PlainText(EditorState::new(
                                            new_file_name.clone(),
                                            plaintext.clone(),
                                            password.clone(),
                                            self.config.clone(),
                                            &ui.ctx(),
                                        ));
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
            },
        );
    }

    fn build_refresh_button(&mut self, ui: &mut egui::Ui) {
        if ui
            .add(Self::make_file_list_top_button(
                egui_material_icons::icons::ICON_REFRESH,
                self.is_dirty(),
            ))
            .clicked()
            && !self.is_dirty()
        {
            let (_, file_names) = Self::get_config_and_filenames();
            self.file_names = file_names;
        }
    }

    fn build_filename_button(&mut self, file_name: String, ui: &mut egui::Ui) -> Result<(), Error> {
        let selected = self.content.get_file_name() == Some(&file_name);
        let disabled = self.is_dirty() || selected;
        if ui
            .add(
                egui::Button::new(egui::WidgetText::RichText(
                    RichText::from(file_name.clone())
                        .size(18.0)
                        .color(if self.is_dirty() {
                            Color32::WHITE.gamma_multiply(0.2)
                        } else if selected {
                            Color32::BLACK
                        } else {
                            Color32::WHITE
                        }),
                ))
                .min_size(Vec2::new(FILE_LIST_BUTTON_WIDTH, 24.0))
                .fill(if selected {
                    Color32::LIGHT_GRAY
                } else {
                    Color32::TRANSPARENT
                }),
            )
            .clicked()
            && !disabled
        {
            let path = PathBuf::from(self.data_dir().clone()).join(format!("{}.safe", file_name));
            let content = std::fs::read(path).map_err(|err| {
                Error::FailedToOpenFile(format!("Failed to open file {}: {:?}", file_name, err))
            })?;

            let content = String::from_utf8(content).unwrap();
            if content.is_empty() {
                self.content = Content::NewFile(NewFileState::new(file_name, self.config.clone()));
            } else {
                self.content = Content::Encrypted(EncryptedFileState::new(
                    file_name,
                    content,
                    self.config.clone(),
                ));
            }
        }
        Ok(())
    }

    pub(super) fn build_file_list(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let data_dir = self.data_dir().clone();
        egui::Frame::new()
            .fill(Color32::GRAY.gamma_multiply(0.2))
            .inner_margin(5.0)
            .show(ui, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                        self.build_create_new_file_button(ui);
                        self.build_refresh_button(ui);
                        self.build_file_list_menu_button(ui);
                    });

                    if let Some(ref mut filename) = self.creating_new_file {
                        ui.add(
                            egui::TextEdit::singleline(filename)
                                .font(FontSelection::FontId(FontId::new(
                                    18.0,
                                    FontFamily::Proportional,
                                )))
                                .desired_width(FILE_LIST_BUTTON_WIDTH),
                        );
                        if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                            let path = PathBuf::from(&data_dir).join(format!("{}.safe", filename));
                            if path.exists() {
                                self.content =
                                    Content::Error(format!("File {} already exists", filename));
                            } else if filename.is_empty() {
                                // Do nothing when input none, i.e., just cancel
                            } else {
                                std::fs::write(path, "").unwrap();
                                self.file_names.push(filename.clone());
                                self.file_names.sort();
                                self.content = Content::NewFile(NewFileState::new(
                                    filename.clone(),
                                    self.config.clone(),
                                ));
                            }
                            self.creating_new_file = None;
                        }
                        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                            self.creating_new_file = None;
                        }
                    }

                    egui::ScrollArea::vertical()
                        .id_salt("file_name_list")
                        .max_height(f32::INFINITY)
                        .auto_shrink([true, false])
                        .max_width(FILE_LIST_WIDTH)
                        .show(ui, |ui| {
                            self.file_names.clone().iter().for_each(|file_name| {
                                if let Err(Error::FailedToOpenFile(s)) =
                                    self.build_filename_button(file_name.clone(), ui)
                                {
                                    self.content = Content::Error(s);
                                }
                            });
                        });
                });
            });
    }
}
