use super::MyApp;

pub use super::state::EditorState;

use crate::app::build::button_style::ButtonStyle;
use crate::data_structures::PlainText;

use eframe::egui;
use egui::{Color32, FontFamily, FontId, FontSelection, RichText, Vec2};
use sha2::Digest;

impl MyApp {
    pub(super) fn build_insert_image_button(
        editor_state: &mut EditorState,
        width: f32,
        ui: &mut egui::Ui,
    ) {
        if ui
            .add(
                Self::make_control_button(
                    "Image",
                    ButtonStyle::NormalInMenu,
                    editor_state.preview_mode,
                )
                .min_size(Vec2::new(width, 24.0)),
            )
            .clicked()
            && !editor_state.preview_mode
        {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("PNG Files", &["png"])
                .pick_file()
            {
                let image = std::fs::read(path);
                if let Ok(image) = image {
                    editor_state.insert_image_at_cursor(image, ui.ctx());
                }
            }
        }
    }

    pub(super) fn build_insert_safe_image_button(
        editor_state: &mut EditorState,
        width: f32,
        ui: &mut egui::Ui,
    ) {
        if ui
            .add(
                Self::make_control_button(
                    "Safe Image",
                    ButtonStyle::NormalInMenu,
                    editor_state.preview_mode,
                )
                .min_size(Vec2::new(width, 24.0)),
            )
            .clicked()
            && !editor_state.preview_mode
        {
            if editor_state.inserting_safe_image.is_some() {
                editor_state.inserting_safe_image = None;
                editor_state.error_inserting_safe_image = None;
            } else {
                editor_state.inserting_safe_image =
                    Some(("".to_string(), "".to_string(), "".to_string()));
                editor_state.error_inserting_safe_image = None;
            }
        }
        if let Some((ref mut filename, ref mut image_digest, ref mut password)) =
            editor_state.inserting_safe_image
        {
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
                egui::TextEdit::singleline(image_digest)
                    .font(FontSelection::FontId(FontId::new(
                        18.0,
                        FontFamily::Proportional,
                    )))
                    .hint_text("Image ID")
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
        }
        if let Some((ref mut filename, ref mut image_digest, ref mut password)) =
            editor_state.inserting_safe_image.clone()
        {
            if ui.ctx().input(|i| i.key_pressed(egui::Key::Enter)) && !filename.is_empty() {
                Self::try_inserting_safe_image(
                    editor_state,
                    &filename,
                    &image_digest,
                    &password,
                    ui,
                );
            }
            if let Some(error) = &editor_state.error_inserting_safe_image {
                ui.add(egui::Label::new(egui::WidgetText::RichText(
                    RichText::from(error).color(Color32::RED),
                )));
            }
        }
    }

    pub(super) fn build_clean_nonexist_image_button(
        editor_state: &mut EditorState,
        width: f32,
        ui: &mut egui::Ui,
    ) {
        if ui
            .add(
                Self::make_control_button("Clean Images", ButtonStyle::NormalInMenu, false)
                    .min_size(Vec2::new(width, 24.0)),
            )
            .clicked()
        {
            editor_state.confirm_clean_nonexist_images = true;
        }
    }

    pub(super) fn try_inserting_safe_file_image(
        editor_state: &mut EditorState,
        filename: &str,
        image_digest: &str,
        content: &str,
        password: &str,
        ui: &mut egui::Ui,
    ) {
        match PlainText::decrypt(password, content) {
            Ok(appended_plaintext) => {
                let image = appended_plaintext.images().iter().find_map(|image| {
                    let digest = format!("{:x}", {
                        let mut hasher = sha2::Sha256::new();
                        hasher.update(image);
                        hasher.finalize()
                    });
                    if digest.starts_with(image_digest) {
                        Some(image)
                    } else {
                        None
                    }
                });
                if let Some(image) = image {
                    EditorState::insert_image_at_cursor(editor_state, image.clone(), ui.ctx());
                    editor_state.dirty = true;
                    editor_state.inserting_safe_image = None;
                    editor_state.error_inserting_safe_image = None;
                } else {
                    editor_state.error_inserting_safe_image = Some(format!("Image not found"));
                }
            }
            Err(err) => {
                editor_state.error_inserting_safe_image = Some(format!(
                    "Failed to decrypt file {}.safe: {:?}",
                    filename, err
                ));
            }
        }
    }

    fn try_inserting_safe_image(
        editor_state: &mut EditorState,
        filename: &String,
        image_digest: &String,
        password: &String,
        ui: &mut egui::Ui,
    ) {
        if filename == &editor_state.filename {
            editor_state.error_inserting_safe_image =
                Some("No need to insert image in this file".to_string());
            return;
        }

        let path = editor_state.full_path_of(&filename);
        if !path.exists() {
            editor_state.error_inserting_safe_image =
                Some(format!("File {}.safe not exists", filename));
            return;
        }
        match std::fs::read(path) {
            Ok(data) => {
                let content = String::from_utf8(data).unwrap();
                if content.is_empty() {
                    editor_state.error_inserting_safe_image =
                        Some(format!("File {}.safe is empty", filename));
                } else {
                    Self::try_inserting_safe_file_image(
                        editor_state,
                        filename,
                        image_digest,
                        &content,
                        password,
                        ui,
                    );
                }
            }
            Err(err) => {
                editor_state.error_inserting_safe_image =
                    Some(format!("Failed to read file {}.safe: {}", filename, err));
            }
        }
    }
}
