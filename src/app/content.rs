use crate::data_structures::PlainText;

#[derive(Default, Clone)]
pub(super) enum Content {
    #[default]
    None,
    NewFile(String),
    Encrypted(String, String, String, String),
    PlainText(String, PlainText, usize),
    Error(String),
    Success(String),
}

impl Content {
    #[allow(unused)]
    pub fn get_plaintext(&mut self) -> Option<&PlainText> {
        match self {
            Content::PlainText(_, ref a, _) => Some(a),
            _ => None,
        }
    }

    pub fn get_plaintext_mut(&mut self) -> Option<&mut PlainText> {
        match self {
            Content::PlainText(_, ref mut a, _) => Some(a),
            _ => None,
        }
    }

    pub fn decrease_selected_index(&mut self) {
        match self {
            Content::PlainText(_, _, index) => {
                *index = (*index).saturating_sub(1);
            }
            _ => {}
        }
    }

    pub fn increase_selected_index(&mut self) {
        match self {
            Content::PlainText(_, _, index) => {
                *index = (*index).saturating_add(1);
            }
            _ => {}
        }
    }

    pub fn get_file_name(&self) -> Option<&String> {
        match self {
            Content::Encrypted(filename, _, _, _) => Some(filename),
            Content::PlainText(filename, _, _) => Some(filename),
            Content::Error(_) => None,
            Content::Success(_) => None,
            Content::NewFile(filename) => Some(filename),
            Content::None => None,
        }
    }
}
