#![cfg(target_os = "macos")]

use super::MyApp;
use egui::FontDefinitions;
use font_kit::source::SystemSource;

impl MyApp {
    pub(super) fn load_font_and_insert(
        name: &'static str,
        id: &'static str,
        index: usize,
        fonts: &mut FontDefinitions,
    ) {
        SystemSource::new()
            .all_families()
            .unwrap()
            .iter()
            .for_each(|name| println!("Family: {}", name));
        let font = SystemSource::new()
            .select_by_postscript_name(name.into())
            .expect(&format!("Cannot find font {}", name))
            .load()
            .unwrap()
            .copy_font_data()
            .unwrap();
        fonts
            .font_data
            .insert(id.to_owned(), egui::FontData::from_owned(font.to_vec()));

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
