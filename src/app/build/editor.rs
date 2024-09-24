use super::{locked::EncryptedFileState, MyApp};

use crate::{
    app::{config::Config, content::Content},
    data_structures::PlainText,
    png::read_png_metadata,
};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use eframe::egui;
use egui::{
    load::SizedTexture, text::CCursorRange, Color32, FontFamily, FontId, FontSelection, Image, Key,
    Label, RichText, TextBuffer, TextEdit, TextureHandle, TextureOptions, Vec2, WidgetText,
};
use image::load_from_memory;
use sha2::Digest;

#[derive(Default, Clone)]
pub struct EditorState {
    filename: String,
    plaintext: PlainText,
    image_map: HashMap<String, (usize, TextureHandle)>,
    selected_index: usize,
    dirty: bool,
    add_new_passage: Option<(String, usize)>,
    editing_passage_name: Option<(String, usize)>,
    confirm_delete_passage: Option<usize>,
    show_passage_operation_buttons: bool,
    appending_another_file: Option<(String, String)>,
    error_appending_another_file: Option<String>,
    preview_mode: bool,
    password: String,
    config: Config,
    text_to_insert: Option<String>,
    image_to_insert: Option<Vec<u8>>,
    inserting_safe_image: Option<(String, String, String)>,
    error_inserting_safe_image: Option<String>,
    show_png_meta_data: Option<usize>,
}

impl EditorState {
    pub fn new(
        filename: String,
        plaintext: PlainText,
        password: String,
        config: Config,
        ctx: &egui::Context,
    ) -> Self {
        let image_map = if plaintext.num_images() > 0 {
            Self::build_image_map(plaintext.images(), ctx)
        } else {
            HashMap::default()
        };
        EditorState {
            filename,
            plaintext,
            image_map,
            password,
            selected_index: 0,
            config,
            ..Default::default()
        }
    }

    fn load_image_from_memory(image_data: &[u8]) -> Result<egui::ColorImage, image::ImageError> {
        let image = load_from_memory(image_data)?;
        let size = [image.width() as _, image.height() as _];
        let image_buffer = image.to_rgba8();
        let pixels: image::FlatSamples<&[u8]> = image_buffer.as_flat_samples();
        Ok(egui::ColorImage::from_rgba_unmultiplied(
            size,
            pixels.as_slice(),
        ))
    }

    fn load_texture_from_memory(
        image_data: &[u8],
        ctx: &egui::Context,
    ) -> Result<TextureHandle, image::ImageError> {
        let image = Self::load_image_from_memory(image_data)?;
        let texture_handle = ctx.load_texture("image", image, TextureOptions::default());
        Ok(texture_handle)
    }

    fn build_image_map(
        images: &[Vec<u8>],
        ctx: &egui::Context,
    ) -> HashMap<String, (usize, TextureHandle)> {
        let mut image_map = HashMap::default();
        // Compute the SHA256 of this image as the key
        for (i, image) in images.iter().enumerate() {
            let digest = format!("{:x}", {
                let mut hasher = sha2::Sha256::new();
                hasher.update(image);
                hasher.finalize()
            });
            let handle = Self::load_texture_from_memory(image.as_slice(), ctx);
            if let Ok(handle) = handle {
                image_map.insert(digest, (i, handle));
            } else {
                println!("Failed to load image: {}", i);
                continue;
            }
        }
        image_map
    }

    fn insert_image(
        editor_state: &mut EditorState,
        image: &Vec<u8>,
        ctx: &egui::Context,
    ) -> String {
        let image_digest = format!("{:x}", {
            let mut hasher = sha2::Sha256::new();
            hasher.update(image);
            hasher.finalize()
        });
        if editor_state.image_map.contains_key(&image_digest) {
            return image_digest;
        }
        editor_state.plaintext.images_mut().push(image.clone());
        let handle = Self::load_texture_from_memory(image.as_slice(), ctx);
        if let Ok(handle) = handle {
            editor_state.image_map.insert(
                image_digest.clone(),
                (editor_state.plaintext.num_images() - 1, handle),
            );
        } else {
            println!("Failed to load image: {}", image_digest);
        }
        editor_state.dirty = true;
        image_digest
    }

    fn image_placeholder(digest: &str) -> String {
        format!("image!({})", digest)
    }

    fn clean_non_referenced_images(editor_state: &mut EditorState, ctx: &egui::Context) {
        // Collect all the strings of the form image!(...) in the passages
        let mut image_references = HashSet::new();
        for passage in editor_state.plaintext.passages() {
            for line in passage.content().split('\n') {
                if line.starts_with("image!(") && line.ends_with(")") && line.len() == 72 {
                    // SHA256 digest hex has length 64, so the line length is
                    // 64 + len("image!()") = 72
                    image_references.insert(&line[7..71]);
                }
            }
        }

        let existing_indices = editor_state
            .image_map
            .iter()
            .filter_map(|(digest, index)| {
                if image_references.contains(digest.as_str()) {
                    Some(index.0)
                } else {
                    None
                }
            })
            .collect::<HashSet<_>>();
        if existing_indices.len() == editor_state.image_map.len() {
            return;
        }
        let mut index = 0;
        editor_state.plaintext.images_mut().retain(|_| {
            let ret = existing_indices.contains(&index);
            index += 1;
            ret
        });
        editor_state.image_map = Self::build_image_map(&editor_state.plaintext.images(), ctx);
    }

    pub fn insert_image_at_cursor(&mut self, image: Vec<u8>, ctx: &egui::Context) {
        let digest = Self::insert_image(self, &image, ctx);
        self.text_to_insert = Some(format!("\n{}\n", Self::image_placeholder(&digest)));
    }

    pub fn empty(filename: String, password: String, config: Config) -> Self {
        EditorState {
            filename,
            plaintext: PlainText::empty(),
            password,
            selected_index: 0,
            config,
            ..Default::default()
        }
    }

    pub fn filename(&self) -> &String {
        &self.filename
    }

    pub fn full_path(&self) -> PathBuf {
        self.full_path_of(&self.filename)
    }

    pub fn full_path_of(&self, filename: &str) -> PathBuf {
        PathBuf::from(self.data_dir().clone()).join(format!("{}.safe", filename))
    }

    pub fn temp_path(&self) -> PathBuf {
        PathBuf::from(self.data_dir().clone()).join(format!("temp.txt"))
    }

    pub fn plaintext(&self) -> &PlainText {
        &self.plaintext
    }

    pub fn plaintext_mut(&mut self) -> &mut PlainText {
        &mut self.plaintext
    }

    #[allow(unused)]
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

#[derive(Debug, Clone, Copy)]
enum ButtonStyle {
    Normal,
    Warning,
    Danger,
}

impl ButtonStyle {
    fn background_color(&self) -> Color32 {
        match self {
            ButtonStyle::Normal => Color32::LIGHT_GREEN.gamma_multiply(0.3),
            ButtonStyle::Warning => Color32::LIGHT_RED.gamma_multiply(0.3),
            ButtonStyle::Danger => Color32::RED,
        }
    }

    fn text_color(&self) -> Color32 {
        match self {
            ButtonStyle::Normal => Color32::WHITE,
            ButtonStyle::Warning => Color32::WHITE,
            ButtonStyle::Danger => Color32::WHITE,
        }
    }
    fn disabled_background_color(&self) -> Color32 {
        match self {
            ButtonStyle::Normal => Color32::LIGHT_GREEN.gamma_multiply(0.2),
            ButtonStyle::Warning => Color32::LIGHT_RED.gamma_multiply(0.2),
            ButtonStyle::Danger => Color32::RED.gamma_multiply(0.2),
        }
    }

    fn disabled_text_color(&self) -> Color32 {
        match self {
            ButtonStyle::Normal => Color32::LIGHT_GRAY.gamma_multiply(0.3),
            ButtonStyle::Warning => Color32::LIGHT_GRAY.gamma_multiply(0.3),
            ButtonStyle::Danger => Color32::LIGHT_GRAY.gamma_multiply(0.3),
        }
    }
}

impl MyApp {
    fn make_control_button(caption: &str, style: ButtonStyle, disabled: bool) -> egui::Button {
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

    pub(super) fn build_editor(
        next_content: &mut Option<Content>,
        editor_state: &mut EditorState,
        ui: &mut egui::Ui,
    ) {
        egui::Frame::none()
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

    fn build_toggle_button(editor_state: &mut EditorState, width: f32, ui: &mut egui::Ui) {
        if ui
            .add(
                Self::make_control_button("...", ButtonStyle::Normal, false)
                    .min_size(Vec2::new(width, 24.0)),
            )
            .clicked()
        {
            editor_state.show_passage_operation_buttons =
                !editor_state.show_passage_operation_buttons;
        }
    }

    fn build_preview_button(editor_state: &mut EditorState, width: f32, ui: &mut egui::Ui) {
        if ui
            .add(
                Self::make_control_button(
                    if editor_state.preview_mode {
                        "Edit"
                    } else {
                        "Preview"
                    },
                    ButtonStyle::Normal,
                    false,
                )
                .min_size(Vec2::new(width, 24.0)),
            )
            .clicked()
        {
            editor_state.preview_mode = !editor_state.preview_mode;
        }
    }

    fn build_insert_image_button(editor_state: &mut EditorState, width: f32, ui: &mut egui::Ui) {
        if ui
            .add(
                Self::make_control_button("Image", ButtonStyle::Normal, editor_state.preview_mode)
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

    fn build_insert_safe_image_button(
        editor_state: &mut EditorState,
        width: f32,
        ui: &mut egui::Ui,
    ) {
        if ui
            .add(
                Self::make_control_button(
                    "Safe Image",
                    ButtonStyle::Normal,
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

    fn build_add_button(editor_state: &mut EditorState, width: f32, ui: &mut egui::Ui) {
        if ui
            .add(
                Self::make_control_button("Add", ButtonStyle::Normal, false)
                    .min_size(Vec2::new(width, 24.0)),
            )
            .clicked()
        {
            editor_state.add_new_passage =
                Some(("".to_string(), editor_state.selected_index() + 1));
            editor_state.editing_passage_name = None;
        }
    }

    fn build_save_button(editor_state: &mut EditorState, width: f32, ui: &mut egui::Ui) {
        if ui
            .add(
                Self::make_control_button("Save", ButtonStyle::Normal, !editor_state.dirty)
                    .min_size(Vec2::new(width, 24.0)),
            )
            .clicked()
            && editor_state.dirty
        {
            Self::save(editor_state, ui.ctx());
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
                Self::make_control_button("Save & Lock", ButtonStyle::Warning, false)
                    .min_size(Vec2::new(width, 24.0)),
            )
            .clicked()
        {
            Self::save_and_lock(next_content, editor_state, ui.ctx());
        }
    }

    fn build_move_button(
        editor_state: &mut EditorState,
        selected_index: usize,
        up: bool,
        width: f32,
        ui: &mut egui::Ui,
    ) {
        if ui
            .add(
                Self::make_control_button(
                    if up { "Move Up" } else { "Move Down" },
                    ButtonStyle::Normal,
                    false,
                )
                .min_size(Vec2::new(width, 24.0)),
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
                Self::make_control_button("Rename", ButtonStyle::Normal, false)
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
                Self::make_control_button("Delete", ButtonStyle::Warning, false)
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
                    ButtonStyle::Normal,
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
                Self::make_control_button("Append File", ButtonStyle::Normal, false)
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

    fn try_appending_safe_file(
        editor_state: &mut EditorState,
        filename: &String,
        password: &String,
        ui: &mut egui::Ui,
    ) {
        if filename == &editor_state.filename {
            editor_state.error_appending_another_file = Some("Cannot append to self".to_string());
            return;
        }

        let path = editor_state.full_path_of(&filename);
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
                            ui,
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

    fn try_appending_safe_file_content(
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

    fn try_inserting_safe_file_image(
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

    fn build_passage_list(
        next_content: &mut Option<Content>,
        editor_state: &mut EditorState,
        width: f32,
        ui: &mut egui::Ui,
    ) {
        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
            Self::build_toggle_button(editor_state, width, ui);
            if ui
                .ctx()
                .input(|i| i.key_pressed(Key::S) && i.modifiers.command)
            {
                Self::save(editor_state, ui.ctx());
            }
            if ui
                .ctx()
                .input(|i| i.key_pressed(Key::L) && i.modifiers.command)
            {
                Self::save_and_lock(next_content, editor_state, ui.ctx());
            }
            if editor_state.show_passage_operation_buttons {
                Self::build_preview_button(editor_state, width, ui);
                Self::build_insert_image_button(editor_state, width, ui);
                Self::build_insert_safe_image_button(editor_state, width, ui);
                Self::build_add_button(editor_state, width, ui);
                Self::build_save_button(editor_state, width, ui);
                Self::build_save_lock_button(next_content, editor_state, width, ui);
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
                Self::build_rename_button(editor_state, width, ui);
                Self::build_delete_button(editor_state, editor_state.selected_index(), width, ui);
                Self::build_read_temp_button(
                    editor_state,
                    editor_state.selected_index(),
                    width,
                    ui,
                );
                Self::build_append_file_button(editor_state, width, ui);
            }
            egui::ScrollArea::vertical()
                .id_source("passage_list")
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

    fn save(editor_state: &mut EditorState, ctx: &egui::Context) {
        EditorState::clean_non_referenced_images(editor_state, ctx);
        let path = editor_state.full_path();
        std::fs::write(path, editor_state.plaintext.encrypt(&editor_state.password)).unwrap();
        editor_state.dirty = false;
    }

    fn save_and_lock(
        next_content: &mut Option<Content>,
        editor_state: &mut EditorState,
        ctx: &egui::Context,
    ) {
        EditorState::clean_non_referenced_images(editor_state, ctx);
        let path = editor_state.full_path();
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
