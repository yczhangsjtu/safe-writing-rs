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

pub struct AppState {
    pub config: Mutex<Config>,
    pub current_session: Mutex<Option<EditorSession>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            config: Mutex::new(crate::config::load_or_create_config()),
            current_session: Mutex::new(None),
        }
    }
}