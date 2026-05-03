#![cfg(target_os = "windows")]

use super::MyApp;
use egui::FontDefinitions;
use font_kit::{family_name::FamilyName, properties::Properties, source::SystemSource};

impl MyApp {
    pub(super) fn load_font_and_insert(
        family_name: &'static str,
        id: &'static str,
        index: usize,
        fonts: &mut FontDefinitions,
    ) {
        let font = SystemSource::new()
            .select_best_match(
                &[FamilyName::Title(family_name.to_string())],
                &Properties::new(),
            )
            .expect(&format!("Cannot find font family {}", family_name))
            .load()
            .unwrap()
            .copy_font_data()
            .unwrap();
        fonts
            .font_data
            .insert(id.to_owned(), std::sync::Arc::new(egui::FontData::from_owned(font.to_vec())));

        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(index, id.to_owned());
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .push(id.to_owned());
    }
}
