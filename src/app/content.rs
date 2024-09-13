use crate::data_structures::PlainText;

use super::build::editor::EditorState;

#[derive(Default, Clone)]
pub(super) enum Content {
    #[default]
    None,
    NewFile(String),
    Encrypted(String, String, String, String),
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

    pub fn get_plaintext_mut(&mut self) -> Option<&mut PlainText> {
        match self {
            Content::PlainText(ref mut a) => Some(a.plaintext_mut()),
            _ => None,
        }
    }

    pub fn decrease_selected_index(&mut self) {
        match self {
            Content::PlainText(plaintext) => plaintext.decrease_selected_index(),
            _ => {}
        }
    }

    pub fn increase_selected_index(&mut self) {
        match self {
            Content::PlainText(plaintext) => plaintext.increase_selected_index(),
            _ => {}
        }
    }

    pub fn get_file_name(&self) -> Option<&String> {
        match self {
            Content::Encrypted(filename, _, _, _) => Some(filename),
            Content::PlainText(editor_state) => Some(editor_state.filename()),
            Content::Error(_) => None,
            Content::Success(_) => None,
            Content::NewFile(filename) => Some(filename),
            Content::None => None,
        }
    }
}
