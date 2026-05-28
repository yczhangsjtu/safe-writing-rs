use std::collections::HashMap;
use std::sync::Mutex;
use sha2::Digest;

use crate::config::Config;
use crate::data_structures::PlainText;
use crate::commands::copilot::CopilotSettings;
use crate::agent::{Workspace, Session};

pub struct EditorSession {
    pub filename: String,
    pub plaintext: PlainText,
    pub password: String,
    pub current_passage_index: usize,
    pub dirty: bool,
    pub image_digests: HashMap<String, usize>,
    pub copilot_settings: CopilotSettings, // Persistent copilot state
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
        // Load copilot settings from .ai passage if exists
        let copilot_settings = CopilotSettings::load_from_plaintext(&plaintext);
        Self {
            filename,
            plaintext,
            password,
            current_passage_index: 0,
            dirty: false,
            image_digests,
            copilot_settings,
        }
    }

    /// Update .ai passage with current copilot settings
    pub fn sync_copilot_to_ai_passage(&mut self) {
        self.copilot_settings.save_to_plaintext(&mut self.plaintext);
    }
}

/// Response for get_app_state command - contains all frontend state
#[derive(serde::Serialize, Clone)]
pub struct AppStateResponse {
    pub current_file: Option<String>,
    pub passages: Vec<crate::data_structures::Passage>,
    pub current_passage_index: usize,
    pub is_dirty: bool,
    pub num_images: usize,
    pub copilot_settings: CopilotSettings,
    pub workspace: Workspace,
    pub session: Session,
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
                num_images: s.plaintext.num_images(),
                copilot_settings: s.copilot_settings.clone(),
                workspace: s.plaintext.workspace.clone(),
                session: s.plaintext.session.clone(),
            },
            None => AppStateResponse {
                current_file: None,
                passages: Vec::new(),
                current_passage_index: 0,
                is_dirty: false,
                num_images: 0,
                copilot_settings: CopilotSettings::default(),
                workspace: Workspace::default(),
                session: Session::default(),
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