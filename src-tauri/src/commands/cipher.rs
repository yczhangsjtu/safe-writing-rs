use std::path::PathBuf;

use crate::config::ensure_data_dir;
use crate::data_structures::PlainText;
use crate::safe_note::load_safe_note_file;
use crate::state::{AppState, EditorSession};
use crate::commands::state::emit_state_change;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct DecryptResult {
    pub passages: Vec<crate::data_structures::Passage>,
    pub num_images: usize,
    pub filename: String,
}

#[tauri::command]
pub fn decrypt_file(
    app: tauri::AppHandle,
    filename: String,
    ciphertext: String,
    password: String,
    state: tauri::State<'_, AppState>,
) -> Result<DecryptResult, String> {
    // If ciphertext is empty, read from file
    let actual_ciphertext = if ciphertext.is_empty() {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        let data_dir = crate::config::ensure_data_dir(&config.data_dir)?;
        drop(config);

        let file_path = data_dir.join(format!("{}.safe", filename));
        if !file_path.exists() {
            return Err(format!("File '{}' does not exist", filename));
        }
        std::fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to read file: {}", e))?
    } else {
        ciphertext
    };

    let plaintext = PlainText::decrypt(&password, &actual_ciphertext)
        .map_err(|e| format!("Decryption failed: {:?}", e))?;

    let session = EditorSession::new(filename.clone(), plaintext.clone(), password);
    {
        let mut current_session = state.current_session.lock().map_err(|e| e.to_string())?;
        *current_session = Some(session);
    }

    emit_state_change(&app, &state);

    Ok(DecryptResult {
        passages: plaintext.passages().clone(),
        num_images: plaintext.num_images(),
        filename,
    })
}

#[tauri::command]
pub fn encrypt_and_save(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let data_dir = ensure_data_dir(&config.data_dir)?;
    drop(config); // Release config lock

    let mut current_session = state.current_session.lock().map_err(|e| e.to_string())?;

    if let Some(session) = current_session.as_mut() {
        // Sync copilot settings to .ai passage before saving
        session.sync_copilot_to_ai_passage();

        let ciphertext = session.plaintext.encrypt(&session.password);
        let file_path = data_dir.join(format!("{}.safe", session.filename));
        std::fs::write(&file_path, ciphertext)
            .map_err(|e| format!("Failed to save file: {}", e))?;
        session.dirty = false;
        drop(current_session);
        emit_state_change(&app, &state);
    } else {
        return Err("No file is currently open".to_string());
    }

    Ok(())
}

#[tauri::command]
pub fn change_password(
    app: tauri::AppHandle,
    old_password: String,
    new_password: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let data_dir = ensure_data_dir(&config.data_dir)?;
    drop(config); // Release config lock

    let mut current_session = state.current_session.lock().map_err(|e| e.to_string())?;

    if let Some(session) = current_session.as_mut() {
        if session.password != old_password {
            return Err("Old password is incorrect".to_string());
        }
        session.password = new_password;
        let ciphertext = session.plaintext.encrypt(&session.password);
        let file_path = data_dir.join(format!("{}.safe", session.filename));
        std::fs::write(&file_path, ciphertext)
            .map_err(|e| format!("Failed to save file: {}", e))?;
        session.dirty = false;
        drop(current_session);
        emit_state_change(&app, &state);
    } else {
        return Err("No file is currently open".to_string());
    }

    Ok(())
}

#[tauri::command]
pub fn close_file(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut current_session = state.current_session.lock().map_err(|e| e.to_string())?;
    *current_session = None;
    drop(current_session);
    emit_state_change(&app, &state);
    Ok(())
}

#[tauri::command]
pub fn import_safe_notes(
    app: tauri::AppHandle,
    file_path: String,
    password: String,
    new_filename: String,
    state: tauri::State<'_, AppState>,
) -> Result<DecryptResult, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let data_dir = ensure_data_dir(&config.data_dir)?;
    drop(config); // Release config lock

    let path = PathBuf::from(&file_path);
    let safe_note = load_safe_note_file(&password, &path)
        .map_err(|e| format!("Failed to load Safe Notes file: {:?}", e))?;

    let plaintext = safe_note.into_plaintext();
    let ciphertext = plaintext.encrypt(&password);

    let save_path = data_dir.join(format!("{}.safe", new_filename));
    if save_path.exists() {
        return Err(format!("File '{}' already exists", new_filename));
    }
    std::fs::write(&save_path, ciphertext)
        .map_err(|e| format!("Failed to save imported file: {}", e))?;

    let session = EditorSession::new(new_filename.clone(), plaintext.clone(), password);
    {
        let mut current_session = state.current_session.lock().map_err(|e| e.to_string())?;
        *current_session = Some(session);
    }

    emit_state_change(&app, &state);

    Ok(DecryptResult {
        passages: plaintext.passages().clone(),
        num_images: plaintext.num_images(),
        filename: new_filename,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_structures::Passage;

    #[test]
    fn test_encrypt_decrypt_roundtrip_cmd() {
        let plaintext = PlainText::from_passages(vec![
            Passage::new(0, "Title".to_string(), "Content".to_string()),
        ]);
        let password = "test_password";
        let ciphertext = plaintext.encrypt(password);
        let decrypted = PlainText::decrypt(password, &ciphertext).unwrap();
        assert_eq!(plaintext.num_passages(), decrypted.num_passages());
    }

    #[test]
    fn test_change_password_logic() {
        let plaintext = PlainText::from_passages(vec![
            Passage::new(0, "Title".to_string(), "Content".to_string()),
        ]);
        let old_password = "old_pass";
        let new_password = "new_pass";

        let ciphertext_old = plaintext.encrypt(old_password);
        let ciphertext_new = plaintext.encrypt(new_password);

        assert_ne!(ciphertext_old, ciphertext_new);

        let decrypted_with_new = PlainText::decrypt(new_password, &ciphertext_new).unwrap();
        assert_eq!(decrypted_with_new.num_passages(), 1);
    }
}