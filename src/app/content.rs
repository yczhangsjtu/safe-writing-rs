#[derive(Default, Clone)]
pub(super) enum Content<T: Clone, P: Clone> {
    #[default]
    None,
    NewFile(T),
    Encrypted(T, T, T, T),
    PlainText(T, P, usize),
    Error(T),
    Success(T),
}

impl<T: Clone, P: Clone> Content<T, P> {
    #[allow(unused)]
    pub fn as_ref(&self) -> Content<&T, &P> {
        match self {
            Content::Encrypted(ref title, ref a, ref b, ref c) => {
                Content::Encrypted(title, a, b, c)
            }
            Content::PlainText(ref title, ref a, index) => Content::PlainText(title, a, *index),
            Content::Error(ref a) => Content::Error(a),
            Content::Success(ref a) => Content::Success(a),
            Content::None => Content::None,
            Content::NewFile(ref title) => Content::NewFile(title),
        }
    }

    #[allow(unused)]
    pub fn get_plaintext(&mut self) -> Option<&P> {
        match self {
            Content::PlainText(_, ref a, _) => Some(a),
            _ => None,
        }
    }

    pub fn get_plaintext_mut(&mut self) -> Option<&mut P> {
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

    pub fn get_file_name(&self) -> Option<&T> {
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
