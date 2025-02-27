use eframe::egui;
use egui::Color32;

#[derive(Debug, Clone, Copy)]
pub enum ButtonStyle {
    Normal,
    Warning,
    Danger,
}

impl ButtonStyle {
    pub fn background_color(&self) -> Color32 {
        match self {
            ButtonStyle::Normal => Color32::LIGHT_GREEN.gamma_multiply(0.3),
            ButtonStyle::Warning => Color32::LIGHT_RED.gamma_multiply(0.3),
            ButtonStyle::Danger => Color32::RED,
        }
    }

    pub fn text_color(&self) -> Color32 {
        match self {
            ButtonStyle::Normal => Color32::WHITE,
            ButtonStyle::Warning => Color32::WHITE,
            ButtonStyle::Danger => Color32::WHITE,
        }
    }
    pub fn disabled_background_color(&self) -> Color32 {
        match self {
            ButtonStyle::Normal => Color32::LIGHT_GREEN.gamma_multiply(0.2),
            ButtonStyle::Warning => Color32::LIGHT_RED.gamma_multiply(0.2),
            ButtonStyle::Danger => Color32::RED.gamma_multiply(0.2),
        }
    }

    pub fn disabled_text_color(&self) -> Color32 {
        match self {
            ButtonStyle::Normal => Color32::LIGHT_GRAY.gamma_multiply(0.3),
            ButtonStyle::Warning => Color32::LIGHT_GRAY.gamma_multiply(0.3),
            ButtonStyle::Danger => Color32::LIGHT_GRAY.gamma_multiply(0.3),
        }
    }
}
