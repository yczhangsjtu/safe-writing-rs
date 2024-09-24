use super::{content::Content, MyApp};
use eframe::egui;
use egui::{Color32, InnerResponse, RichText, Vec2, WidgetText};

const FILE_LIST_WIDTH: f32 = 200.0;
const PASSWORD_SCREEN_TOP_SPACE: f32 = 200.0;
const INFO_TEXT_SIZE: f32 = 18.0;

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
            RichText::new(self.data_dir().as_str()).color(Color32::WHITE),
        ));
        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
            self.build_file_list(FILE_LIST_WIDTH, ctx, ui);
            match &mut self.content {
                Content::NewFile(ref mut new_file_state) => {
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        ui.allocate_space(Vec2::new(0.0, PASSWORD_SCREEN_TOP_SPACE));
                        self.next_content = Self::build_uninitialized_file(new_file_state, ctx, ui);
                    });
                }
                Content::Encrypted(ref mut encrypted_file_state) => {
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        ui.allocate_space(Vec2::new(0.0, PASSWORD_SCREEN_TOP_SPACE));
                        self.next_content =
                            Self::build_encrypted_file(encrypted_file_state, ctx, ui);
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
                Content::PlainText(editor_state) => {
                    Self::build_editor(&mut self.next_content, editor_state, ui);
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
}
