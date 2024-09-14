use super::{locked::EncryptedFileState, MyApp};

use crate::{
    app::{config::Config, content::Content},
    data_structures::PlainText,
};
use std::path::PathBuf;

use eframe::egui;
use egui::{Color32, FontFamily, FontId, FontSelection, Key, RichText, TextEdit, Vec2};

#[derive(Default, Clone)]
pub struct EditorState {
    filename: String,
    plaintext: PlainText,
    selected_index: usize,
    dirty: bool,
    add_new_passage: Option<(String, usize)>,
    editing_passage_name: Option<(String, usize)>,
    confirm_delete_passage: Option<usize>,
    show_passage_operation_buttons: bool,
    appending_another_file: Option<(String, String)>,
    error_appending_another_file: Option<String>,
    password: String,
    config: Config,
}

impl EditorState {
    pub fn new(filename: String, plaintext: PlainText, password: String, config: Config) -> Self {
        EditorState {
            filename,
            plaintext,
            password,
            selected_index: 0,
            config,
            ..Default::default()
        }
    }

    pub fn empty(filename: String, password: String, config: Config) -> Self {
        Self::new(filename, PlainText::empty(), password, config)
    }

    pub fn filename(&self) -> &String {
        &self.filename
    }

    pub fn plaintext(&self) -> &PlainText {
        &self.plaintext
    }

    pub fn plaintext_mut(&mut self) -> &mut PlainText {
        &mut self.plaintext
    }

    pub fn edited_text_mut(&mut self) -> Option<&mut String> {
        self.plaintext.content_of_passage_mut(self.selected_index)
    }

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn decrease_selected_index(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn increase_selected_index(&mut self) {
        if self.selected_index < self.plaintext.num_passages() - 1 {
            self.selected_index += 1;
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn data_dir(&self) -> &String {
        &self.config.data_dir
    }

    pub fn font_size(&self) -> f32 {
        self.config.font_size
    }
}

impl MyApp {
    pub(super) fn build_editor(
        next_content: &mut Option<Content>,
        editor_state: &mut EditorState,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) {
        egui::Frame::none()
            .fill(Color32::LIGHT_GRAY.gamma_multiply(0.1))
            .inner_margin(5.0)
            .show(ui, |ui| {
                Self::build_passage_list(next_content, editor_state, 150.0, ctx, ui);
            });
        if editor_state.plaintext().is_empty() {
            ui.with_layout(
                egui::Layout::centered_and_justified(egui::Direction::TopDown),
                |ui| {
                    ui.add(egui::Label::new(egui::WidgetText::RichText(
                        RichText::from("Empty file").size(18.0),
                    )));
                },
            );
        } else if let Some(to_delete_passage_index) = editor_state.confirm_delete_passage {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.allocate_space(Vec2::new(0.0, 200.0));
                ui.label(
                    egui::WidgetText::from(format!(
                        "Sure to delete this passage? Titled: {}",
                        editor_state
                            .plaintext()
                            .title_of_passage(to_delete_passage_index)
                            .unwrap()
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
                    let plaintext = editor_state.plaintext_mut();
                    plaintext.remove_passage(to_delete_passage_index);
                    let num_passages = plaintext.num_passages();
                    let new_selected_index = if plaintext.is_empty() {
                        0
                    } else if editor_state.selected_index() >= num_passages {
                        editor_state.selected_index() - 1
                    } else {
                        editor_state.selected_index()
                    };
                    editor_state.dirty = true;
                    editor_state.selected_index = new_selected_index;
                    editor_state.confirm_delete_passage = None;
                }
            });
        } else {
            egui::ScrollArea::vertical()
                .id_source(format!(
                    "editor:{}:{}",
                    editor_state.filename(),
                    editor_state
                        .plaintext()
                        .id_of_passage(editor_state.selected_index())
                        .unwrap()
                ))
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let font_size = editor_state.font_size();
                    if let Some(edited_text) = editor_state.edited_text_mut() {
                        if ui
                            .add(
                                TextEdit::multiline(edited_text)
                                    .frame(false)
                                    .desired_width(f32::INFINITY)
                                    .desired_rows(50)
                                    .font(FontSelection::FontId(FontId::new(
                                        font_size,
                                        FontFamily::Proportional,
                                    )))
                                    .text_color(Color32::WHITE),
                            )
                            .changed()
                        {
                            editor_state.dirty = true;
                        }
                    } else {
                        ui.with_layout(
                            egui::Layout::centered_and_justified(egui::Direction::TopDown),
                            |ui| {
                                ui.add(egui::Label::new(egui::WidgetText::RichText(
                                    RichText::from("No passage selected").size(18.0),
                                )));
                            },
                        );
                    }
                });
        }
    }

    fn build_toggle_button(
        editor_state: &mut EditorState,
        width: f32,
        _ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) {
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
            editor_state.show_passage_operation_buttons =
                !editor_state.show_passage_operation_buttons;
        }
    }

    fn build_add_button(
        editor_state: &mut EditorState,
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
            editor_state.add_new_passage = Some(("".to_string(), selected_index + 1));
            editor_state.editing_passage_name = None;
        }
    }

    fn build_save_button(
        editor_state: &mut EditorState,
        width: f32,
        _ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) {
        if ui
            .add(
                egui::Button::new(egui::WidgetText::RichText(
                    RichText::from("Save")
                        .size(18.0)
                        .color(if editor_state.dirty {
                            Color32::WHITE
                        } else {
                            Color32::LIGHT_GRAY.gamma_multiply(0.3)
                        }),
                ))
                .min_size(Vec2::new(width, 24.0))
                .fill(Color32::LIGHT_GREEN.gamma_multiply(0.3)),
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
            Self::save_and_lock(next_content, editor_state);
        }
    }

    fn build_move_button(
        editor_state: &mut EditorState,
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

    fn build_rename_button(
        editor_state: &mut EditorState,
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
            editor_state.confirm_delete_passage = Some(selected_index);
        }
    }

    fn build_read_temp_button(
        editor_state: &mut EditorState,
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
            let temp_file_path = PathBuf::from(editor_state.data_dir().clone()).join("temp.txt");
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

    fn build_append_file_button(
        editor_state: &mut EditorState,
        width: f32,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) {
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
            if ctx.input(|i| i.key_pressed(egui::Key::Enter)) && !filename.is_empty() {
                Self::try_appending_safe_file(editor_state, &filename, &password);
                editor_state.appending_another_file = None;
            }
            if let Some(error) = &editor_state.error_appending_another_file {
                ui.add(egui::Label::new(egui::WidgetText::RichText(
                    RichText::from(error).color(Color32::RED),
                )));
            }
        }
    }

    fn try_appending_safe_file(
        editor_state: &mut EditorState,
        filename: &String,
        password: &String,
    ) {
        if filename == &editor_state.filename {
            editor_state.error_appending_another_file = Some("Cannot append to self".to_string());
        }

        let path =
            PathBuf::from(editor_state.data_dir().clone()).join(format!("{}.safe", filename));
        if !path.exists() {
            editor_state.error_appending_another_file =
                Some(format!("File {}.safe not exists", filename));
        } else {
            match std::fs::read(path) {
                Ok(data) => {
                    let content = String::from_utf8(data).unwrap();
                    if content.is_empty() {
                        editor_state.error_appending_another_file =
                            Some(format!("File {}.safe is empty", filename));
                    } else {
                        Self::try_appending_safe_file_content(
                            editor_state,
                            filename,
                            &content,
                            password,
                        );
                    }
                }
                Err(err) => {
                    editor_state.error_appending_another_file =
                        Some(format!("Failed to read file {}.safe: {}", filename, err));
                }
            }
        }
    }

    fn try_appending_safe_file_content(
        editor_state: &mut EditorState,
        filename: &str,
        content: &str,
        password: &str,
    ) {
        if content.len() < 3 {
            editor_state.error_appending_another_file =
                Some(format!("File {}.safe has invalid format", filename));
        } else {
            match PlainText::decrypt(password, content) {
                Ok(appended_plaintext) => {
                    editor_state
                        .plaintext_mut()
                        .append_plaintext(appended_plaintext);
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
    }

    fn build_passage_list(
        next_content: &mut Option<Content>,
        editor_state: &mut EditorState,
        width: f32,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) {
        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
            Self::build_toggle_button(editor_state, width, ctx, ui);
            if ctx.input(|i| i.key_pressed(Key::S) && i.modifiers.command) {
                Self::save(editor_state);
            }
            if ctx.input(|i| i.key_pressed(Key::L) && i.modifiers.command) {
                Self::save_and_lock(next_content, editor_state);
            }
            if editor_state.show_passage_operation_buttons {
                Self::build_add_button(editor_state, editor_state.selected_index(), width, ctx, ui);
                Self::build_save_button(editor_state, width, ctx, ui);
                Self::build_save_lock_button(next_content, editor_state, width, ctx, ui);
                Self::build_move_button(
                    editor_state,
                    editor_state.selected_index(),
                    true,
                    width,
                    ctx,
                    ui,
                );
                Self::build_move_button(
                    editor_state,
                    editor_state.selected_index(),
                    false,
                    width,
                    ctx,
                    ui,
                );
                Self::build_rename_button(editor_state, width, ctx, ui);
                Self::build_delete_button(
                    editor_state,
                    editor_state.selected_index(),
                    width,
                    ctx,
                    ui,
                );
                Self::build_read_temp_button(
                    editor_state,
                    editor_state.selected_index(),
                    width,
                    ctx,
                    ui,
                );
                Self::build_append_file_button(editor_state, width, ctx, ui);
            }
            egui::ScrollArea::vertical()
                .id_source("passage_list")
                .max_height(f32::INFINITY)
                .auto_shrink([true, false])
                .max_width(width)
                .show(ui, |ui| {
                    if editor_state.plaintext.is_empty() {
                        Self::build_new_passage_add(editor_state, 0, width, ctx, ui);
                    }
                    (0..editor_state.plaintext.num_passages()).for_each(|i| {
                        if editor_state
                            .editing_passage_name
                            .clone()
                            .map(|(_, index)| index)
                            == Some(i)
                        {
                            Self::build_passage_rename(editor_state, width, ctx, ui);
                        } else {
                            Self::build_passage_button(editor_state, i, width, ctx, ui);
                        }
                    });
                });
        });
    }

    fn build_new_passage_add(
        editor_state: &mut EditorState,
        current_index: usize,
        width: f32,
        ctx: &egui::Context,
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
            if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
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
        ctx: &egui::Context,
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
        Self::build_new_passage_add(editor_state, curr_index + 1, width, ctx, ui);
    }

    fn build_passage_rename(
        editor_state: &mut EditorState,
        width: f32,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) {
        ui.add(
            egui::TextEdit::singleline(&mut editor_state.editing_passage_name.as_mut().unwrap().0)
                .min_size(Vec2::new(width, 24.0))
                .text_color(Color32::BLACK)
                .font(FontSelection::FontId(FontId::new(
                    18.0,
                    FontFamily::Proportional,
                ))),
        );
        if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            let selected_index = editor_state.selected_index;
            let title = editor_state.editing_passage_name.clone().unwrap().0;
            editor_state
                .plaintext_mut()
                .set_title(selected_index, title);
            editor_state.editing_passage_name = None;
            editor_state.dirty = true;
        }
    }

    fn save(editor_state: &mut EditorState) {
        let path = PathBuf::from(editor_state.data_dir().clone())
            .join(format!("{}.safe", editor_state.filename));
        std::fs::write(path, editor_state.plaintext.encrypt(&editor_state.password)).unwrap();
        editor_state.dirty = false;
    }

    fn save_and_lock(next_content: &mut Option<Content>, editor_state: &mut EditorState) {
        let path = PathBuf::from(editor_state.data_dir().clone())
            .join(format!("{}.safe", editor_state.filename));
        let ciphertext = editor_state.plaintext.encrypt(&editor_state.password);
        std::fs::write(path, &ciphertext).unwrap();
        editor_state.dirty = false;
        *next_content = Some(Content::Encrypted(EncryptedFileState::new(
            editor_state.filename.clone(),
            ciphertext,
            editor_state.config().clone(),
        )));
    }
}
