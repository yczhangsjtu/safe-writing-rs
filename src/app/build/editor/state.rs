use crate::{app::config::Config, data_structures::PlainText};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use eframe::egui;
use egui::{TextureHandle, TextureOptions};
use image::load_from_memory;
use sha2::Digest;

#[derive(Default, Clone)]
pub struct EditorState {
    pub(super) filename: String,
    pub(super) plaintext: PlainText,
    pub(super) image_map: HashMap<String, (usize, TextureHandle)>,
    pub(super) selected_index: usize,
    pub(super) dirty: bool,
    pub(super) add_new_passage: Option<(String, usize)>,
    pub(super) editing_passage_name: Option<(String, usize)>,
    pub(super) confirm_delete_passage: Option<usize>,
    pub(super) confirm_clean_nonexist_images: bool,
    pub(super) appending_another_file: Option<(String, String)>,
    pub(super) error_appending_another_file: Option<String>,
    pub(super) preview_mode: bool,
    pub(super) password: String,
    pub(super) config: Config,
    pub(super) text_to_insert: Option<String>,
    pub(super) image_to_insert: Option<Vec<u8>>,
    pub(super) inserting_safe_image: Option<(String, String, String)>,
    pub(super) error_inserting_safe_image: Option<String>,
    pub(super) show_png_meta_data: Option<usize>,
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

    pub(super) fn insert_image(
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

    pub(super) fn clean_non_referenced_images(editor_state: &mut EditorState, ctx: &egui::Context) {
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
