use super::MyApp;
use crate::{app::content::Content, error::Error, safe_note::load_safe_note_file};
use std::path::PathBuf;

use eframe::egui;
use egui::{Color32, FontFamily, FontId, FontSelection, Key, RichText, TextEdit, Vec2};

impl MyApp {
    fn build_create_new_file_button(&mut self, width: f32, ctx: &egui::Context, ui: &mut egui::Ui) {
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
                let path = PathBuf::from(self.data_dir.clone()).join(format!("{}.safe", filename));
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
    }

    pub(super) fn build_file_list(&mut self, width: f32, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::Frame::none()
            .fill(Color32::GRAY.gamma_multiply(0.2))
            .inner_margin(5.0)
            .show(ui, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    self.build_create_new_file_button(width, ctx, ui);
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
}
