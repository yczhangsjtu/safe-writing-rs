use super::MyApp;

pub use super::state::EditorState;

use crate::app::build::button_style::ButtonStyle;
use crate::{app::content::Content, data_structures::PlainText, png::read_png_metadata};
use std::collections::HashMap;

use eframe::egui;
use egui::{
    load::SizedTexture, text::CCursorRange, Color32, FontFamily, FontId, FontSelection, Image,
    Label, RichText, TextBuffer, TextEdit, TextureHandle, Vec2, WidgetText,
};

impl MyApp {
    pub(super) fn make_control_button(
        caption: &str,
        style: ButtonStyle,
        disabled: bool,
    ) -> egui::Button {
        egui::Button::new(egui::WidgetText::RichText(
            RichText::from(caption).size(18.0).color(if disabled {
                style.disabled_text_color()
            } else {
                style.text_color()
            }),
        ))
        .fill(if disabled {
            style.disabled_background_color()
        } else {
            style.background_color()
        })
    }

    pub(crate) fn build_editor(
        next_content: &mut Option<Content>,
        editor_state: &mut EditorState,
        ui: &mut egui::Ui,
    ) {
        egui::Frame::new()
            .fill(Color32::LIGHT_GRAY.gamma_multiply(0.1))
            .inner_margin(5.0)
            .show(ui, |ui| {
                Self::build_passage_list(next_content, editor_state, 150.0, ui);
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
                    .add(Self::make_control_button(
                        "Delete",
                        ButtonStyle::Danger,
                        false,
                    ))
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
        } else if editor_state.confirm_clean_nonexist_images {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.allocate_space(Vec2::new(0.0, 200.0));ui.label(
                    egui::WidgetText::from(
                        "Sure to clean? All images that are not used in the passage will be deleted."
                    )
                    .color(Color32::LIGHT_RED),
                );
                if ui
                    .add(Self::make_control_button(
                        "I'm Sure",
                        ButtonStyle::Danger,
                        false,
                    ))
                    .clicked()
                {
                    EditorState::clean_non_referenced_images(editor_state, ui.ctx());
                    editor_state.confirm_clean_nonexist_images = false;
                }
            });
        } else {
            egui::ScrollArea::vertical()
                .id_salt(format!(
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
                    if editor_state.preview_mode {
                        if let Some(text) = editor_state
                            .plaintext
                            .content_of_passage(editor_state.selected_index)
                        {
                            Self::build_reading_area(
                                &editor_state.plaintext,
                                &editor_state.image_map,
                                ui,
                                text,
                                font_size,
                                &mut editor_state.show_png_meta_data,
                            );
                        } else {
                            Self::build_no_passage_selected_screen(ui);
                        }
                    } else {
                        if let Some(edited_text) = editor_state
                            .plaintext
                            .content_of_passage_mut(editor_state.selected_index)
                        {
                            Self::build_editing_area(
                                ui,
                                edited_text,
                                &mut editor_state.dirty,
                                font_size,
                                &mut editor_state.text_to_insert,
                                &mut editor_state.image_to_insert,
                            );
                        } else {
                            Self::build_no_passage_selected_screen(ui);
                        }
                        if let Some(data) = editor_state.image_to_insert.take() {
                            if editor_state
                                .plaintext
                                .content_of_passage(editor_state.selected_index)
                                .is_some()
                            {
                                editor_state.insert_image_at_cursor(data, ui.ctx());
                            }
                        }
                    }
                });
        }
    }

    fn build_editing_area(
        ui: &mut egui::Ui,
        text: &mut String,
        dirty: &mut bool,
        font_size: f32,
        text_to_insert: &mut Option<String>,
        image_to_insert: &mut Option<Vec<u8>>,
    ) {
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::Max), |ui| {
            let screen_size = ui.ctx().input(|input| input.screen_rect());
            let editor_area = TextEdit::multiline(text)
                .frame(false)
                .desired_width(f32::INFINITY)
                .desired_rows(1.max(((screen_size.height() - 20f32) / (font_size * 1.4)) as usize))
                // .desired_rows(50)
                .font(FontSelection::FontId(FontId::new(
                    font_size,
                    FontFamily::Proportional,
                )))
                .text_color(Color32::WHITE);

            ui.ctx().input(|input| {
                input.raw.dropped_files.iter().for_each(|file| {
                    if let Some(path) = file.path.clone() {
                        if let Some(ext) = path.extension() {
                            if ext.to_string_lossy() == "png" {
                                let data = std::fs::read(path);
                                if let Ok(data) = data {
                                    *image_to_insert = Some(data);
                                }
                            }
                        }
                    }
                })
            });

            let response = ui.add(editor_area);
            if response.changed() {
                *dirty = true;
            }

            if let Some(mut state) = TextEdit::load_state(ui.ctx(), response.id) {
                let cursor = state.cursor.char_range();
                if let Some(text_to_insert) = text_to_insert.take() {
                    if let Some(cursor) = cursor {
                        let mut cursor = text.delete_selected_ccursor_range(cursor.sorted());
                        text.insert_text_at(&mut cursor, &text_to_insert, usize::MAX);
                        *dirty = true;
                        state.cursor.set_char_range(Some(CCursorRange::one(cursor)));
                    }
                }
            }
        });
    }

    fn build_reading_area(
        plaintext: &PlainText,
        image_map: &HashMap<String, (usize, TextureHandle)>,
        ui: &mut egui::Ui,
        text: &String,
        font_size: f32,
        show_png_meta_data: &mut Option<usize>,
    ) {
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
            // Split the passage by the image placeholders
            let mut remained_text = &text[..];
            while !remained_text.is_empty() {
                // Find the next "image!(" that is the start of the line and
                // that line ends with ")"
                let mut image_placeholder_index = remained_text.len();
                let mut start_search_pos = 0;
                while let Some(index) = &remained_text[start_search_pos..].find("image!(") {
                    if start_search_pos + *index > 0
                        && &remained_text[start_search_pos + *index - 1..start_search_pos + *index]
                            != "\n"
                    {
                        start_search_pos = start_search_pos + *index + 7;
                        continue;
                    }
                    let next_eol = &remained_text[start_search_pos + *index..]
                        .find('\n')
                        .map(|x| x + start_search_pos + index)
                        .unwrap_or(remained_text.len());
                    if *next_eol <= start_search_pos + *index + 7 {
                        break;
                    }
                    if &remained_text[next_eol - 1..*next_eol] != ")" {
                        start_search_pos = *next_eol;
                        continue;
                    }
                    let digest = &remained_text[start_search_pos + *index + 7..next_eol - 1];
                    if digest.len() != 64 || !digest.chars().all(|x| x.is_ascii_hexdigit()) {
                        start_search_pos = *next_eol;
                        continue;
                    }
                    image_placeholder_index = start_search_pos + *index;
                    break;
                }

                let text_before_image = remained_text[..image_placeholder_index].trim_end();
                if !text_before_image.is_empty() {
                    let area = Label::new(WidgetText::RichText(
                        RichText::new(text_before_image)
                            .size(font_size)
                            .color(Color32::WHITE),
                    ))
                    .selectable(true);
                    ui.add(area);
                }

                remained_text = &remained_text[image_placeholder_index..];
                if !remained_text.is_empty() {
                    assert!(remained_text.starts_with("image!"));
                    let digest = &remained_text[7..71];
                    remained_text = &remained_text[72..];

                    if let Some(image) = image_map.get(digest) {
                        let index = image.0;
                        let image = Image::from_texture(SizedTexture::from_handle(&image.1))
                            .max_width(ui.available_width())
                            .sense(egui::Sense::click());
                        let response = ui.add(image);
                        if response.clicked_by(egui::PointerButton::Primary) {
                            if show_png_meta_data.is_some() {
                                *show_png_meta_data = None;
                            } else {
                                *show_png_meta_data = Some(index);
                            }
                        }
                        if show_png_meta_data == &Some(index) {
                            if let Some(metadata) = read_png_metadata(&plaintext.images()[index]) {
                                ui.add(Label::new(WidgetText::RichText(RichText::new(metadata))));
                            }
                        }
                    } else {
                        let area = Label::new(WidgetText::RichText(
                            RichText::new(format!("Error loading image: {}", digest))
                                .color(Color32::RED),
                        ))
                        .selectable(true);
                        ui.add(area);
                    }
                }
            }
        });
    }

    fn build_no_passage_selected_screen(ui: &mut egui::Ui) {
        ui.with_layout(
            egui::Layout::centered_and_justified(egui::Direction::TopDown),
            |ui| {
                ui.add(egui::Label::new(egui::WidgetText::RichText(
                    RichText::from("No passage selected").size(18.0),
                )));
            },
        );
    }

    pub(super) fn build_preview_button(
        editor_state: &mut EditorState,
        width: f32,
        ui: &mut egui::Ui,
    ) {
        if ui
            .add(
                Self::make_control_button(
                    if editor_state.preview_mode {
                        "Edit"
                    } else {
                        "Preview"
                    },
                    ButtonStyle::NormalInMenu,
                    false,
                )
                .min_size(Vec2::new(width, 24.0)),
            )
            .clicked()
        {
            editor_state.preview_mode = !editor_state.preview_mode;
        }
    }
}
