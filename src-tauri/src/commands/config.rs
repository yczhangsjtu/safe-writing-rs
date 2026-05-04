use crate::config::{load_or_create_config, save_config, Config};

#[tauri::command]
pub fn get_config(state: tauri::State<'_, crate::state::AppState>) -> Result<Config, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

#[tauri::command]
pub fn update_config(
    font_size: Option<f32>,
    theme: Option<String>,
    llamacpp_url: Option<String>,
    state: tauri::State<'_, crate::state::AppState>,
) -> Result<Config, String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;

    if let Some(size) = font_size {
        config.font_size = size;
    }
    if let Some(t) = theme {
        config.theme = t;
    }
    if let Some(url) = llamacpp_url {
        config.llamacpp_url = url;
    }

    save_config(&config)?;
    Ok(config.clone())
}

#[tauri::command]
pub fn get_data_dir(state: tauri::State<'_, crate::state::AppState>) -> Result<String, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    Ok(config.data_dir.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_update() {
        let mut config = Config::default();
        config.font_size = 20.0;
        config.theme = "light".to_string();
        assert_eq!(config.font_size, 20.0);
        assert_eq!(config.theme, "light");
    }
}