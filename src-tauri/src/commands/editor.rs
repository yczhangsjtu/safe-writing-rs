use crate::config::ensure_data_dir;
use crate::data_structures::{Passage, PlainText};
use crate::state::AppState;
use sha2::Digest;
use crate::commands::state::emit_state_change;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ImageInfo {
    pub digest: String,
    pub index: usize,
    pub data: String, // base64 encoded image data
}

#[tauri::command]
pub fn get_passages(state: tauri::State<'_, AppState>) -> Result<Vec<Passage>, String> {
    let current_session = state.current_session.lock().map_err(|e| e.to_string())?;
    match current_session.as_ref() {
        Some(session) => Ok(session.plaintext.passages().clone()),
        None => Err("No file is currently open".to_string()),
    }
}

#[tauri::command]
pub fn update_passage_content(
    app: tauri::AppHandle,
    index: usize,
    content: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut current_session = state.current_session.lock().map_err(|e| e.to_string())?;
    match current_session.as_mut() {
        Some(session) => {
            if index < session.plaintext.num_passages() {
                session.plaintext.set_content(index, content);
                session.dirty = true;
                // Release lock before emitting
                drop(current_session);
                emit_state_change(&app, &state);
                Ok(())
            } else {
                Err(format!("Passage index {} out of bounds", index))
            }
        }
        None => Err("No file is currently open".to_string()),
    }
}

#[tauri::command]
pub fn update_passage_title(
    app: tauri::AppHandle,
    index: usize,
    title: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut current_session = state.current_session.lock().map_err(|e| e.to_string())?;
    match current_session.as_mut() {
        Some(session) => {
            if index < session.plaintext.num_passages() {
                session.plaintext.set_title(index, title);
                session.dirty = true;
                drop(current_session);
                emit_state_change(&app, &state);
                Ok(())
            } else {
                Err(format!("Passage index {} out of bounds", index))
            }
        }
        None => Err("No file is currently open".to_string()),
    }
}

#[tauri::command]
pub fn add_passage(
    app: tauri::AppHandle,
    title: String,
    state: tauri::State<'_, AppState>,
) -> Result<usize, String> {
    let mut current_session = state.current_session.lock().map_err(|e| e.to_string())?;
    match current_session.as_mut() {
        Some(session) => {
            // If there are no passages, insert at index 0
            // Otherwise, insert after current passage
            let index = if session.plaintext.num_passages() == 0 {
                0
            } else {
                session.current_passage_index + 1
            };
            session.plaintext.insert_new_passage(index, title);
            session.dirty = true;
            drop(current_session);
            emit_state_change(&app, &state);
            Ok(index)
        }
        None => Err("No file is currently open".to_string()),
    }
}

#[tauri::command]
pub fn remove_passage(
    app: tauri::AppHandle,
    index: usize,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut current_session = state.current_session.lock().map_err(|e| e.to_string())?;
    match current_session.as_mut() {
        Some(session) => {
            if index < session.plaintext.num_passages() {
                session.plaintext.remove_passage(index);
                session.dirty = true;
                if session.current_passage_index >= session.plaintext.num_passages() {
                    session.current_passage_index = session.plaintext.bounded_index(session.current_passage_index);
                }
                drop(current_session);
                emit_state_change(&app, &state);
                Ok(())
            } else {
                Err(format!("Passage index {} out of bounds", index))
            }
        }
        None => Err("No file is currently open".to_string()),
    }
}

#[tauri::command]
pub fn move_passage(
    app: tauri::AppHandle,
    from: usize,
    to: usize,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut current_session = state.current_session.lock().map_err(|e| e.to_string())?;
    match current_session.as_mut() {
        Some(session) => {
            let num = session.plaintext.num_passages();
            // Allow to == num to mean "insert at the end"
            if from < num && to <= num && from != to {
                // Remove from original position
                let passage = session.plaintext.remove_passage(from);

                // Calculate insert position
                let insert_index = if to == num {
                    // Insert at the end
                    num - 1
                } else if from < to {
                    to - 1  // After removal, items between shifted down by 1
                } else {
                    to
                };

                // Insert at new position
                session.plaintext.content.insert(insert_index, passage);
                session.dirty = true;

                // Update current_passage_index to track the moved passage
                if session.current_passage_index == from {
                    session.current_passage_index = insert_index;
                } else if from < session.current_passage_index && session.current_passage_index <= to.min(num - 1) {
                    // Current passage was between from and to, shift it
                    session.current_passage_index -= 1;
                } else if to <= session.current_passage_index && session.current_passage_index < from {
                    // Current passage was between to and from, shift it
                    session.current_passage_index += 1;
                }

                drop(current_session);
                emit_state_change(&app, &state);
                Ok(())
            } else if from == to || (to == num && from == num - 1) {
                Ok(()) // No movement needed
            } else {
                Err("Passage index out of bounds".to_string())
            }
        }
        None => Err("No file is currently open".to_string()),
    }
}

#[tauri::command]
pub fn insert_image(
    app: tauri::AppHandle,
    image_data: Vec<u8>,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let mut current_session = state.current_session.lock().map_err(|e| e.to_string())?;
    match current_session.as_mut() {
        Some(session) => {
            let digest = format!("{:x}", {
                let mut hasher = sha2::Sha256::new();
                hasher.update(&image_data);
                hasher.finalize()
            });

            if session.image_digests.contains_key(&digest) {
                return Ok(digest);
            }

            session.plaintext.images_mut().push(image_data);
            let index = session.plaintext.num_images() - 1;
            session.image_digests.insert(digest.clone(), index);
            session.dirty = true;

            drop(current_session);
            emit_state_change(&app, &state);
            Ok(digest)
        }
        None => Err("No file is currently open".to_string()),
    }
}

#[tauri::command]
pub fn get_images(state: tauri::State<'_, AppState>) -> Result<Vec<ImageInfo>, String> {
    let current_session = state.current_session.lock().map_err(|e| e.to_string())?;
    match current_session.as_ref() {
        Some(session) => {
            let images: Vec<ImageInfo> = session
                .image_digests
                .iter()
                .map(|(digest, index)| {
                    let image_data = session.plaintext.images().get(*index);
                    let base64_data = image_data
                        .map(|data| base64::Engine::encode(&base64::engine::general_purpose::STANDARD, data))
                        .unwrap_or_default();
                    ImageInfo {
                        digest: digest.clone(),
                        index: *index,
                        data: base64_data,
                    }
                })
                .collect();
            Ok(images)
        }
        None => Err("No file is currently open".to_string()),
    }
}

#[tauri::command]
pub fn get_image_metadata(index: usize, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let current_session = state.current_session.lock().map_err(|e| e.to_string())?;
    match current_session.as_ref() {
        Some(session) => {
            if index < session.plaintext.num_images() {
                let image_data = &session.plaintext.images()[index];
                let metadata = crate::png::read_png_metadata(image_data);
                Ok(metadata.unwrap_or_else(|| "No PNG metadata found".to_string()))
            } else {
                Err(format!("Image index {} out of bounds", index))
            }
        }
        None => Err("No file is currently open".to_string()),
    }
}

#[tauri::command]
pub fn get_current_file(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let current_session = state.current_session.lock().map_err(|e| e.to_string())?;
    match current_session.as_ref() {
        Some(session) => Ok(session.filename.clone()),
        None => Err("No file is currently open".to_string()),
    }
}

#[tauri::command]
pub fn set_current_passage(
    app: tauri::AppHandle,
    index: usize,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut current_session = state.current_session.lock().map_err(|e| e.to_string())?;
    match current_session.as_mut() {
        Some(session) => {
            if index < session.plaintext.num_passages() {
                session.current_passage_index = index;
                drop(current_session);
                emit_state_change(&app, &state);
                Ok(())
            } else {
                Err(format!("Passage index {} out of bounds", index))
            }
        }
        None => Err("No file is currently open".to_string()),
    }
}

#[tauri::command]
pub fn get_current_passage_index(state: tauri::State<'_, AppState>) -> Result<usize, String> {
    let current_session = state.current_session.lock().map_err(|e| e.to_string())?;
    match current_session.as_ref() {
        Some(session) => Ok(session.current_passage_index),
        None => Err("No file is currently open".to_string()),
    }
}

#[tauri::command]
pub fn is_dirty(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    let current_session = state.current_session.lock().map_err(|e| e.to_string())?;
    match current_session.as_ref() {
        Some(session) => Ok(session.dirty),
        None => Err("No file is currently open".to_string()),
    }
}

#[tauri::command]
pub fn append_file(
    app: tauri::AppHandle,
    filename: String,
    password: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let data_dir = ensure_data_dir(&config.data_dir)?;
    drop(config); // Release config lock

    let mut current_session = state.current_session.lock().map_err(|e| e.to_string())?;

    match current_session.as_mut() {
        Some(session) => {
            if session.filename == filename {
                return Err("Cannot append to self".to_string());
            }

            let file_path = data_dir.join(format!("{}.safe", filename));
            if !file_path.exists() {
                return Err(format!("File '{}' does not exist", filename));
            }

            let content = std::fs::read_to_string(&file_path)
                .map_err(|e| format!("Failed to read file: {}", e))?;

            if content.is_empty() {
                return Err(format!("File '{}' is empty", filename));
            }

            let other_plaintext = PlainText::decrypt(&password, &content)
                .map_err(|e| format!("Failed to decrypt file: {:?}", e))?;

            for image in other_plaintext.images() {
                let digest = format!("{:x}", {
                    let mut hasher = sha2::Sha256::new();
                    hasher.update(image);
                    hasher.finalize()
                });
                if !session.image_digests.contains_key(&digest) {
                    session.plaintext.images_mut().push(image.clone());
                    session.image_digests.insert(digest, session.plaintext.num_images() - 1);
                }
            }

            session.plaintext.append_plaintext(&other_plaintext);
            session.dirty = true;

            drop(current_session);
            emit_state_change(&app, &state);
            Ok(())
        }
        None => Err("No file is currently open".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_structures::Passage;

    fn create_test_session() -> EditorSession {
        let plaintext = PlainText::from_passages(vec![
            Passage::new(0, "Title 1".to_string(), "Content 1".to_string()),
            Passage::new(1, "Title 2".to_string(), "Content 2".to_string()),
        ]);
        EditorSession::new("test_file".to_string(), plaintext, "password".to_string())
    }

    #[test]
    fn test_update_passage_content() {
        let mut session = create_test_session();
        session.plaintext.set_content(0, "New Content".to_string());
        session.dirty = true;  // Mark as dirty since we're directly modifying
        assert_eq!(session.plaintext.content_of_passage(0), Some(&"New Content".to_string()));
        assert!(session.dirty);
    }

    #[test]
    fn test_add_passage() {
        let mut session = create_test_session();
        session.current_passage_index = 0;
        session.plaintext.insert_new_passage(1, "New Title".to_string());
        assert_eq!(session.plaintext.num_passages(), 3);
        assert_eq!(session.plaintext.title_of_passage(1), Some("New Title".to_string()));
    }

    #[test]
    fn test_remove_passage() {
        let mut session = create_test_session();
        session.plaintext.remove_passage(0);
        assert_eq!(session.plaintext.num_passages(), 1);
        assert_eq!(session.plaintext.title_of_passage(0), Some("Title 2".to_string()));
    }

    #[test]
    fn test_move_passage() {
        let mut session = create_test_session();
        session.plaintext.swap(0, 1);
        assert_eq!(session.plaintext.title_of_passage(0), Some("Title 2".to_string()));
        assert_eq!(session.plaintext.title_of_passage(1), Some("Title 1".to_string()));
    }

    #[test]
    fn test_insert_image() {
        let mut session = EditorSession::new(
            "test".to_string(),
            PlainText::empty(),
            "password".to_string(),
        );
        let image_data = vec![1, 2, 3, 4, 5];
        let digest = format!("{:x}", {
            let mut hasher = sha2::Sha256::new();
            hasher.update(&image_data);
            hasher.finalize()
        });
        session.plaintext.images_mut().push(image_data.clone());
        session.image_digests.insert(digest.clone(), 0);
        assert_eq!(session.plaintext.num_images(), 1);
        assert!(session.image_digests.contains_key(&digest));
    }
}