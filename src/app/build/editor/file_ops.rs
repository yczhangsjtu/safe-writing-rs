use super::state::EditorState;
use super::{super::locked::EncryptedFileState, MyApp};
use crate::app::content::Content;

use eframe::egui;

impl MyApp {
    pub(super) fn try_appending_safe_file(
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
    pub(super) fn save(editor_state: &mut EditorState) {
        // EditorState::clean_non_referenced_images(editor_state, ctx);
        let path = editor_state.full_path();
        std::fs::write(path, editor_state.plaintext.encrypt(&editor_state.password)).unwrap();
        editor_state.dirty = false;
    }

    pub(super) fn save_and_lock(
        next_content: &mut Option<Content>,
        editor_state: &mut EditorState,
    ) {
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
