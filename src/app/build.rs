use super::{content::Content, MyApp};
use crate::data_structures::{Passage, PlainText};
use std::path::PathBuf;

use eframe::egui;
use egui::{
    Color32, FontFamily, FontId, FontSelection, InnerResponse, RichText, TextEdit, Vec2, WidgetText,
};

const FILE_LIST_WIDTH: f32 = 200.0;
const PASSWORD_SCREEN_TOP_SPACE: f32 = 200.0;
const INFO_TEXT_SIZE: f32 = 18.0;

mod editor;
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
