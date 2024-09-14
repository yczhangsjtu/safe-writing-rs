use crate::app::{content::Content, MyApp};
use crate::data_structures::PlainText;
use std::path::PathBuf;

use super::editor::EditorState;
use eframe::egui;
use egui::{Color32, RichText, TextEdit, Vec2};

#[derive(Debug, Default, Clone)]
pub struct NewFileState {
    filename: String,
    new_password: String,
    confirm_password: String,
    font_size: f32,
    data_dir: String,
}

impl NewFileState {
    pub fn new(filename: String, font_size: f32, data_dir: String) -> Self {
        Self {
            filename,
            new_password: "".to_string(),
            confirm_password: "".to_string(),
            font_size,
            data_dir,
        }
    }
    pub fn filename(&self) -> &String {
        &self.filename
    }
}

#[derive(Debug, Default, Clone)]
pub struct EncryptedFileState {
    filename: String,
    ciphertext: String,
    password: String,
    new_password: String,
    confirm_password: String,
    font_size: f32,
    data_dir: String,
}

impl EncryptedFileState {
    pub fn new(filename: String, ciphertext: String, font_size: f32, data_dir: String) -> Self {
        Self {
            filename,
            ciphertext,
            font_size,
            data_dir,
            ..Default::default()
        }
    }

    pub fn filename(&self) -> &String {
        &self.filename
    }
}

impl MyApp {
    pub(super) fn build_uninitialized_file(
        new_file_state: &mut NewFileState,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) -> Option<Content> {
        ui.add(
            TextEdit::singleline(&mut new_file_state.new_password)
                .password(true)
                .hint_text("New Password"),
        );
        ui.add(
            TextEdit::singleline(&mut new_file_state.confirm_password)
                .password(true)
                .hint_text("Confirm Password"),
        );
        ui.allocate_space(Vec2::new(0.0, 10.0));
        if ui
            .button(egui::WidgetText::RichText(
                RichText::from("Create").size(18.0).color(
                    if new_file_state.new_password == new_file_state.confirm_password {
                        Color32::BLACK
                    } else {
                        Color32::WHITE.gamma_multiply(0.3)
                    },
                ),
            ))
            .clicked()
            || ctx.input(|i| i.key_pressed(egui::Key::Enter))
        {
            if new_file_state.new_password.len() > 0
                && new_file_state.new_password == new_file_state.confirm_password
            {
                return Some(Content::PlainText(EditorState::empty(
                    new_file_state.filename.clone(),
                    new_file_state.font_size,
                    new_file_state.new_password.clone(),
                    new_file_state.data_dir.clone(),
                )));
            }
        }
        return None;
    }

    pub(super) fn build_encrypted_file(
        encrypted_file_state: &mut EncryptedFileState,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) -> Option<Content> {
        ui.add(
            TextEdit::singleline(&mut encrypted_file_state.password)
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
            match PlainText::decrypt(
                &encrypted_file_state.password,
                &encrypted_file_state.ciphertext,
            ) {
                Ok(plaintext) => {
                    let editor_state = EditorState::new(
                        encrypted_file_state.filename.clone(),
                        plaintext,
                        encrypted_file_state.password.clone(),
                        encrypted_file_state.font_size,
                        encrypted_file_state.data_dir.clone(),
                    );
                    return Some(Content::PlainText(editor_state));
                }
                Err(err) => {
                    return Some(Content::Error(format!("{:?}", err)));
                }
            }
        }
        ui.allocate_space(Vec2::new(0.0, 100.0));
        ui.add(
            TextEdit::singleline(&mut encrypted_file_state.new_password)
                .password(true)
                .hint_text("New Password"),
        );
        ui.add(
            TextEdit::singleline(&mut encrypted_file_state.confirm_password)
                .password(true)
                .hint_text("Confirm Password"),
        );
        if ui
            .button(
                egui::WidgetText::RichText(RichText::from("Change Password").size(18.0)).color(
                    if encrypted_file_state.new_password == encrypted_file_state.confirm_password {
                        Color32::BLACK
                    } else {
                        Color32::WHITE.gamma_multiply(0.3)
                    },
                ),
            )
            .clicked()
        {
            if encrypted_file_state.new_password == encrypted_file_state.confirm_password {
                match PlainText::decrypt(
                    &encrypted_file_state.password,
                    &encrypted_file_state.ciphertext,
                ) {
                    Ok(plaintext) => {
                        let ciphertext = plaintext.encrypt(&encrypted_file_state.new_password);
                        let path = PathBuf::from(encrypted_file_state.data_dir.clone())
                            .join(format!("{}.safe", encrypted_file_state.filename));
                        std::fs::write(path, &ciphertext).unwrap();
                        return Some(Content::Success(
                            "Password changed successfully".to_string(),
                        ));
                    }
                    Err(err) => {
                        return Some(Content::Error(format!("{:?}", err)));
                    }
                }
            }
        }
        return None;
    }
}
