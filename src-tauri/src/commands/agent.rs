use crate::agent::{
    Workspace, Session,
    workspace::{Character, Relationship, KeyValueEntry},
    session::{SessionMessage, ToolCall},
    tools::ToolDefinition,
    parser::ParsedToolCall,
    executor::ToolResult,
};
use crate::state::AppState;
use crate::commands::state::emit_state_change;
use tauri::Emitter;

// ========== Workspace Commands ==========

#[tauri::command]
pub fn get_workspace(state: tauri::State<'_, AppState>) -> Result<Workspace, String> {
    let session = state.current_session.lock().map_err(|e| e.to_string())?;
    match session.as_ref() {
        Some(s) => Ok(s.plaintext.workspace.clone()),
        None => Err("No file open".to_string()),
    }
}

#[tauri::command]
pub fn get_characters(state: tauri::State<'_, AppState>) -> Result<Vec<Character>, String> {
    let session = state.current_session.lock().map_err(|e| e.to_string())?;
    match session.as_ref() {
        Some(s) => Ok(s.plaintext.workspace.characters.clone()),
        None => Err("No file open".to_string()),
    }
}

#[tauri::command]
pub fn add_character(
    app: tauri::AppHandle,
    name: String,
    description: String,
    state: tauri::State<'_, AppState>,
) -> Result<Character, String> {
    let mut session = state.current_session.lock().map_err(|e| e.to_string())?;
    match session.as_mut() {
        Some(s) => {
            let character = s.plaintext.workspace.add_character(name, description);
            s.dirty = true;
            drop(session);
            emit_state_change(&app, &state);
            Ok(character)
        }
        None => Err("No file open".to_string()),
    }
}

#[tauri::command]
pub fn update_character(
    app: tauri::AppHandle,
    character_id: usize,
    name: Option<String>,
    description: Option<String>,
    aliases: Option<Vec<String>>,
    traits: Option<Vec<String>>,
    notes: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<Character, String> {
    let mut session = state.current_session.lock().map_err(|e| e.to_string())?;
    match session.as_mut() {
        Some(s) => {
            let character = s.plaintext.workspace.update_character(
                character_id, name, description, aliases, traits, notes
            )?;
            s.dirty = true;
            drop(session);
            emit_state_change(&app, &state);
            Ok(character)
        }
        None => Err("No file open".to_string()),
    }
}

#[tauri::command]
pub fn remove_character(
    app: tauri::AppHandle,
    character_id: usize,
    state: tauri::State<'_, AppState>,
) -> Result<Character, String> {
    let mut session = state.current_session.lock().map_err(|e| e.to_string())?;
    match session.as_mut() {
        Some(s) => {
            let character = s.plaintext.workspace.remove_character(character_id)?;
            s.dirty = true;
            drop(session);
            emit_state_change(&app, &state);
            Ok(character)
        }
        None => Err("No file open".to_string()),
    }
}

#[tauri::command]
pub fn get_relationships(state: tauri::State<'_, AppState>) -> Result<Vec<Relationship>, String> {
    let session = state.current_session.lock().map_err(|e| e.to_string())?;
    match session.as_ref() {
        Some(s) => Ok(s.plaintext.workspace.relationships.clone()),
        None => Err("No file open".to_string()),
    }
}

#[tauri::command]
pub fn add_relationship(
    app: tauri::AppHandle,
    character_a_id: usize,
    character_b_id: usize,
    relationship_type: String,
    description: String,
    state: tauri::State<'_, AppState>,
) -> Result<Relationship, String> {
    let mut session = state.current_session.lock().map_err(|e| e.to_string())?;
    match session.as_mut() {
        Some(s) => {
            let relationship = s.plaintext.workspace.add_relationship(
                character_a_id, character_b_id, relationship_type, description
            )?;
            s.dirty = true;
            drop(session);
            emit_state_change(&app, &state);
            Ok(relationship)
        }
        None => Err("No file open".to_string()),
    }
}

#[tauri::command]
pub fn update_relationship(
    app: tauri::AppHandle,
    relationship_id: usize,
    relationship_type: Option<String>,
    description: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<Relationship, String> {
    let mut session = state.current_session.lock().map_err(|e| e.to_string())?;
    match session.as_mut() {
        Some(s) => {
            let relationship = s.plaintext.workspace.update_relationship(
                relationship_id, relationship_type, description
            )?;
            s.dirty = true;
            drop(session);
            emit_state_change(&app, &state);
            Ok(relationship)
        }
        None => Err("No file open".to_string()),
    }
}

#[tauri::command]
pub fn remove_relationship(
    app: tauri::AppHandle,
    relationship_id: usize,
    state: tauri::State<'_, AppState>,
) -> Result<Relationship, String> {
    let mut session = state.current_session.lock().map_err(|e| e.to_string())?;
    match session.as_mut() {
        Some(s) => {
            let relationship = s.plaintext.workspace.remove_relationship(relationship_id)?;
            s.dirty = true;
            drop(session);
            emit_state_change(&app, &state);
            Ok(relationship)
        }
        None => Err("No file open".to_string()),
    }
}

#[tauri::command]
pub fn get_kv_store(state: tauri::State<'_, AppState>) -> Result<Vec<KeyValueEntry>, String> {
    let session = state.current_session.lock().map_err(|e| e.to_string())?;
    match session.as_ref() {
        Some(s) => Ok(s.plaintext.workspace.key_value_store.clone()),
        None => Err("No file open".to_string()),
    }
}

#[tauri::command]
pub fn set_kv(
    app: tauri::AppHandle,
    key: String,
    value: String,
    category: String,
    notes: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut session = state.current_session.lock().map_err(|e| e.to_string())?;
    match session.as_mut() {
        Some(s) => {
            s.plaintext.workspace.set_kv(key, value, category, notes);
            s.dirty = true;
            drop(session);
            emit_state_change(&app, &state);
            Ok(())
        }
        None => Err("No file open".to_string()),
    }
}

#[tauri::command]
pub fn delete_kv(
    app: tauri::AppHandle,
    key: String,
    state: tauri::State<'_, AppState>,
) -> Result<KeyValueEntry, String> {
    let mut session = state.current_session.lock().map_err(|e| e.to_string())?;
    match session.as_mut() {
        Some(s) => {
            let entry = s.plaintext.workspace.delete_kv(&key)
                .ok_or_else(|| format!("Key '{}' not found", key))?;
            s.dirty = true;
            drop(session);
            emit_state_change(&app, &state);
            Ok(entry)
        }
        None => Err("No file open".to_string()),
    }
}

// ========== Session Commands ==========

#[tauri::command]
pub fn get_session(state: tauri::State<'_, AppState>) -> Result<Session, String> {
    let session = state.current_session.lock().map_err(|e| e.to_string())?;
    match session.as_ref() {
        Some(s) => Ok(s.plaintext.session.clone()),
        None => Err("No file open".to_string()),
    }
}

#[tauri::command]
pub fn edit_session_message(
    app: tauri::AppHandle,
    message_id: usize,
    new_content: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut session = state.current_session.lock().map_err(|e| e.to_string())?;
    match session.as_mut() {
        Some(s) => {
            s.plaintext.session.edit_message(message_id, new_content)?;
            s.dirty = true;
            drop(session);
            emit_state_change(&app, &state);
            Ok(())
        }
        None => Err("No file open".to_string()),
    }
}

#[tauri::command]
pub fn delete_session_message(
    app: tauri::AppHandle,
    message_id: usize,
    state: tauri::State<'_, AppState>,
) -> Result<SessionMessage, String> {
    let mut session = state.current_session.lock().map_err(|e| e.to_string())?;
    match session.as_mut() {
        Some(s) => {
            let message = s.plaintext.session.delete_message(message_id)?;
            s.dirty = true;
            drop(session);
            emit_state_change(&app, &state);
            Ok(message)
        }
        None => Err("No file open".to_string()),
    }
}

#[tauri::command]
pub fn compress_session(
    app: tauri::AppHandle,
    summary: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut session = state.current_session.lock().map_err(|e| e.to_string())?;
    match session.as_mut() {
        Some(s) => {
            s.plaintext.session.compress(summary);
            s.dirty = true;
            drop(session);
            emit_state_change(&app, &state);
            Ok(())
        }
        None => Err("No file open".to_string()),
    }
}

#[tauri::command]
pub fn clear_session(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut session = state.current_session.lock().map_err(|e| e.to_string())?;
    match session.as_mut() {
        Some(s) => {
            s.plaintext.session.clear();
            s.dirty = true;
            drop(session);
            emit_state_change(&app, &state);
            Ok(())
        }
        None => Err("No file open".to_string()),
    }
}

// ========== Agent Commands ==========

#[tauri::command]
pub fn get_tool_definitions() -> Result<Vec<ToolDefinition>, String> {
    Ok(crate::agent::tools::get_tool_definitions())
}

#[tauri::command]
pub fn confirm_tool_call(
    app: tauri::AppHandle,
    tool_call_id: String,
    confirmed: bool,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut session = state.current_session.lock().map_err(|e| e.to_string())?;
    match session.as_mut() {
        Some(s) => {
            if confirmed {
                s.plaintext.session.confirm_tool_call(&tool_call_id)?;
            } else {
                s.plaintext.session.cancel_tool_call(&tool_call_id)?;
            }
            s.dirty = true;
            drop(session);
            emit_state_change(&app, &state);
            Ok(())
        }
        None => Err("No file open".to_string()),
    }
}

#[tauri::command]
pub async fn execute_confirmed_tool(
    app: tauri::AppHandle,
    tool_call_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<ToolResult, String> {
    // Get tool call details
    let (tool_call, _current_passage_content) = {
        let session = state.current_session.lock().map_err(|e| e.to_string())?;
        match session.as_ref() {
            Some(s) => {
                // Find the tool call
                let tool_call = s.plaintext.session.messages.iter()
                    .filter_map(|m| m.tool_calls.as_ref())
                    .flat_map(|tc| tc.iter())
                    .find(|tc| tc.id == tool_call_id)
                    .cloned();

                match tool_call {
                    Some(tc) => {
                        let content = s.plaintext.content_of_passage(s.current_passage_index)
                            .cloned()
                            .unwrap_or_default();
                        (Some(tc), content)
                    }
                    None => return Err(format!("Tool call {} not found", tool_call_id)),
                }
            }
            None => return Err("No file open".to_string()),
        }
    };

    let tool_call = tool_call.ok_or_else(|| format!("Tool call {} not found", tool_call_id))?;

    // Create ParsedToolCall for execution
    let parsed = ParsedToolCall {
        id: tool_call.id.clone(),
        tool_name: tool_call.tool_name.clone(),
        arguments: tool_call.arguments.clone(),
    };

    // Execute the tool
    let result = {
        let mut session = state.current_session.lock().map_err(|e| e.to_string())?;
        match session.as_mut() {
            Some(s) => {
                let result = execute_tool_on_session(&parsed, &mut s.plaintext, &mut s.copilot_settings);

                // Mark tool as executed
                if result.success {
                    s.plaintext.session.mark_tool_executed(&tool_call_id)?;
                    // Sync copilot settings to .ai passage if AI settings were updated
                    if parsed.tool_name == "update_ai_settings" {
                        s.sync_copilot_to_ai_passage();
                    }
                }

                s.dirty = true;
                drop(session);
                emit_state_change(&app, &state);
                result
            }
            None => return Err("No file open".to_string()),
        }
    };

    Ok(result)
}

/// Execute a tool on session (handles borrow issues)
fn execute_tool_on_session(
    tool_call: &ParsedToolCall,
    plaintext: &mut crate::data_structures::PlainText,
    copilot_settings: &mut crate::commands::copilot::CopilotSettings,
) -> ToolResult {
    execute_tool_internal(tool_call, plaintext, copilot_settings)
}

fn execute_tool_internal(
    tool_call: &ParsedToolCall,
    plaintext: &mut crate::data_structures::PlainText,
    copilot_settings: &mut crate::commands::copilot::CopilotSettings,
) -> ToolResult {
    match tool_call.tool_name.as_str() {
        // Passage reading tools
        "read_passage" => {
            let passages = plaintext.passages();
            let passage = if let Some(idx) = tool_call.arguments["passage_index"].as_u64() {
                passages.get(idx as usize)
            } else if let Some(title) = tool_call.arguments["passage_title"].as_str() {
                passages.iter().find(|p| p.title == title)
            } else {
                passages.first()
            };

            match passage {
                Some(p) => ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    success: true,
                    output: serde_json::json!({
                        "title": p.title,
                        "content": p.content.clone()
                    }).to_string(),
                    error: None,
                },
                None => ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    success: false,
                    output: String::new(),
                    error: Some("Passage not found".to_string()),
                },
            }
        }
        "list_passages" => {
            let list: Vec<serde_json::Value> = plaintext.passages().iter().enumerate().map(|(i, p)| {
                serde_json::json!({
                    "index": i,
                    "title": p.title,
                    "length": p.content.len()
                })
            }).collect();
            ToolResult {
                tool_call_id: String::new(),
                success: true,
                output: serde_json::to_string(&list).unwrap_or_default(),
                error: None,
            }
        }

        // Writing tools
        "continue_writing" => {
            let idx = tool_call.arguments["passage_index"].as_u64().unwrap_or(0) as usize;
            let text = tool_call.arguments["text"].as_str().unwrap_or("");

            if idx >= plaintext.passages().len() {
                return ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    success: false,
                    output: String::new(),
                    error: Some(format!("Invalid passage index: {}", idx)),
                };
            }

            let current = plaintext.content_of_passage(idx).cloned().unwrap_or_default();
            plaintext.set_content(idx, current + text);
            ToolResult {
                tool_call_id: tool_call.id.clone(),
                success: true,
                output: "Text appended".to_string(),
                error: None,
            }
        }
        "insert_text" => {
            let idx = tool_call.arguments["passage_index"].as_u64().unwrap_or(0) as usize;
            let position = tool_call.arguments["position"].as_u64().unwrap_or(0) as usize;
            let text = tool_call.arguments["text"].as_str().unwrap_or("");

            if idx >= plaintext.passages().len() {
                return ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    success: false,
                    output: String::new(),
                    error: Some(format!("Invalid passage index: {}", idx)),
                };
            }

            let current = plaintext.content_of_passage(idx).cloned().unwrap_or_default();
            if position > current.len() {
                return ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    success: false,
                    output: String::new(),
                    error: Some("Invalid position".to_string()),
                };
            }

            plaintext.set_content(idx, current[..position].to_string() + text + &current[position..]);
            ToolResult {
                tool_call_id: tool_call.id.clone(),
                success: true,
                output: "Text inserted".to_string(),
                error: None,
            }
        }
        "replace_text" => {
            let idx = tool_call.arguments["passage_index"].as_u64().unwrap_or(0) as usize;
            let start = tool_call.arguments["start"].as_u64().unwrap_or(0) as usize;
            let end = tool_call.arguments["end"].as_u64().unwrap_or(0) as usize;
            let new_text = tool_call.arguments["new_text"].as_str().unwrap_or("");

            if idx >= plaintext.passages().len() {
                return ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    success: false,
                    output: String::new(),
                    error: Some(format!("Invalid passage index: {}", idx)),
                };
            }

            let current = plaintext.content_of_passage(idx).cloned().unwrap_or_default();
            if start > end || end > current.len() {
                return ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    success: false,
                    output: String::new(),
                    error: Some("Invalid range".to_string()),
                };
            }

            plaintext.set_content(idx, current[..start].to_string() + new_text + &current[end..]);
            ToolResult {
                tool_call_id: tool_call.id.clone(),
                success: true,
                output: "Text replaced".to_string(),
                error: None,
            }
        }

        // Character tools
        "create_character" => {
            let name = tool_call.arguments["name"].as_str().unwrap_or("").to_string();
            let description = tool_call.arguments["description"].as_str().unwrap_or("").to_string();

            let char = plaintext.workspace.add_character(name, description);
            ToolResult {
                tool_call_id: tool_call.id.clone(),
                success: true,
                output: serde_json::to_string(&char).unwrap_or_default(),
                error: None,
            }
        }
        "update_character" => {
            let id = tool_call.arguments["character_id"].as_u64().unwrap_or(0) as usize;
            let name = tool_call.arguments["name"].as_str().map(|s| s.to_string());
            let description = tool_call.arguments["description"].as_str().map(|s| s.to_string());
            let aliases = tool_call.arguments["aliases"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect());
            let traits = tool_call.arguments["traits"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect());
            let notes = tool_call.arguments["notes"].as_str().map(|s| s.to_string());

            match plaintext.workspace.update_character(id, name, description, aliases, traits, notes) {
                Ok(c) => ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    success: true,
                    output: serde_json::to_string(&c).unwrap_or_default(),
                    error: None,
                },
                Err(e) => ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    success: false,
                    output: String::new(),
                    error: Some(e),
                },
            }
        }
        "delete_character" => {
            let id = tool_call.arguments["character_id"].as_u64().unwrap_or(0) as usize;
            match plaintext.workspace.remove_character(id) {
                Ok(_) => ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    success: true,
                    output: "Character deleted".to_string(),
                    error: None,
                },
                Err(e) => ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    success: false,
                    output: String::new(),
                    error: Some(e),
                },
            }
        }
        "list_characters" => {
            let list: Vec<serde_json::Value> = plaintext.workspace.characters.iter().map(|c| {
                serde_json::json!({
                    "id": c.id,
                    "name": c.name,
                    "description": c.description
                })
            }).collect();
            ToolResult {
                tool_call_id: String::new(),
                success: true,
                output: serde_json::to_string(&list).unwrap_or_default(),
                error: None,
            }
        }
        "get_character" => {
            let char = if let Some(id) = tool_call.arguments["character_id"].as_u64() {
                plaintext.workspace.get_character(id as usize)
            } else if let Some(name) = tool_call.arguments["character_name"].as_str() {
                plaintext.workspace.find_character_by_name(name)
            } else {
                None
            };

            match char {
                Some(c) => ToolResult {
                    tool_call_id: String::new(),
                    success: true,
                    output: serde_json::to_string(c).unwrap_or_default(),
                    error: None,
                },
                None => ToolResult {
                    tool_call_id: String::new(),
                    success: false,
                    output: String::new(),
                    error: Some("Character not found".to_string()),
                },
            }
        }

        // Relationship tools
        "create_relationship" => {
            let char_a = tool_call.arguments["character_a_id"].as_u64().unwrap_or(0) as usize;
            let char_b = tool_call.arguments["character_b_id"].as_u64().unwrap_or(0) as usize;
            let rel_type = tool_call.arguments["relationship_type"].as_str().unwrap_or("").to_string();
            let description = tool_call.arguments["description"].as_str().unwrap_or("").to_string();

            match plaintext.workspace.add_relationship(char_a, char_b, rel_type, description) {
                Ok(r) => ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    success: true,
                    output: serde_json::to_string(&r).unwrap_or_default(),
                    error: None,
                },
                Err(e) => ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    success: false,
                    output: String::new(),
                    error: Some(e),
                },
            }
        }
        "update_relationship" => {
            let id = tool_call.arguments["relationship_id"].as_u64().unwrap_or(0) as usize;
            let rel_type = tool_call.arguments["relationship_type"].as_str().map(|s| s.to_string());
            let description = tool_call.arguments["description"].as_str().map(|s| s.to_string());

            match plaintext.workspace.update_relationship(id, rel_type, description) {
                Ok(r) => ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    success: true,
                    output: serde_json::to_string(&r).unwrap_or_default(),
                    error: None,
                },
                Err(e) => ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    success: false,
                    output: String::new(),
                    error: Some(e),
                },
            }
        }
        "delete_relationship" => {
            let id = tool_call.arguments["relationship_id"].as_u64().unwrap_or(0) as usize;
            match plaintext.workspace.remove_relationship(id) {
                Ok(_) => ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    success: true,
                    output: "Relationship deleted".to_string(),
                    error: None,
                },
                Err(e) => ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    success: false,
                    output: String::new(),
                    error: Some(e),
                },
            }
        }
        "list_relationships" => {
            let list: Vec<serde_json::Value> = plaintext.workspace.relationships.iter().map(|r| {
                serde_json::json!({
                    "id": r.id,
                    "character_a_id": r.character_a_id,
                    "character_b_id": r.character_b_id,
                    "type": r.relationship_type
                })
            }).collect();
            ToolResult {
                tool_call_id: String::new(),
                success: true,
                output: serde_json::to_string(&list).unwrap_or_default(),
                error: None,
            }
        }
        "query_character_relationships" => {
            let char_id = if let Some(id) = tool_call.arguments["character_id"].as_u64() {
                id as usize
            } else if let Some(name) = tool_call.arguments["character_name"].as_str() {
                match plaintext.workspace.find_character_by_name(name) {
                    Some(c) => c.id,
                    None => {
                        return ToolResult {
                            tool_call_id: String::new(),
                            success: false,
                            output: String::new(),
                            error: Some(format!("Character '{}' not found", name)),
                        };
                    }
                }
            } else {
                return ToolResult {
                    tool_call_id: String::new(),
                    success: false,
                    output: String::new(),
                    error: Some("No character specified".to_string()),
                };
            };

            let rels = plaintext.workspace.get_character_relationships(char_id);
            let list: Vec<serde_json::Value> = rels.iter().map(|r| {
                let other_id = if r.character_a_id == char_id { r.character_b_id } else { r.character_a_id };
                serde_json::json!({
                    "id": r.id,
                    "other_character_id": other_id,
                    "type": r.relationship_type,
                    "description": r.description
                })
            }).collect();
            ToolResult {
                tool_call_id: String::new(),
                success: true,
                output: serde_json::to_string(&list).unwrap_or_default(),
                error: None,
            }
        }

        // KV store tools
        "set_kv" => {
            let key = tool_call.arguments["key"].as_str().unwrap_or("").to_string();
            let value = tool_call.arguments["value"].as_str().unwrap_or("").to_string();
            let category = tool_call.arguments["category"].as_str().unwrap_or("general").to_string();
            let notes = tool_call.arguments["notes"].as_str().unwrap_or("").to_string();

            plaintext.workspace.set_kv(key, value, category, notes);
            ToolResult {
                tool_call_id: tool_call.id.clone(),
                success: true,
                output: "Key-value set".to_string(),
                error: None,
            }
        }
        "get_kv" => {
            let key = tool_call.arguments["key"].as_str().unwrap_or("");
            match plaintext.workspace.get_kv(key) {
                Some(e) => ToolResult {
                    tool_call_id: String::new(),
                    success: true,
                    output: serde_json::to_string(e).unwrap_or_default(),
                    error: None,
                },
                None => ToolResult {
                    tool_call_id: String::new(),
                    success: false,
                    output: String::new(),
                    error: Some(format!("Key '{}' not found", key)),
                },
            }
        }
        "delete_kv" => {
            let key = tool_call.arguments["key"].as_str().unwrap_or("");
            match plaintext.workspace.delete_kv(key) {
                Some(_) => ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    success: true,
                    output: "Key deleted".to_string(),
                    error: None,
                },
                None => ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    success: false,
                    output: String::new(),
                    error: Some(format!("Key '{}' not found", key)),
                },
            }
        }
        "list_kv" => {
            let category = tool_call.arguments["category"].as_str();
            let entries = if let Some(cat) = category {
                plaintext.workspace.list_kv_by_category(cat)
            } else {
                plaintext.workspace.key_value_store.iter().collect()
            };

            let list: Vec<serde_json::Value> = entries.iter().map(|e| {
                serde_json::json!({
                    "key": e.key,
                    "value": e.value,
                    "category": e.category
                })
            }).collect();
            ToolResult {
                tool_call_id: String::new(),
                success: true,
                output: serde_json::to_string(&list).unwrap_or_default(),
                error: None,
            }
        }

        // AI Settings tools
        "get_ai_settings" => {
            ToolResult {
                tool_call_id: String::new(),
                success: true,
                output: serde_json::json!({
                    "background": copilot_settings.background,
                    "settings": copilot_settings.settings,
                    "notes": copilot_settings.notes,
                    "requirements": copilot_settings.requirements,
                    "writing_style": copilot_settings.writing_style
                }).to_string(),
                error: None,
            }
        }
        "update_ai_settings" => {
            if let Some(bg) = tool_call.arguments["background"].as_str() {
                copilot_settings.background = bg.to_string();
            }
            if let Some(s) = tool_call.arguments["settings"].as_str() {
                copilot_settings.settings = s.to_string();
            }
            if let Some(n) = tool_call.arguments["notes"].as_str() {
                copilot_settings.notes = n.to_string();
            }
            if let Some(r) = tool_call.arguments["requirements"].as_str() {
                copilot_settings.requirements = r.to_string();
            }
            if let Some(ws) = tool_call.arguments["writing_style"].as_str() {
                copilot_settings.writing_style = ws.to_string();
            }
            ToolResult {
                tool_call_id: tool_call.id.clone(),
                success: true,
                output: "AI settings updated successfully".to_string(),
                error: None,
            }
        }

        _ => ToolResult {
            tool_call_id: tool_call.id.clone(),
            success: false,
            output: String::new(),
            error: Some(format!("Unknown tool: {}", tool_call.tool_name)),
        },
    }
}

#[tauri::command]
pub fn get_pending_tool_calls(state: tauri::State<'_, AppState>) -> Result<Vec<ToolCall>, String> {
    let session = state.current_session.lock().map_err(|e| e.to_string())?;
    match session.as_ref() {
        Some(s) => {
            let pending = s.plaintext.session.get_pending_tool_calls()
                .into_iter()
                .cloned()
                .collect();
            Ok(pending)
        }
        None => Err("No file open".to_string()),
    }
}

#[tauri::command]
pub async fn send_agent_message(
    app: tauri::AppHandle,
    prompt: String,
    current_passage: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    use crate::agent::parser::{parse_tool_calls, to_session_tool_calls};
    use crate::agent::tools::get_tool_definitions;
    use futures_util::StreamExt;

    // Get config
    let url = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        format!("{}/v1/chat/completions", config.llamacpp_url)
    };

    // Add user message to session
    {
        let mut session_guard = state.current_session.lock().map_err(|e| e.to_string())?;
        let session = session_guard.as_mut().ok_or("No file open")?;
        session.plaintext.session.add_user_message(prompt.clone());
        session.dirty = true;
    }

    // Emit user message event
    app.emit("agent-user-message", &prompt).map_err(|e| e.to_string())?;

    // Build system prompt
    let (workspace, session_messages, copilot_settings) = {
        let session_guard = state.current_session.lock().map_err(|e| e.to_string())?;
        let session = session_guard.as_ref().ok_or("No file open")?;
        (
            session.plaintext.workspace.clone(),
            session.plaintext.session.messages.clone(),
            session.copilot_settings.clone(),
        )
    };

    let system_prompt = crate::agent::parser::build_agent_system_prompt(&workspace, &copilot_settings);
    let tool_defs = get_tool_definitions();
    let tools_json = serde_json::to_string_pretty(&tool_defs).unwrap_or_default();

    // Build API messages
    let mut api_messages = vec![
        serde_json::json!({
            "role": "system",
            "content": format!("{}\n\nAVAILABLE TOOLS:\n{}", system_prompt, tools_json)
        })
    ];

    // Add summary if exists
    if let Some(summary) = &session_messages.iter().find_map(|m| {
        if m.role == "system" && m.content.starts_with("Summary:") { Some(&m.content) } else { None }
    }) {
        api_messages.push(serde_json::json!({
            "role": "system",
            "content": summary
        }));
    }

    // Add session messages (excluding compressed ones)
    for msg in session_messages.iter().filter(|m| !m.compressed) {
        api_messages.push(serde_json::json!({
            "role": msg.role,
            "content": msg.content
        }));
    }

    // Add current passage context
    api_messages.push(serde_json::json!({
        "role": "system",
        "content": format!("Current passage content:\n{}", current_passage)
    }));

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
        .map_err(|e| format!("Failed to connect to LLM: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body_text = response.text().await.unwrap_or_default();
        return Err(format!("LLM error ({}): {}", status, body_text));
    }

    app.emit("agent-start", "").map_err(|e| e.to_string())?;

    // Stream response
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
                    app.emit("agent-chunk", content).map_err(|e| e.to_string())?;
                }
            }
        }
    }

    // Parse tool calls
    let (remaining_content, parsed_tool_calls) = parse_tool_calls(&full_response);
    let tool_calls = to_session_tool_calls(parsed_tool_calls);

    // Add assistant message to session
    let msg_id = {
        let mut session_guard = state.current_session.lock().map_err(|e| e.to_string())?;
        let session = session_guard.as_mut().ok_or("No file open")?;
        let msg = session.plaintext.session.add_assistant_message(remaining_content.clone(), Some(tool_calls.clone()));
        session.dirty = true;
        msg.id
    };

    // Emit done event
    app.emit("agent-done", serde_json::json!({
        "id": msg_id,
        "role": "assistant",
        "content": remaining_content
    })).map_err(|e| e.to_string())?;

    // Emit pending tool calls
    for tc in tool_calls.iter().filter(|tc| tc.status == crate::agent::session::ToolCallStatus::Pending) {
        app.emit("agent-tool-call", tc).map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Get all available skills (built-in + custom)
#[tauri::command]
pub fn get_skills(state: tauri::State<'_, AppState>) -> Result<Vec<crate::agent::skills::Skill>, String> {
    let session = state.current_session.lock().map_err(|e| e.to_string())?;
    match session.as_ref() {
        Some(s) => {
            let all_skills = crate::agent::skills::get_all_skills(&s.copilot_settings.custom_skills);
            Ok(all_skills)
        }
        None => {
            // Return built-in skills even when no file is open
            Ok(crate::agent::skills::get_builtin_skills())
        }
    }
}

/// Start a skill session - sends the skill's first message automatically
#[tauri::command]
pub async fn start_skill(
    app: tauri::AppHandle,
    skill_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    use crate::agent::parser::{parse_tool_calls, to_session_tool_calls};
    use crate::agent::tools::get_tool_definitions;
    use futures_util::StreamExt;

    // Get the skill
    let skill = {
        let session = state.current_session.lock().map_err(|e| e.to_string())?;
        let custom_skills = session.as_ref()
            .map(|s| s.copilot_settings.custom_skills.clone())
            .unwrap_or_default();
        let all_skills = crate::agent::skills::get_all_skills(&custom_skills);
        all_skills.iter().find(|s| s.id == skill_id).cloned()
            .ok_or_else(|| format!("Skill '{}' not found", skill_id))?
    };

    // Clear previous session messages
    {
        let mut session_guard = state.current_session.lock().map_err(|e| e.to_string())?;
        if let Some(session) = session_guard.as_mut() {
            session.plaintext.session.clear();
            session.dirty = true;
        }
    }

    // Emit skill started event
    app.emit("skill-started", &skill).map_err(|e| e.to_string())?;

    // Build system prompt with skill addition
    let (workspace, copilot_settings) = {
        let session_guard = state.current_session.lock().map_err(|e| e.to_string())?;
        match session_guard.as_ref() {
            Some(s) => (s.plaintext.workspace.clone(), s.copilot_settings.clone()),
            None => return Err("No file open".to_string()),
        }
    };

    let base_prompt = crate::agent::parser::build_agent_system_prompt(&workspace, &copilot_settings);
    let full_system_prompt = format!("{}\n\n{}\n\n{}", base_prompt, skill.system_prompt_addition, "You should now greet the user with your first message as defined in this skill.");

    let tool_defs = get_tool_definitions();
    let tools_json = serde_json::to_string_pretty(&tool_defs).unwrap_or_default();

    // Build API messages - we want the agent to say the first message
    let api_messages = vec![
        serde_json::json!({
            "role": "system",
            "content": format!("{}\n\nAVAILABLE TOOLS:\n{}", full_system_prompt, tools_json)
        }),
        serde_json::json!({
            "role": "user",
            "content": "[Skill activated: Please start the conversation with your greeting message.]"
        })
    ];

    // Get URL
    let url = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        format!("{}/v1/chat/completions", config.llamacpp_url)
    };

    // Send request
    let body = serde_json::json!({
        "model": "local",
        "messages": api_messages,
        "stream": true,
    });

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
    app.emit("agent-start", "").map_err(|e| e.to_string())?;

    // Stream response
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
                    app.emit("agent-chunk", content).map_err(|e| e.to_string())?;
                }
            }
        }
    }

    // Parse tool calls
    let (remaining_content, parsed_tool_calls) = parse_tool_calls(&full_response);
    let tool_calls = to_session_tool_calls(parsed_tool_calls);

    // Add assistant message to session
    let msg_id = {
        let mut session_guard = state.current_session.lock().map_err(|e| e.to_string())?;
        let session = session_guard.as_mut().ok_or("No file open")?;
        // Add the assistant's first message
        let msg = session.plaintext.session.add_assistant_message(remaining_content.clone(), Some(tool_calls.clone()));
        session.dirty = true;
        msg.id
    };

    // Emit done event
    app.emit("agent-done", serde_json::json!({
        "id": msg_id,
        "role": "assistant",
        "content": remaining_content
    })).map_err(|e| e.to_string())?;

    // Emit pending tool calls
    for tc in tool_calls.iter().filter(|tc| tc.status == crate::agent::session::ToolCallStatus::Pending) {
        app.emit("agent-tool-call", tc).map_err(|e| e.to_string())?;
    }

    Ok(())
}