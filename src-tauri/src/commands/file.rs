use crate::config::ensure_data_dir;
use crate::data_structures::PlainText;
use crate::state::AppState;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub is_encrypted: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct OpenFileResult {
    pub filename: String,
    pub is_new: bool,
    pub ciphertext: Option<String>,
}

#[tauri::command]
pub fn list_files(state: tauri::State<'_, AppState>) -> Result<Vec<String>, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let data_dir = ensure_data_dir(&config.data_dir)?;

    let files: Vec<String> = std::fs::read_dir(&data_dir)
        .map_err(|e| format!("Failed to read data directory: {}", e))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let filename = entry.file_name().to_string_lossy().to_string();
            if filename.ends_with(".safe") {
                Some(filename[..filename.len() - 5].to_string())
            } else {
                None
            }
        })
        .collect();

    Ok(files)
}

#[tauri::command]
pub fn create_file(
    filename: String,
    password: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let data_dir = ensure_data_dir(&config.data_dir)?;

    let file_path = data_dir.join(format!("{}.safe", filename));
    if file_path.exists() {
        return Err(format!("File '{}' already exists", filename));
    }

    let plaintext = PlainText::empty();
    let ciphertext = plaintext.encrypt(&password);
    std::fs::write(&file_path, ciphertext).map_err(|e| format!("Failed to create file: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn open_file(
    filename: String,
    state: tauri::State<'_, AppState>,
) -> Result<OpenFileResult, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let data_dir = ensure_data_dir(&config.data_dir)?;

    let file_path = data_dir.join(format!("{}.safe", filename));
    if !file_path.exists() {
        return Err(format!("File '{}' does not exist", filename));
    }

    let content = std::fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    if content.is_empty() {
        Ok(OpenFileResult {
            filename,
            is_new: true,
            ciphertext: None,
        })
    } else {
        Ok(OpenFileResult {
            filename,
            is_new: false,
            ciphertext: Some(content),
        })
    }
}

#[tauri::command]
pub fn delete_file(filename: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let data_dir = ensure_data_dir(&config.data_dir)?;

    let file_path = data_dir.join(format!("{}.safe", filename));
    if !file_path.exists() {
        return Err(format!("File '{}' does not exist", filename));
    }

    std::fs::remove_file(&file_path).map_err(|e| format!("Failed to delete file: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn refresh_files(state: tauri::State<'_, AppState>) -> Result<Vec<String>, String> {
    list_files(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_unique_temp_dir(test_name: &str) -> PathBuf {
        let temp_dir = std::env::temp_dir().join(format!("safe_writing_test_{}", test_name));
        if temp_dir.exists() {
            std::fs::remove_dir_all(&temp_dir).unwrap_or_default();
        }
        std::fs::create_dir_all(&temp_dir).unwrap();
        temp_dir
    }

    #[test]
    fn test_list_files_empty() {
        let temp_dir = create_unique_temp_dir("empty");
        let files: Vec<String> = std::fs::read_dir(&temp_dir)
            .unwrap()
            .filter_map(|e| {
                let name = e.unwrap().file_name().to_string_lossy().to_string();
                if name.ends_with(".safe") {
                    Some(name[..name.len() - 5].to_string())
                } else {
                    None
                }
            })
            .collect();
        assert!(files.is_empty(), "Expected empty file list, got: {:?}", files);
        std::fs::remove_dir_all(&temp_dir).unwrap_or_default();
    }

    #[test]
    fn test_list_files_with_files() {
        let temp_dir = create_unique_temp_dir("with_files");
        let file1 = temp_dir.join("test1.safe");
        let file2 = temp_dir.join("test2.safe");
        std::fs::write(&file1, "").unwrap();
        std::fs::write(&file2, "").unwrap();

        let files: Vec<String> = std::fs::read_dir(&temp_dir)
            .unwrap()
            .filter_map(|e| {
                let name = e.unwrap().file_name().to_string_lossy().to_string();
                if name.ends_with(".safe") {
                    Some(name[..name.len() - 5].to_string())
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(files.len(), 2);
        std::fs::remove_dir_all(&temp_dir).unwrap_or_default();
    }

    #[test]
    fn test_create_and_open_file() {
        let temp_dir = create_unique_temp_dir("create_open");
        let filename = "test_file";
        let password = "test_password";

        let plaintext = PlainText::empty();
        let ciphertext = plaintext.encrypt(password);
        let file_path = temp_dir.join(format!("{}.safe", filename));
        std::fs::write(&file_path, ciphertext).unwrap();

        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(!content.is_empty());
        std::fs::remove_dir_all(&temp_dir).unwrap_or_default();
    }

    #[test]
    fn test_delete_file() {
        let temp_dir = create_unique_temp_dir("delete");
        let filename = "to_delete";
        let file_path = temp_dir.join(format!("{}.safe", filename));
        std::fs::write(&file_path, "content").unwrap();

        assert!(file_path.exists());
        std::fs::remove_file(&file_path).unwrap();
        assert!(!file_path.exists());
        std::fs::remove_dir_all(&temp_dir).unwrap_or_default();
    }
}