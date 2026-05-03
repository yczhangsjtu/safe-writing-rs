use super::{content::Content, MyApp};
use eframe::egui;
use egui::{Color32, InnerResponse, RichText, Vec2, WidgetText};

const PASSWORD_SCREEN_TOP_SPACE: f32 = 200.0;
const INFO_TEXT_SIZE: f32 = 18.0;

pub(super) mod button_style;
pub(super) mod copilot;
pub(super) mod editor;
pub(super) mod file_list;
pub(super) mod locked;

impl MyApp {
    pub(super) fn main_layout(
        &mut self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) -> InnerResponse<()> {
        ui.label(WidgetText::RichText(
            RichText::new(self.formatted_data_dir().as_str())
                .color(Color32::CYAN)
                .into(),
        ));
        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
            self.build_file_list(ctx, ui);
            match &mut self.content {
                Content::NewFile(ref mut new_file_state) => {
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        ui.allocate_space(Vec2::new(0.0, PASSWORD_SCREEN_TOP_SPACE));
                        self.next_content = Self::build_uninitialized_file(new_file_state, ctx, ui);
                    });
                }
                Content::Encrypted(ref mut encrypted_file_state) => {
                    if self.creating_new_file == None {
                        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                            ui.allocate_space(Vec2::new(0.0, PASSWORD_SCREEN_TOP_SPACE));
                            let result = Self::build_encrypted_file(encrypted_file_state, ctx, ui);
                            if let Some(Content::PlainText(ref editor_state)) = result {
                                self.copilot.load_from_plaintext(editor_state.plaintext(), editor_state.filename());
                            }
                            self.next_content = result;
                        });
                    } else {
                        self.next_content = Some(Content::None);
                    }
                }
                Content::None => {
                    ui.with_layout(
                        egui::Layout::centered_and_justified(egui::Direction::TopDown),
                        |ui| {
                            ui.add(egui::Label::new(egui::WidgetText::RichText(
                                RichText::from("Please select a file to open")
                                    .size(INFO_TEXT_SIZE)
                                    .into(),
                            )));
                        },
                    );
                }
                Content::PlainText(editor_state) => {
                    let copilot_visible = self.copilot.visible;
                    let copilot_width = if copilot_visible {
                        crate::consts::COPILOT_PANEL_WIDTH
                    } else {
                        0.0
                    };
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                        let editor_width = (ui.available_width() - copilot_width).max(0.0);
                        ui.allocate_ui(egui::Vec2::new(editor_width, ui.available_height()), |ui| {
                            Self::build_editor(&mut self.next_content, editor_state, copilot_visible, ui);
                        });
                        if copilot_visible {
                            ui.allocate_ui(egui::Vec2::new(copilot_width, ui.available_height()), |ui| {
                                let selected_text = Some(editor_state.selected_text());
                                let reload_error = editor_state.copilot_reload_error();
                                let current_passage: &str = editor_state.plaintext()
                                    .content_of_passage(editor_state.selected_index())
                                    .map(|s| s.as_str())
                                    .unwrap_or("");
                                let (output_to_insert, needs_save_ai) = copilot::build_copilot_panel(
                                    &mut self.copilot,
                                    &self.config,
                                    selected_text,
                                    editor_state.plaintext(),
                                    current_passage,
                                    reload_error,
                                    ui,
                                );
                                if let Some(output) = output_to_insert {
                                    editor_state.set_text_to_insert(output);
                                }
                                if needs_save_ai {
                                    self.copilot.save_to_plaintext(editor_state.plaintext_mut());
                                    editor_state.mark_dirty();
                                }
                            });
                        }
                    });
                    if editor_state.save_with_copilot() {
                        Self::save(editor_state);
                        if let Err(err) = self.copilot.reload_from_plaintext(editor_state.plaintext()) {
                            editor_state.set_copilot_reload_error(Some(err));
                        } else {
                            editor_state.set_copilot_reload_error(None);
                        }
                        editor_state.set_save_with_copilot(false);
                    }
                    if editor_state.save_and_lock_with_copilot() {
                        Self::save_and_lock(&mut self.next_content, editor_state);
                        self.copilot.clear();
                        editor_state.set_save_and_lock_with_copilot(false);
                    }
                }
                Content::Error(err) => {
                    ui.with_layout(
                        egui::Layout::centered_and_justified(egui::Direction::TopDown),
                        |ui| {
                            ui.add(egui::Label::new(egui::WidgetText::RichText(
                                RichText::from(err).size(18.0).color(Color32::RED).into(),
                            )));
                        },
                    );
                }
                Content::Success(err) => {
                    ui.with_layout(
                        egui::Layout::centered_and_justified(egui::Direction::TopDown),
                        |ui| {
                            ui.add(egui::Label::new(egui::WidgetText::RichText(
                                RichText::from(err).size(18.0).color(Color32::GREEN).into(),
                            )));
                        },
                    );
                }
            }
        })
    }
}
