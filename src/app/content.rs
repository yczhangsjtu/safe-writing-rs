use crate::data_structures::PlainText;

use super::build::{editor::EditorState, EncryptedFileState, NewFileState};

#[derive(Default, Clone)]
pub(super) enum Content {
    #[default]
    None,
    NewFile(NewFileState),
    Encrypted(EncryptedFileState),
    PlainText(EditorState),
    Error(String),
    Success(String),
}

impl Content {
    #[allow(unused)]
    pub fn get_plaintext(&mut self) -> Option<&PlainText> {
        match self {
            Content::PlainText(ref a) => Some(a.plaintext()),
            _ => None,
        }
    }

    pub fn get_file_name(&self) -> Option<&String> {
        match self {
            Content::Encrypted(encrypted_file_state) => Some(encrypted_file_state.filename()),
            Content::PlainText(editor_state) => Some(editor_state.filename()),
            Content::Error(_) => None,
            Content::Success(_) => None,
            Content::NewFile(new_file_state) => Some(new_file_state.filename()),
            Content::None => None,
        }
    }
}
