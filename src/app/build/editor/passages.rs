use super::MyApp;

pub use super::state::EditorState;

use crate::app::build::button_style::ButtonStyle;
use crate::{app::content::Content, data_structures::PlainText};

use eframe::egui;
use egui::{Color32, FontFamily, FontId, FontSelection, Key, RichText, Vec2};

impl MyApp {
    pub(super) fn build_passage_list_menu_button(
        editor_state: &mut EditorState,
        width: f32,
        ui: &mut egui::Ui,
        next_content: &mut Option<Content>,
    ) {
        egui::menu::menu_custom_button(
            ui,
            Self::make_control_button("...", ButtonStyle::Normal, false)
                .min_size(Vec2::new(24.0, 24.0)),
            |ui| {
                Self::build_preview_button(editor_state, width, ui);
                Self::build_insert_image_button(editor_state, width, ui);
                Self::build_insert_safe_image_button(editor_state, width, ui);
                Self::build_clean_nonexist_image_button(editor_state, width, ui);
                Self::build_save_lock_button(next_content, editor_state, width, ui);
                Self::build_rename_button(editor_state, width, ui);
                Self::build_delete_button(editor_state, editor_state.selected_index(), width, ui);
                Self::build_read_temp_button(
                    editor_state,
                    editor_state.selected_index(),
                    width,
                    ui,
                );
                Self::build_append_file_button(editor_state, width, ui);
            },
        );
    }

    pub(super) fn try_appending_safe_file_content(
        editor_state: &mut EditorState,
        filename: &str,
        content: &str,
        password: &str,
        ui: &mut egui::Ui,
    ) {
        match PlainText::decrypt(password, content) {
            Ok(appended_plaintext) => {
                editor_state
                    .plaintext_mut()
                    .append_plaintext(&appended_plaintext);
                appended_plaintext.images().iter().for_each(|image| {
                    EditorState::insert_image(editor_state, image, ui.ctx());
                });
                editor_state.dirty = true;
                editor_state.appending_another_file = None;
                editor_state.error_appending_another_file = None;
            }
            Err(err) => {
                editor_state.error_appending_another_file = Some(format!(
                    "Failed to decrypt file {}.safe: {:?}",
                    filename, err
                ));
            }
        }
    }

    pub(super) fn build_passage_list(
        next_content: &mut Option<Content>,
        editor_state: &mut EditorState,
        width: f32,
        ui: &mut egui::Ui,
    ) {
        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                Self::build_add_button(editor_state, width, ui);
                Self::build_save_button(editor_state, width, ui);
                Self::build_move_button(
                    editor_state,
                    editor_state.selected_index(),
                    true,
                    width,
                    ui,
                );
                Self::build_move_button(
                    editor_state,
                    editor_state.selected_index(),
                    false,
                    width,
                    ui,
                );
                Self::build_passage_list_menu_button(editor_state, width, ui, next_content);
            });
            if ui
                .ctx()
                .input(|i| i.key_pressed(Key::S) && i.modifiers.command)
            {
                Self::save(editor_state);
            }
            if ui
                .ctx()
                .input(|i| i.key_pressed(Key::L) && i.modifiers.command)
            {
                Self::save_and_lock(next_content, editor_state);
            }
            egui::ScrollArea::vertical()
                .id_salt("passage_list")
                .max_height(f32::INFINITY)
                .auto_shrink([true, false])
                .max_width(width)
                .show(ui, |ui| {
                    if editor_state.plaintext.is_empty() {
                        Self::build_new_passage_add(editor_state, 0, width, ui);
                    }
                    (0..editor_state.plaintext.num_passages()).for_each(|i| {
                        if editor_state
                            .editing_passage_name
                            .clone()
                            .map(|(_, index)| index)
                            == Some(i)
                        {
                            Self::build_passage_rename(editor_state, width, ui);
                        } else {
                            Self::build_passage_button(editor_state, i, width, ui);
                        }
                    });
                });
        });
    }

    fn build_new_passage_add(
        editor_state: &mut EditorState,
        current_index: usize,
        width: f32,
        ui: &mut egui::Ui,
    ) {
        if let Some((ref mut title, to_insert_index)) = editor_state.add_new_passage {
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
        }
        if let Some((title, _)) = &editor_state.add_new_passage.clone() {
            if ui.ctx().input(|i| i.key_pressed(egui::Key::Enter)) {
                editor_state
                    .plaintext_mut()
                    .insert_new_passage(current_index, title.clone());
                editor_state.add_new_passage = None;
                editor_state.dirty = true;
            }
        }
    }

    fn build_passage_button(
        editor_state: &mut EditorState,
        curr_index: usize,
        width: f32,
        ui: &mut egui::Ui,
    ) {
        if ui
            .add(
                egui::Button::new(egui::WidgetText::RichText(
                    RichText::from(
                        editor_state
                            .plaintext
                            .title_of_passage(curr_index)
                            .unwrap()
                            .clone(),
                    )
                    .size(18.0)
                    .color(if curr_index == editor_state.selected_index() {
                        Color32::BLACK
                    } else {
                        Color32::WHITE
                    }),
                ))
                .min_size(Vec2::new(width, 24.0))
                .fill(if curr_index == editor_state.selected_index() {
                    Color32::WHITE.gamma_multiply(0.5)
                } else {
                    Color32::TRANSPARENT
                }),
            )
            .clicked()
        {
            if curr_index != editor_state.selected_index() {
                editor_state.selected_index = curr_index;
            }
        }
        Self::build_new_passage_add(editor_state, curr_index + 1, width, ui);
    }

    fn build_passage_rename(editor_state: &mut EditorState, width: f32, ui: &mut egui::Ui) {
        ui.add(
            egui::TextEdit::singleline(&mut editor_state.editing_passage_name.as_mut().unwrap().0)
                .min_size(Vec2::new(width, 24.0))
                .text_color(Color32::BLACK)
                .font(FontSelection::FontId(FontId::new(
                    18.0,
                    FontFamily::Proportional,
                ))),
        );
        if ui.ctx().input(|i| i.key_pressed(egui::Key::Enter)) {
            let selected_index = editor_state.selected_index;
            let title = editor_state.editing_passage_name.clone().unwrap().0;
            editor_state
                .plaintext_mut()
                .set_title(selected_index, title);
            editor_state.editing_passage_name = None;
            editor_state.dirty = true;
        }
    }

    fn build_add_button(editor_state: &mut EditorState, _width: f32, ui: &mut egui::Ui) {
        if ui
            .add(
                Self::make_control_button(
                    egui_material_icons::icons::ICON_ADD,
                    ButtonStyle::Normal,
                    false,
                )
                .min_size(Vec2::new(24.0, 24.0)),
            )
            .clicked()
        {
            editor_state.add_new_passage =
                Some(("".to_string(), editor_state.selected_index() + 1));
            editor_state.editing_passage_name = None;
        }
    }

    fn build_save_button(editor_state: &mut EditorState, _width: f32, ui: &mut egui::Ui) {
        if ui
            .add(
                Self::make_control_button(
                    egui_material_icons::icons::ICON_SAVE,
                    ButtonStyle::Normal,
                    !editor_state.dirty,
                )
                .min_size(Vec2::new(24.0, 24.0)),
            )
            .clicked()
            && editor_state.dirty
        {
            Self::save(editor_state);
        }
    }

    fn build_save_lock_button(
        next_content: &mut Option<Content>,
        editor_state: &mut EditorState,
        width: f32,
        ui: &mut egui::Ui,
    ) {
        if ui
            .add(
                Self::make_control_button("Save & Lock", ButtonStyle::WarningInMenu, false)
                    .min_size(Vec2::new(width, 24.0)),
            )
            .clicked()
        {
            Self::save_and_lock(next_content, editor_state);
        }
    }

    fn build_move_button(
        editor_state: &mut EditorState,
        selected_index: usize,
        up: bool,
        _width: f32,
        ui: &mut egui::Ui,
    ) {
        if ui
            .add(
                Self::make_control_button(
                    if up {
                        egui_material_icons::icons::ICON_MOVE_UP
                    } else {
                        egui_material_icons::icons::ICON_MOVE_DOWN
                    },
                    ButtonStyle::Normal,
                    false,
                )
                .min_size(Vec2::new(24.0, 24.0)),
            )
            .clicked()
        {
            let plaintext = editor_state.plaintext_mut();

            if up && selected_index > 0 {
                plaintext.swap(selected_index, selected_index - 1);
                editor_state.dirty = true;
                editor_state.decrease_selected_index();
            } else if !up && selected_index < plaintext.num_passages() - 1 {
                plaintext.swap(selected_index, selected_index + 1);
                editor_state.dirty = true;
                editor_state.increase_selected_index();
            }
        }
    }

    fn build_rename_button(editor_state: &mut EditorState, width: f32, ui: &mut egui::Ui) {
        if ui
            .add(
                Self::make_control_button("Rename", ButtonStyle::NormalInMenu, false)
                    .min_size(Vec2::new(width, 24.0)),
            )
            .clicked()
        {
            editor_state.add_new_passage = None;
            editor_state.editing_passage_name = Some((
                editor_state
                    .plaintext
                    .title_of_passage(editor_state.selected_index)
                    .unwrap(),
                editor_state.selected_index,
            ));
        }
    }

    fn build_delete_button(
        editor_state: &mut EditorState,
        selected_index: usize,
        width: f32,
        ui: &mut egui::Ui,
    ) {
        if ui
            .add(
                Self::make_control_button("Delete", ButtonStyle::WarningInMenu, false)
                    .min_size(Vec2::new(width, 24.0)),
            )
            .clicked()
        {
            editor_state.confirm_delete_passage = Some(selected_index);
        }
    }

    fn build_read_temp_button(
        editor_state: &mut EditorState,
        selected_index: usize,
        width: f32,
        ui: &mut egui::Ui,
    ) {
        if ui
            .add(
                Self::make_control_button(
                    "Read Temp",
                    ButtonStyle::NormalInMenu,
                    editor_state.preview_mode,
                )
                .min_size(Vec2::new(width, 24.0)),
            )
            .clicked()
            && !editor_state.preview_mode
        {
            let temp_file_path = editor_state.temp_path();
            if let Ok(temp_content) = std::fs::read_to_string(&temp_file_path) {
                let plaintext = editor_state.plaintext_mut();
                plaintext.set_content(
                    selected_index,
                    plaintext
                        .content_of_passage(selected_index)
                        .unwrap()
                        .to_string()
                        + &format!("\n\n{}", temp_content.trim()),
                );
                editor_state.dirty = true;
                if let Err(err) = std::fs::remove_file(&temp_file_path) {
                    println!("Failed to remove temp file: {}", err);
                }
            }
        }
    }

    fn build_append_file_button(editor_state: &mut EditorState, width: f32, ui: &mut egui::Ui) {
        if ui
            .add(
                Self::make_control_button("Append File", ButtonStyle::NormalInMenu, false)
                    .min_size(Vec2::new(width, 24.0)),
            )
            .clicked()
        {
            if editor_state.appending_another_file.is_some() {
                editor_state.appending_another_file = None;
            } else {
                editor_state.appending_another_file = Some((String::new(), String::new()));
            }
            editor_state.error_appending_another_file = None;
        }
        if let Some((ref mut filename, ref mut password)) = editor_state.appending_another_file {
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
        }
        if let Some((filename, password)) = editor_state.appending_another_file.clone() {
            if ui.ctx().input(|i| i.key_pressed(egui::Key::Enter)) && !filename.is_empty() {
                Self::try_appending_safe_file(editor_state, &filename, &password, ui);
                editor_state.appending_another_file = None;
            }
            if let Some(error) = &editor_state.error_appending_another_file {
                ui.add(egui::Label::new(egui::WidgetText::RichText(
                    RichText::from(error).color(Color32::RED),
                )));
            }
        }
    }
}
