use super::MyApp;

use crate::{
    app::content::Content,
    data_structures::{Passage, PlainText},
};
use std::path::PathBuf;

use eframe::egui;
use egui::{Color32, FontFamily, FontId, FontSelection, Key, RichText, TextEdit, Vec2};

impl MyApp {
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
                        "Sure to delete this passage? Titled: {}",
                        plaintext.title_of_passage(to_delete_passage_index).unwrap()
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

    fn build_toggle_button(&mut self, width: f32, _ctx: &egui::Context, ui: &mut egui::Ui) {
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
    }

    fn build_add_button(
        &mut self,
        selected_index: usize,
        width: f32,
        _ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) {
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
    }

    fn build_save_button(
        &mut self,
        filename: &str,
        plaintext: &PlainText,
        width: f32,
        _ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) {
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
            self.save(filename.to_string(), plaintext);
        }
    }

    fn build_save_lock_button(
        &mut self,
        filename: &str,
        plaintext: &PlainText,
        width: f32,
        _ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) {
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
            self.save_and_lock(filename.to_string(), plaintext);
        }
    }

    fn build_move_button(
        &mut self,
        selected_index: usize,
        up: bool,
        width: f32,
        _ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) {
        if ui
            .add(
                egui::Button::new(egui::WidgetText::RichText(
                    RichText::from(if up { "Move Up" } else { "Move Down" })
                        .size(18.0)
                        .color(Color32::WHITE),
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
                    if up && selected_index > 0 {
                        plaintext.swap(selected_index, selected_index - 1);
                        self.dirty = true;
                        Some(())
                    } else if !up && selected_index < plaintext.num_passages() - 1 {
                        plaintext.swap(selected_index, selected_index + 1);
                        self.dirty = true;
                        Some(())
                    } else {
                        None
                    }
                })
                .is_some()
            {
                if up {
                    self.content.decrease_selected_index();
                } else {
                    self.content.increase_selected_index();
                }
            };
        }
    }

    fn build_rename_button(
        &mut self,
        selected_index: usize,
        plaintext: &PlainText,
        width: f32,
        _ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) {
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
    }

    fn build_delete_button(
        &mut self,
        selected_index: usize,
        width: f32,
        _ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) {
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
    }

    fn build_read_temp_button(
        &mut self,
        selected_index: usize,
        width: f32,
        _ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) {
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
    }

    fn build_append_file_button(&mut self, width: f32, ctx: &egui::Context, ui: &mut egui::Ui) {
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

                let path = PathBuf::from(self.data_dir.clone()).join(format!("{}.safe", filename));
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
                                    self.error_appending_another_file =
                                        Some(format!("File {}.safe has invalid format", filename));
                                } else {
                                    match PlainText::decrypt(
                                        password, content[0], content[1], content[2],
                                    ) {
                                        Ok(appended_plaintext) => {
                                            self.content.get_plaintext_mut().map(|plaintext| {
                                                plaintext.append_plaintext(appended_plaintext);
                                                self.dirty = true;
                                                self.appending_another_file = None;
                                                self.error_appending_another_file = None;
                                            });
                                        }
                                        Err(err) => {
                                            self.error_appending_another_file = Some(format!(
                                                "Failed to decrypt file {}.safe: {:?}",
                                                filename, err
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            self.error_appending_another_file =
                                Some(format!("Failed to read file {}.safe: {}", filename, err));
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
            self.build_toggle_button(width, ctx, ui);
            if ctx.input(|i| i.key_pressed(Key::S) && i.modifiers.command) {
                self.save(filename.clone(), plaintext);
            }
            if ctx.input(|i| i.key_pressed(Key::L) && i.modifiers.command) {
                self.save_and_lock(filename.clone(), plaintext);
            }
            if self.show_passage_operation_buttons {
                self.build_add_button(selected_index, width, ctx, ui);
                self.build_save_button(filename, plaintext, width, ctx, ui);
                self.build_save_lock_button(filename, plaintext, width, ctx, ui);
                self.build_move_button(selected_index, true, width, ctx, ui);
                self.build_move_button(selected_index, false, width, ctx, ui);
                self.build_rename_button(selected_index, plaintext, width, ctx, ui);
                self.build_delete_button(selected_index, width, ctx, ui);
                self.build_read_temp_button(selected_index, width, ctx, ui);
                self.build_append_file_button(width, ctx, ui);
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
}
