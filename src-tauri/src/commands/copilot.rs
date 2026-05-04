use std::io::BufRead;
use std::sync::mpsc::{channel, Receiver, Sender};

use crate::config::Config;
use crate::data_structures::PlainText;
use crate::state::AppState;

pub const AI_PASSAGE_NAME: &str = ".ai";

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    pub display: String,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct FavoritePrompt {
    pub name: String,
    pub prompt: String,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct CopilotSettings {
    pub system_prompt: String,
    pub buffers: Vec<String>,
    pub favorite_prompts: Vec<FavoritePrompt>,
    pub messages: Vec<Message>,
}

impl Default for CopilotSettings {
    fn default() -> Self {
        Self {
            system_prompt: "You are a helpful writing assistant.".to_string(),
            buffers: vec![String::new(); 10],
            favorite_prompts: Vec::new(),
            messages: Vec::new(),
        }
    }
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

fn extract_favorite_prompts(xml: &str) -> Option<Vec<(String, String)>> {
    let favs_section = extract_tag(xml, "favorite_prompts")?;
    let mut result = Vec::new();
    let mut remaining = favs_section.as_str();
    while let Some(prompt_start) = remaining.find("<prompt>") {
        let after_prompt_start = &remaining[prompt_start + 8..];
        if let Some(prompt_end) = after_prompt_start.find("</prompt>") {
            let prompt_content = &after_prompt_start[..prompt_end];
            if let (Some(name), Some(content)) = (
                extract_tag(prompt_content, "name"),
                extract_tag(prompt_content, "content"),
            ) {
                result.push((name, content));
            }
            remaining = &after_prompt_start[prompt_end + 9..];
        } else {
            break;
        }
    }
    Some(result)
}

fn extract_buffers(xml: &str) -> Option<Vec<String>> {
    let buffers_section = extract_tag(xml, "buffers")?;
    let mut buffers = vec![String::new(); 10];
    let mut remaining = buffers_section.as_str();
    while let Some(buffer_start) = remaining.find("<buffer") {
        let after_buffer_start = &remaining[buffer_start..];
        if let Some(gt_pos) = after_buffer_start.find('>') {
            let attr_part = &after_buffer_start[7..gt_pos];
            if let Some(index_str) = attr_part
                .strip_prefix(" index=\"")
                .and_then(|s| s.strip_suffix("\""))
            {
                if let Ok(index) = index_str.parse::<usize>() {
                    let content_start = gt_pos + 1;
                    if let Some(end_tag) = after_buffer_start.find("</buffer>") {
                        let content = &after_buffer_start[content_start..end_tag];
                        if index < 10 {
                            buffers[index] = unescape_xml(content);
                        }
                        remaining = &after_buffer_start[end_tag + 9..];
                        continue;
                    }
                }
            }
        }
        break;
    }
    Some(buffers)
}

impl CopilotSettings {
    pub fn to_xml(&self) -> String {
        let mut xml = String::new();
        xml.push_str("<ai_settings>\n");
        xml.push_str(&format!(
            "  <system_prompt>{}</system_prompt>\n",
            escape_xml(&self.system_prompt)
        ));
        xml.push_str("  <favorite_prompts>\n");
        for fav in &self.favorite_prompts {
            xml.push_str("    <prompt>\n");
            xml.push_str(&format!("      <name>{}</name>\n", escape_xml(&fav.name)));
            xml.push_str(&format!(
                "      <content>{}</content>\n",
                escape_xml(&fav.prompt)
            ));
            xml.push_str("    </prompt>\n");
        }
        xml.push_str("  </favorite_prompts>\n");
        xml.push_str("  <buffers>\n");
        for (i, buf) in self.buffers.iter().enumerate().take(10) {
            if !buf.is_empty() {
                xml.push_str(&format!(
                    "    <buffer index=\"{}\">{}</buffer>\n",
                    i,
                    escape_xml(buf)
                ));
            }
        }
        xml.push_str("  </buffers>\n");
        xml.push_str("</ai_settings>");
        xml
    }

    pub fn from_xml(xml: &str) -> Option<Self> {
        let system_prompt = extract_tag(xml, "system_prompt")?;
        let favorite_prompts = extract_favorite_prompts(xml)?;
        let buffers = extract_buffers(xml).unwrap_or_else(|| vec![String::new(); 10]);
        Some(Self {
            system_prompt: unescape_xml(&system_prompt),
            buffers,
            favorite_prompts: favorite_prompts
                .into_iter()
                .map(|(name, prompt)| FavoritePrompt {
                    name: unescape_xml(&name),
                    prompt: unescape_xml(&prompt),
                })
                .collect(),
            messages: Vec::new(),
        })
    }

    pub fn load_from_plaintext(plaintext: &PlainText) -> Self {
        for passage in plaintext.passages() {
            if passage.title == AI_PASSAGE_NAME {
                if let Some(settings) = Self::from_xml(&passage.content) {
                    return settings;
                }
            }
        }
        Self::default()
    }

    pub fn save_to_plaintext(&self, plaintext: &mut PlainText) {
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

    fn build_prompt(prompt: &str, buffers: &[String], current_passage: &str) -> String {
        let buffer_names = [
            "first", "second", "third", "fourth", "fifth", "sixth", "seventh", "eighth", "ninth",
            "tenth",
        ];

        let mut result = String::new();
        let mut referenced_names: Vec<(&str, usize)> = Vec::new();
        let mut include_all = false;

        if prompt.contains("<all>") {
            include_all = true;
        }

        for (i, _) in buffers.iter().enumerate().take(10) {
            let id = format!("#{}", i);
            if prompt.contains(&id) {
                referenced_names.push((buffer_names[i], i));
            }
        }

        let mut processed_prompt = prompt.to_string();
        for (name, i) in &referenced_names {
            let id = format!("#{}", i);
            let replacement = format!("<{}>", name);
            processed_prompt = processed_prompt.replace(&id, &replacement);
        }

        result.push_str(&processed_prompt);

        if !referenced_names.is_empty() || include_all {
            result.push_str("\n\n<referenced_buffers>\n");
            for (name, i) in &referenced_names {
                result.push_str(&format!("<{}>\n", name));
                result.push_str(&buffers[*i]);
                result.push_str(&format!("\n</{}>\n", name));
            }
            if include_all {
                result.push_str("<all>\n");
                result.push_str(current_passage);
                result.push_str("\n</all>\n");
            }
            result.push_str("</referenced_buffers>\n");
        }

        result
    }
}

pub struct StreamState {
    pub receiver: Option<Receiver<String>>,
    pub abort_sender: Option<Sender<()>>,
    pub waiting: bool,
    pub output: String,
}

impl Default for StreamState {
    fn default() -> Self {
        Self {
            receiver: None,
            abort_sender: None,
            waiting: false,
            output: String::new(),
        }
    }
}

#[tauri::command]
pub fn send_message(
    prompt: String,
    current_passage: String,
    buffers: Vec<String>,
    system_prompt: String,
    messages: Vec<Message>,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;

    let xml_content = CopilotSettings::build_prompt(&prompt, &buffers, &current_passage);

    let buffer_instructions = "\n\n<buffer_instructions>\nThe user may reference text buffers using placeholders like <first>, <second>, etc. When such placeholders appear, the actual buffer content will be provided in a <referenced_buffers> section within the user's message.\n</buffer_instructions>";

    let mut api_messages = vec![serde_json::json!({
        "role": "system",
        "content": format!("{}{}", system_prompt, buffer_instructions),
    })];

    for msg in &messages {
        api_messages.push(serde_json::json!({
            "role": msg.role,
            "content": msg.content,
        }));
    }

    api_messages.push(serde_json::json!({
        "role": "user",
        "content": xml_content,
    }));

    let body = serde_json::json!({
        "model": "local",
        "messages": api_messages,
        "stream": true,
    });

    let url = format!("{}/v1/chat/completions", config.llamacpp_url);

    Ok(())
}

#[tauri::command]
pub fn abort_generation() -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub fn save_ai_settings(
    settings: CopilotSettings,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut current_session = state.current_session.lock().map_err(|e| e.to_string())?;

    match current_session.as_mut() {
        Some(session) => {
            settings.save_to_plaintext(&mut session.plaintext);
            session.dirty = true;
            Ok(())
        }
        None => Err("No file is currently open".to_string()),
    }
}

#[tauri::command]
pub fn load_ai_settings(state: tauri::State<'_, AppState>) -> Result<CopilotSettings, String> {
    let current_session = state.current_session.lock().map_err(|e| e.to_string())?;

    match current_session.as_ref() {
        Some(session) => Ok(CopilotSettings::load_from_plaintext(&session.plaintext)),
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

    #[test]
    fn test_copilot_settings_default() {
        let settings = CopilotSettings::default();
        assert_eq!(settings.system_prompt, "You are a helpful writing assistant.");
        assert_eq!(settings.buffers.len(), 10);
        assert!(settings.favorite_prompts.is_empty());
    }

    #[test]
    fn test_copilot_settings_xml_roundtrip() {
        let settings = CopilotSettings {
            system_prompt: "Test prompt".to_string(),
            buffers: vec!["buffer content".to_string(), String::new(), String::new(), String::new(), String::new(), String::new(), String::new(), String::new(), String::new(), String::new()],
            favorite_prompts: vec![FavoritePrompt {
                name: "fav1".to_string(),
                prompt: "prompt content".to_string(),
            }],
            messages: vec![],
        };
        let xml = settings.to_xml();
        let parsed = CopilotSettings::from_xml(&xml).unwrap();
        assert_eq!(parsed.system_prompt, "Test prompt");
        assert_eq!(parsed.buffers[0], "buffer content");
        assert_eq!(parsed.favorite_prompts.len(), 1);
    }

    #[test]
    fn test_copilot_build_prompt_simple() {
        let prompt = "Hello world";
        let buffers = vec![String::new(); 10];
        let passage = "current passage content";
        let result = CopilotSettings::build_prompt(prompt, &buffers, passage);
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_copilot_build_prompt_with_buffer() {
        let prompt = "#0 summarize this";
        let buffers = vec!["test buffer".to_string(), String::new(), String::new(), String::new(), String::new(), String::new(), String::new(), String::new(), String::new(), String::new()];
        let passage = "current passage";
        let result = CopilotSettings::build_prompt(prompt, &buffers, passage);
        assert!(result.contains("<first>"));
        assert!(result.contains("test buffer"));
    }

    #[test]
    fn test_copilot_build_prompt_with_all() {
        let prompt = "<all> summarize all";
        let buffers = vec![String::new(); 10];
        let passage = "current passage content";
        let result = CopilotSettings::build_prompt(prompt, &buffers, passage);
        assert!(result.contains("<all>"));
        assert!(result.contains("current passage content"));
    }

    #[test]
    fn test_save_load_from_plaintext() {
        let mut plaintext = PlainText::empty();
        let settings = CopilotSettings {
            system_prompt: "My system prompt".to_string(),
            buffers: vec![String::new(); 10],
            favorite_prompts: vec![],
            messages: vec![],
        };
        settings.save_to_plaintext(&mut plaintext);
        assert!(plaintext.passages().iter().any(|p| p.title == AI_PASSAGE_NAME));

        let loaded = CopilotSettings::load_from_plaintext(&plaintext);
        assert_eq!(loaded.system_prompt, "My system prompt");
    }

    #[test]
    fn test_escape_unescape_xml() {
        let original = "Hello <world> & friends";
        let escaped = escape_xml(original);
        assert_eq!(escaped, "Hello &lt;world&gt; &amp; friends");
        let unescaped = unescape_xml(&escaped);
        assert_eq!(unescaped, original);
    }
}