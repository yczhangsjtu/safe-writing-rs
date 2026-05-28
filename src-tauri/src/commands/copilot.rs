use crate::state::AppState;
use tauri::Emitter;

pub const AI_PASSAGE_NAME: &str = ".ai";

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    pub display: String,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct CopilotSettings {
    // Core writing guidance
    pub background: String,      // 故事背景
    pub settings: String,        // 设定
    pub notes: String,           // 注意事项
    pub requirements: String,    // 要求
    pub writing_style: String,   // 写作风格

    // User customizable system prompt (base template)
    pub custom_system_prompt: String,

    // Whether to use default prompt or custom
    pub use_custom_prompt: bool,

    // Custom skills that can override or extend built-in skills
    pub custom_skills: Vec<crate::agent::skills::CustomSkill>,
}

impl Default for CopilotSettings {
    fn default() -> Self {
        Self {
            background: String::new(),
            settings: String::new(),
            notes: String::new(),
            requirements: String::new(),
            writing_style: String::new(),
            custom_system_prompt: String::new(),
            use_custom_prompt: false,
            custom_skills: Vec::new(),
        }
    }
}

/// Get the default system prompt template
pub fn get_default_system_prompt() -> String {
    r#"You are a professional writing assistant. Your role is to help create high-quality content while respecting established context and guidelines.

Follow these principles:
1. Consistency - Maintain established characters, settings, and plot elements
2. Quality - Write engaging, polished prose
3. Respect - Honor the author's vision and preferences
4. Tools - Use available tools proactively to understand context before writing"#.to_string()
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn unescape_xml(s: &str) -> String {
    s.replace("&gt;", ">")
        .replace("&lt;", "<")
        .replace("&amp;", "&")
}

fn extract_tag(xml: &str, tag: &str) -> Option<String> {
    let start_tag = format!("<{}>", tag);
    let end_tag = format!("</{}>", tag);
    let start = xml.find(&start_tag)? + start_tag.len();
    let end = xml[start..].find(&end_tag)?;
    Some(xml[start..start + end].to_string())
}

impl CopilotSettings {
    pub fn to_xml(&self) -> String {
        let mut xml = String::new();
        xml.push_str("<ai_settings>\n");

        // Writing guidance fields
        xml.push_str(&format!(
            "  <background>{}</background>\n",
            escape_xml(&self.background)
        ));
        xml.push_str(&format!(
            "  <settings>{}</settings>\n",
            escape_xml(&self.settings)
        ));
        xml.push_str(&format!(
            "  <notes>{}</notes>\n",
            escape_xml(&self.notes)
        ));
        xml.push_str(&format!(
            "  <requirements>{}</requirements>\n",
            escape_xml(&self.requirements)
        ));
        xml.push_str(&format!(
            "  <writing_style>{}</writing_style>\n",
            escape_xml(&self.writing_style)
        ));

        // Custom prompt settings
        xml.push_str(&format!(
            "  <use_custom_prompt>{}</use_custom_prompt>\n",
            self.use_custom_prompt
        ));
        xml.push_str(&format!(
            "  <custom_system_prompt>{}</custom_system_prompt>\n",
            escape_xml(&self.custom_system_prompt)
        ));

        xml.push_str("</ai_settings>");
        xml
    }

    pub fn from_xml(xml: &str) -> Option<Self> {
        Some(Self {
            background: extract_tag(xml, "background").map(|s| unescape_xml(&s)).unwrap_or_default(),
            settings: extract_tag(xml, "settings").map(|s| unescape_xml(&s)).unwrap_or_default(),
            notes: extract_tag(xml, "notes").map(|s| unescape_xml(&s)).unwrap_or_default(),
            requirements: extract_tag(xml, "requirements").map(|s| unescape_xml(&s)).unwrap_or_default(),
            writing_style: extract_tag(xml, "writing_style").map(|s| unescape_xml(&s)).unwrap_or_default(),
            use_custom_prompt: extract_tag(xml, "use_custom_prompt")
                .map(|s| s == "true")
                .unwrap_or(false),
            custom_system_prompt: extract_tag(xml, "custom_system_prompt")
                .map(|s| unescape_xml(&s))
                .unwrap_or_default(),
            custom_skills: Vec::new(), // Not stored in XML, always empty on load
        })
    }

    pub fn load_from_plaintext(plaintext: &crate::data_structures::PlainText) -> Self {
        for passage in plaintext.passages() {
            if passage.title == AI_PASSAGE_NAME {
                if let Some(settings) = Self::from_xml(&passage.content) {
                    return settings;
                }
            }
        }
        Self::default()
    }

    pub fn save_to_plaintext(&self, plaintext: &mut crate::data_structures::PlainText) {
        let xml = self.to_xml();
        let ai_passage_index = plaintext
            .passages()
            .iter()
            .position(|p| p.title == AI_PASSAGE_NAME);

        if let Some(index) = ai_passage_index {
            plaintext.set_content(index, xml);
        } else {
            plaintext.insert_new_passage(plaintext.num_passages(), AI_PASSAGE_NAME.to_string());
            plaintext.set_content(plaintext.num_passages() - 1, xml);
        }
    }

    /// Reset to default values
    pub fn reset_to_default(&mut self) {
        *self = Self::default();
    }
}

#[derive(serde::Serialize)]
pub struct StreamEvent {
    pub event_type: String,
    pub content: String,
}

#[tauri::command]
pub async fn send_message(
    app: tauri::AppHandle,
    prompt: String,
    current_passage: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    // Get config and settings - release locks before async
    let (url, settings, workspace) = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        let url = format!("{}/v1/chat/completions", config.llamacpp_url);
        drop(config);

        let session = state.current_session.lock().map_err(|e| e.to_string())?;
        let settings = session.as_ref()
            .map(|s| s.copilot_settings.clone())
            .unwrap_or_default();
        let workspace = session.as_ref()
            .map(|s| s.plaintext.workspace.clone())
            .unwrap_or_default();
        (url, settings, workspace)
    };

    // Build system prompt using settings and workspace
    let system_prompt = crate::agent::parser::build_agent_system_prompt(&workspace, &settings);

    // Build messages
    let mut api_messages = vec![serde_json::json!({
        "role": "system",
        "content": system_prompt
    })];

    // Add current passage context
    api_messages.push(serde_json::json!({
        "role": "system",
        "content": format!("Current passage content:\n{}", current_passage)
    }));

    // Add user message
    let user_message = serde_json::json!({
        "role": "user",
        "content": prompt.clone()
    });
    api_messages.push(user_message.clone());

    // Emit user message to frontend
    app.emit("copilot-user-message", &serde_json::json!({
        "role": "user",
        "content": prompt.clone(),
        "display": prompt.clone()
    })).map_err(|e| e.to_string())?;

    let body = serde_json::json!({
        "model": "local",
        "messages": api_messages,
        "stream": true,
    });

    // Send request
    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .send()
        .await
        .map_err(|e| format!("Failed to connect to LLM server: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body_text = response.text().await.unwrap_or_default();
        return Err(format!("LLM server error ({}): {}", status, body_text));
    }

    // Emit start event
    app.emit("copilot-start", "").map_err(|e| e.to_string())?;

    // Stream response
    use futures_util::StreamExt;
    let mut stream = response.bytes_stream();
    let mut full_response = String::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Stream error: {}", e))?;
        let text = String::from_utf8_lossy(&chunk);

        for line in text.lines() {
            if line.is_empty() || !line.starts_with("data: ") {
                continue;
            }

            let data = &line[6..];
            if data == "[DONE]" {
                break;
            }

            if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                if let Some(content) = json["choices"][0]["delta"]["content"].as_str() {
                    full_response.push_str(content);
                    app.emit("copilot-chunk", content).map_err(|e| e.to_string())?;
                }
            }
        }
    }

    // Emit done event
    app.emit("copilot-done", &serde_json::json!({
        "role": "assistant",
        "content": full_response.clone(),
        "display": full_response
    })).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn abort_generation() -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub fn reset_copilot_settings(state: tauri::State<'_, AppState>) -> Result<CopilotSettings, String> {
    let mut session = state.current_session.lock().map_err(|e| e.to_string())?;
    match session.as_mut() {
        Some(s) => {
            s.copilot_settings.reset_to_default();
            s.dirty = true;
            Ok(s.copilot_settings.clone())
        }
        None => Ok(CopilotSettings::default()),
    }
}
#[tauri::command]
pub fn update_copilot_settings(
    app: tauri::AppHandle,
    settings: CopilotSettings,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut current_session = state.current_session.lock().map_err(|e| e.to_string())?;

    match current_session.as_mut() {
        Some(session) => {
            session.copilot_settings = settings;
            session.dirty = true;
            drop(current_session);
            crate::commands::state::emit_state_change(&app, &state);
            Ok(())
        }
        None => Err("No file is currently open".to_string()),
    }
}

#[tauri::command]
pub fn save_ai_settings(
    app: tauri::AppHandle,
    settings: CopilotSettings,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut current_session = state.current_session.lock().map_err(|e| e.to_string())?;

    match current_session.as_mut() {
        Some(session) => {
            session.copilot_settings = settings;
            session.sync_copilot_to_ai_passage();
            session.dirty = true;
            drop(current_session);
            crate::commands::state::emit_state_change(&app, &state);
            Ok(())
        }
        None => Err("No file is currently open".to_string()),
    }
}

#[tauri::command]
pub fn load_ai_settings(state: tauri::State<'_, AppState>) -> Result<CopilotSettings, String> {
    let current_session = state.current_session.lock().map_err(|e| e.to_string())?;

    match current_session.as_ref() {
        Some(session) => Ok(session.copilot_settings.clone()),
        None => Err("No file is currently open".to_string()),
    }
}

#[tauri::command]
pub fn clear_copilot() -> Result<CopilotSettings, String> {
    Ok(CopilotSettings::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_structures::PlainText;

    #[test]
    fn test_copilot_settings_default() {
        let settings = CopilotSettings::default();
        assert!(settings.background.is_empty());
        assert!(settings.settings.is_empty());
        assert!(settings.notes.is_empty());
        assert!(settings.requirements.is_empty());
        assert!(settings.writing_style.is_empty());
        assert!(settings.custom_system_prompt.is_empty());
        assert!(!settings.use_custom_prompt);
    }

    #[test]
    fn test_copilot_settings_xml_roundtrip() {
        let settings = CopilotSettings {
            background: "Fantasy world".to_string(),
            settings: "Magic system".to_string(),
            notes: "Keep consistent".to_string(),
            requirements: "Write 500 words".to_string(),
            writing_style: "Descriptive".to_string(),
            custom_system_prompt: "Custom prompt".to_string(),
            use_custom_prompt: true,
            custom_skills: Vec::new(),
        };
        let xml = settings.to_xml();
        let parsed = CopilotSettings::from_xml(&xml).unwrap();
        assert_eq!(parsed.background, "Fantasy world");
        assert_eq!(parsed.settings, "Magic system");
        assert_eq!(parsed.notes, "Keep consistent");
        assert_eq!(parsed.requirements, "Write 500 words");
        assert_eq!(parsed.writing_style, "Descriptive");
        assert_eq!(parsed.custom_system_prompt, "Custom prompt");
        assert!(parsed.use_custom_prompt);
    }

    #[test]
    fn test_save_load_from_plaintext() {
        let mut plaintext = PlainText::empty();
        let settings = CopilotSettings {
            background: "My background".to_string(),
            settings: String::new(),
            notes: String::new(),
            requirements: String::new(),
            writing_style: String::new(),
            custom_system_prompt: String::new(),
            use_custom_prompt: false,
            custom_skills: Vec::new(),
        };
        settings.save_to_plaintext(&mut plaintext);
        assert!(plaintext.passages().iter().any(|p| p.title == AI_PASSAGE_NAME));

        let loaded = CopilotSettings::load_from_plaintext(&plaintext);
        assert_eq!(loaded.background, "My background");
    }

    #[test]
    fn test_escape_unescape_xml() {
        let original = "Hello <world> & friends";
        let escaped = escape_xml(original);
        assert_eq!(escaped, "Hello &lt;world&gt; &amp; friends");
        let unescaped = unescape_xml(&escaped);
        assert_eq!(unescaped, original);
    }

    #[test]
    fn test_reset_to_default() {
        let mut settings = CopilotSettings {
            background: "Some background".to_string(),
            settings: "Some settings".to_string(),
            notes: "Some notes".to_string(),
            requirements: "Some requirements".to_string(),
            writing_style: "Some style".to_string(),
            custom_system_prompt: "Custom".to_string(),
            use_custom_prompt: true,
            custom_skills: Vec::new(),
        };
        settings.reset_to_default();
        assert!(settings.background.is_empty());
        assert!(!settings.use_custom_prompt);
    }
}