use std::collections::HashMap;
use std::sync::Mutex;
use sha2::Digest;

use crate::config::Config;
use crate::data_structures::PlainText;

pub struct EditorSession {
    pub filename: String,
    pub plaintext: PlainText,
    pub password: String,
    pub current_passage_index: usize,
    pub dirty: bool,
    pub image_digests: HashMap<String, usize>,
}

impl EditorSession {
    pub fn new(filename: String, plaintext: PlainText, password: String) -> Self {
        let mut image_digests = HashMap::new();
        for (i, image) in plaintext.images().iter().enumerate() {
            let digest = format!("{:x}", {
                let mut hasher = sha2::Sha256::new();
                sha2::Digest::update(&mut hasher, image);
                sha2::Digest::finalize(hasher)
            });
            image_digests.insert(digest, i);
        }
        Self {
            filename,
            plaintext,
            password,
            current_passage_index: 0,
            dirty: false,
            image_digests,
        }
    }
}

/// Response for get_app_state command - contains all frontend state
#[derive(serde::Serialize, Clone)]
pub struct AppStateResponse {
    pub current_file: Option<String>,
    pub passages: Vec<crate::data_structures::Passage>,
    pub current_passage_index: usize,
    pub is_dirty: bool,
}

pub struct AppState {
    pub config: Mutex<Config>,
    pub current_session: Mutex<Option<EditorSession>>,
}

impl AppState {
    /// Get current state for frontend
    pub fn get_state(&self) -> AppStateResponse {
        let session = self.current_session.lock().unwrap();
        match session.as_ref() {
            Some(s) => AppStateResponse {
                current_file: Some(s.filename.clone()),
                passages: s.plaintext.passages().clone(),
                current_passage_index: s.current_passage_index,
                is_dirty: s.dirty,
            },
            None => AppStateResponse {
                current_file: None,
                passages: Vec::new(),
                current_passage_index: 0,
                is_dirty: false,
            },
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            config: Mutex::new(crate::config::load_or_create_config()),
            current_session: Mutex::new(None),
        }
    }
}