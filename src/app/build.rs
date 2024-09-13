use super::{content::Content, MyApp};
use crate::{
    data_structures::{Passage, PlainText},
    error::Error,
};
use std::path::PathBuf;

use eframe::egui;
use egui::{
    Color32, FontFamily, FontId, FontSelection, InnerResponse, Key, RichText, TextEdit, Vec2,
    WidgetText,
};

const FILE_LIST_WIDTH: f32 = 200.0;
const PASSWORD_SCREEN_TOP_SPACE: f32 = 200.0;
const INFO_TEXT_SIZE: f32 = 18.0;

mod file_list;

impl MyApp {
    pub(super) fn main_layout(
        &mut self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) -> InnerResponse<()> {
        ui.label(WidgetText::RichText(
            RichText::new(self.data_dir.as_str()).color(Color32::WHITE),
        ));
        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
            self.build_file_list(FILE_LIST_WIDTH, ctx, ui);
            match self.content.clone() {
                Content::NewFile(filename) => {
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        ui.allocate_space(Vec2::new(0.0, PASSWORD_SCREEN_TOP_SPACE));
                        self.build_uninitialized_file(filename, ctx, ui);
                    });
                }
                Content::Encrypted(ref filename, ref iv, ref data, ref mac) => {
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        ui.allocate_space(Vec2::new(0.0, PASSWORD_SCREEN_TOP_SPACE));
                        self.build_encrypted_file(filename, ctx, ui, iv, data, mac);
                    });
                }
                Content::None => {
                    ui.with_layout(
                        egui::Layout::centered_and_justified(egui::Direction::TopDown),
                        |ui| {
                            ui.add(egui::Label::new(egui::WidgetText::RichText(
                                RichText::from("Please select a file to open").size(INFO_TEXT_SIZE),
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

    pub(super) fn build_uninitialized_file(
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

    pub(super) fn build_encrypted_file(
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

    pub(super) fn build_editor(
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
