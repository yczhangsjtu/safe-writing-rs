use crate::agent::parser::ParsedToolCall;
use crate::agent::workspace::Workspace;
use crate::data_structures::PlainText;

/// Result of executing a tool
#[derive(Debug, Clone, serde::Serialize)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

/// Execute a tool call
pub fn execute_tool(
    tool_call: &ParsedToolCall,
    plaintext: &mut PlainText,
    workspace: &mut Workspace,
    _session: &mut crate::agent::session::Session,
) -> ToolResult {
    match tool_call.tool_name.as_str() {
        // Passage reading tools
        "read_passage" => execute_read_passage(tool_call, plaintext),
        "list_passages" => execute_list_passages(plaintext),

        // Writing tools
        "continue_writing" => execute_continue_writing(tool_call, plaintext),
        "insert_text" => execute_insert_text(tool_call, plaintext),
        "replace_text" => execute_replace_text(tool_call, plaintext),

        // Character tools
        "create_character" => execute_create_character(tool_call, workspace),
        "update_character" => execute_update_character(tool_call, workspace),
        "delete_character" => execute_delete_character(tool_call, workspace),
        "list_characters" => execute_list_characters(workspace),
        "get_character" => execute_get_character(tool_call, workspace),

        // Relationship tools
        "create_relationship" => execute_create_relationship(tool_call, workspace),
        "update_relationship" => execute_update_relationship(tool_call, workspace),
        "delete_relationship" => execute_delete_relationship(tool_call, workspace),
        "list_relationships" => execute_list_relationships(workspace),
        "query_character_relationships" => execute_query_character_relationships(tool_call, workspace),

        // KV store tools
        "set_kv" => execute_set_kv(tool_call, workspace),
        "get_kv" => execute_get_kv(tool_call, workspace),
        "delete_kv" => execute_delete_kv(tool_call, workspace),
        "list_kv" => execute_list_kv(tool_call, workspace),

        _ => ToolResult {
            tool_call_id: tool_call.id.clone(),
            success: false,
            output: String::new(),
            error: Some(format!("Unknown tool: {}", tool_call.tool_name)),
        },
    }
}

// Passage tools

fn execute_read_passage(tool_call: &ParsedToolCall, plaintext: &PlainText) -> ToolResult {
    let passages = plaintext.passages();

    // Get passage by index or title
    let passage = if let Some(idx) = tool_call.arguments["passage_index"].as_u64() {
        let idx = idx as usize;
        passages.get(idx)
    } else if let Some(title) = tool_call.arguments["passage_title"].as_str() {
        passages.iter().find(|p| p.title == title)
    } else {
        // Default to current passage (index 0)
        passages.first()
    };

    match passage {
        Some(p) => {
            let content = p.content.clone();
            let start = tool_call.arguments["start_offset"].as_u64().unwrap_or(0) as usize;
            let len = tool_call.arguments["length"].as_u64().map(|l| l as usize);
            let content_len = content.len();

            let result = if let Some(len) = len {
                if start < content_len {
                    content[start..std::cmp::min(start + len, content_len)].to_string()
                } else {
                    String::new()
                }
            } else {
                if start < content_len {
                    content[start..].to_string()
                } else {
                    content
                }
            };

            ToolResult {
                tool_call_id: tool_call.id.clone(),
                success: true,
                output: serde_json::json!({
                    "title": p.title,
                    "content": result,
                    "total_length": content_len
                }).to_string(),
                error: None,
            }
        }
        None => ToolResult {
            tool_call_id: tool_call.id.clone(),
            success: false,
            output: String::new(),
            error: Some("Passage not found".to_string()),
        },
    }
}

fn execute_list_passages(plaintext: &PlainText) -> ToolResult {
    let passages = plaintext.passages();
    let list: Vec<serde_json::Value> = passages.iter().enumerate().map(|(i, p)| {
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

fn execute_continue_writing(tool_call: &ParsedToolCall, plaintext: &mut PlainText) -> ToolResult {
    let idx = tool_call.arguments["passage_index"].as_u64().unwrap_or(0) as usize;
    let text = tool_call.arguments["text"].as_str().unwrap_or("");

    let passages = plaintext.passages();
    if idx >= passages.len() {
        return ToolResult {
            tool_call_id: tool_call.id.clone(),
            success: false,
            output: String::new(),
            error: Some(format!("Invalid passage index: {}", idx)),
        };
    }

    let current_content = &passages[idx].content;
    let new_content = current_content.clone() + text;
    plaintext.set_content(idx, new_content);

    ToolResult {
        tool_call_id: tool_call.id.clone(),
        success: true,
        output: "Text appended successfully".to_string(),
        error: None,
    }
}

fn execute_insert_text(tool_call: &ParsedToolCall, plaintext: &mut PlainText) -> ToolResult {
    let idx = tool_call.arguments["passage_index"].as_u64().unwrap_or(0) as usize;
    let position = tool_call.arguments["position"].as_u64().unwrap_or(0) as usize;
    let text = tool_call.arguments["text"].as_str().unwrap_or("");

    let passages = plaintext.passages();
    if idx >= passages.len() {
        return ToolResult {
            tool_call_id: tool_call.id.clone(),
            success: false,
            output: String::new(),
            error: Some(format!("Invalid passage index: {}", idx)),
        };
    }

    let current_content = &passages[idx].content;
    if position > current_content.len() {
        return ToolResult {
            tool_call_id: tool_call.id.clone(),
            success: false,
            output: String::new(),
            error: Some(format!("Invalid position: {} (content length: {})", position, current_content.len())),
        };
    }

    let new_content = current_content[..position].to_string() + text + &current_content[position..];
    let new_len = new_content.len();
    plaintext.set_content(idx, new_content);

    ToolResult {
        tool_call_id: tool_call.id.clone(),
        success: true,
        output: serde_json::json!({
            "inserted_at": position,
            "inserted_length": text.len(),
            "new_total_length": new_len
        }).to_string(),
        error: None,
    }
}

fn execute_replace_text(tool_call: &ParsedToolCall, plaintext: &mut PlainText) -> ToolResult {
    let idx = tool_call.arguments["passage_index"].as_u64().unwrap_or(0) as usize;
    let start = tool_call.arguments["start"].as_u64().unwrap_or(0) as usize;
    let end = tool_call.arguments["end"].as_u64().unwrap_or(0) as usize;
    let new_text = tool_call.arguments["new_text"].as_str().unwrap_or("");

    let passages = plaintext.passages();
    if idx >= passages.len() {
        return ToolResult {
            tool_call_id: tool_call.id.clone(),
            success: false,
            output: String::new(),
            error: Some(format!("Invalid passage index: {}", idx)),
        };
    }

    let current_content = &passages[idx].content;
    if start > end || end > current_content.len() {
        return ToolResult {
            tool_call_id: tool_call.id.clone(),
            success: false,
            output: String::new(),
            error: Some(format!("Invalid range: {}-{} (content length: {})", start, end, current_content.len())),
        };
    }

    let new_content = current_content[..start].to_string() + new_text + &current_content[end..];
    let new_len = new_content.len();
    plaintext.set_content(idx, new_content);

    ToolResult {
        tool_call_id: tool_call.id.clone(),
        success: true,
        output: serde_json::json!({
            "replaced_length": end - start,
            "new_length": new_text.len(),
            "new_total_length": new_len
        }).to_string(),
        error: None,
    }
}

// Character tools

fn execute_create_character(tool_call: &ParsedToolCall, workspace: &mut Workspace) -> ToolResult {
    let name = tool_call.arguments["name"].as_str().unwrap_or("").to_string();
    let description = tool_call.arguments["description"].as_str().unwrap_or("").to_string();

    if name.is_empty() {
        return ToolResult {
            tool_call_id: tool_call.id.clone(),
            success: false,
            output: String::new(),
            error: Some("Character name is required".to_string()),
        };
    }

    let aliases: Vec<String> = tool_call.arguments["aliases"].as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();

    let traits: Vec<String> = tool_call.arguments["traits"].as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();

    let character = workspace.add_character(name.clone(), description.clone());
    if !aliases.is_empty() {
        workspace.update_character(character.id, None, None, Some(aliases), None, None).ok();
    }
    if !traits.is_empty() {
        workspace.update_character(character.id, None, None, None, Some(traits), None).ok();
    }

    let updated_char = workspace.get_character(character.id).unwrap().clone();

    ToolResult {
        tool_call_id: tool_call.id.clone(),
        success: true,
        output: serde_json::to_string(&updated_char).unwrap_or_default(),
        error: None,
    }
}

fn execute_update_character(tool_call: &ParsedToolCall, workspace: &mut Workspace) -> ToolResult {
    let id = tool_call.arguments["character_id"].as_u64().unwrap_or(0) as usize;

    let name = tool_call.arguments["name"].as_str().map(|s| s.to_string());
    let description = tool_call.arguments["description"].as_str().map(|s| s.to_string());
    let aliases: Option<Vec<String>> = tool_call.arguments["aliases"].as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect());
    let traits: Option<Vec<String>> = tool_call.arguments["traits"].as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect());
    let notes = tool_call.arguments["notes"].as_str().map(|s| s.to_string());

    match workspace.update_character(id, name, description, aliases, traits, notes) {
        Ok(char) => ToolResult {
            tool_call_id: tool_call.id.clone(),
            success: true,
            output: serde_json::to_string(&char).unwrap_or_default(),
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

fn execute_delete_character(tool_call: &ParsedToolCall, workspace: &mut Workspace) -> ToolResult {
    let id = tool_call.arguments["character_id"].as_u64().unwrap_or(0) as usize;

    match workspace.remove_character(id) {
        Ok(char) => ToolResult {
            tool_call_id: tool_call.id.clone(),
            success: true,
            output: serde_json::json!({
                "deleted_character": char.name,
                "deleted_relationships": workspace.relationships.len()  // Count after removal
            }).to_string(),
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

fn execute_list_characters(workspace: &Workspace) -> ToolResult {
    let list: Vec<serde_json::Value> = workspace.characters.iter().map(|c| {
        serde_json::json!({
            "id": c.id,
            "name": c.name,
            "description": c.description,
            "aliases": c.aliases,
            "traits": c.traits
        })
    }).collect();

    ToolResult {
        tool_call_id: String::new(),
        success: true,
        output: serde_json::to_string(&list).unwrap_or_default(),
        error: None,
    }
}

fn execute_get_character(tool_call: &ParsedToolCall, workspace: &Workspace) -> ToolResult {
    let character = if let Some(id) = tool_call.arguments["character_id"].as_u64() {
        workspace.get_character(id as usize)
    } else if let Some(name) = tool_call.arguments["character_name"].as_str() {
        workspace.find_character_by_name(name)
    } else {
        None
    };

    match character {
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

fn execute_create_relationship(tool_call: &ParsedToolCall, workspace: &mut Workspace) -> ToolResult {
    let char_a = tool_call.arguments["character_a_id"].as_u64().unwrap_or(0) as usize;
    let char_b = tool_call.arguments["character_b_id"].as_u64().unwrap_or(0) as usize;
    let rel_type = tool_call.arguments["relationship_type"].as_str().unwrap_or("").to_string();
    let description = tool_call.arguments["description"].as_str().unwrap_or("").to_string();

    match workspace.add_relationship(char_a, char_b, rel_type, description) {
        Ok(rel) => ToolResult {
            tool_call_id: tool_call.id.clone(),
            success: true,
            output: serde_json::to_string(&rel).unwrap_or_default(),
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

fn execute_update_relationship(tool_call: &ParsedToolCall, workspace: &mut Workspace) -> ToolResult {
    let id = tool_call.arguments["relationship_id"].as_u64().unwrap_or(0) as usize;
    let rel_type = tool_call.arguments["relationship_type"].as_str().map(|s| s.to_string());
    let description = tool_call.arguments["description"].as_str().map(|s| s.to_string());

    match workspace.update_relationship(id, rel_type, description) {
        Ok(rel) => ToolResult {
            tool_call_id: tool_call.id.clone(),
            success: true,
            output: serde_json::to_string(&rel).unwrap_or_default(),
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

fn execute_delete_relationship(tool_call: &ParsedToolCall, workspace: &mut Workspace) -> ToolResult {
    let id = tool_call.arguments["relationship_id"].as_u64().unwrap_or(0) as usize;

    match workspace.remove_relationship(id) {
        Ok(rel) => ToolResult {
            tool_call_id: tool_call.id.clone(),
            success: true,
            output: serde_json::json!({"deleted_id": rel.id}).to_string(),
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

fn execute_list_relationships(workspace: &Workspace) -> ToolResult {
    let list: Vec<serde_json::Value> = workspace.relationships.iter().map(|r| {
        serde_json::json!({
            "id": r.id,
            "character_a_id": r.character_a_id,
            "character_b_id": r.character_b_id,
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

fn execute_query_character_relationships(tool_call: &ParsedToolCall, workspace: &Workspace) -> ToolResult {
    let char_id = if let Some(id) = tool_call.arguments["character_id"].as_u64() {
        id as usize
    } else if let Some(name) = tool_call.arguments["character_name"].as_str() {
        match workspace.find_character_by_name(name) {
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

    let relationships = workspace.get_character_relationships(char_id);
    let char_name = workspace.get_character(char_id)
        .map(|c| c.name.clone())
        .unwrap_or_default();

    let list: Vec<serde_json::Value> = relationships.iter().map(|r| {
        let other_id = if r.character_a_id == char_id { r.character_b_id } else { r.character_a_id };
        let other_name = workspace.get_character(other_id)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| format!("Unknown({})", other_id));

        serde_json::json!({
            "id": r.id,
            "with_character": other_name,
            "with_character_id": other_id,
            "type": r.relationship_type,
            "description": r.description
        })
    }).collect();

    ToolResult {
        tool_call_id: String::new(),
        success: true,
        output: serde_json::json!({
            "character": char_name,
            "character_id": char_id,
            "relationships": list
        }).to_string(),
        error: None,
    }
}

// KV store tools

fn execute_set_kv(tool_call: &ParsedToolCall, workspace: &mut Workspace) -> ToolResult {
    let key = tool_call.arguments["key"].as_str().unwrap_or("").to_string();
    let value = tool_call.arguments["value"].as_str().unwrap_or("").to_string();
    let category = tool_call.arguments["category"].as_str().unwrap_or("general").to_string();
    let notes = tool_call.arguments["notes"].as_str().unwrap_or("").to_string();

    if key.is_empty() {
        return ToolResult {
            tool_call_id: tool_call.id.clone(),
            success: false,
            output: String::new(),
            error: Some("Key is required".to_string()),
        };
    }

    workspace.set_kv(key.clone(), value, category, notes);

    ToolResult {
        tool_call_id: tool_call.id.clone(),
        success: true,
        output: serde_json::json!({"key": key, "set": true}).to_string(),
        error: None,
    }
}

fn execute_get_kv(tool_call: &ParsedToolCall, workspace: &Workspace) -> ToolResult {
    let key = tool_call.arguments["key"].as_str().unwrap_or("");

    match workspace.get_kv(key) {
        Some(entry) => ToolResult {
            tool_call_id: String::new(),
            success: true,
            output: serde_json::to_string(entry).unwrap_or_default(),
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

fn execute_delete_kv(tool_call: &ParsedToolCall, workspace: &mut Workspace) -> ToolResult {
    let key = tool_call.arguments["key"].as_str().unwrap_or("");

    match workspace.delete_kv(key) {
        Some(entry) => ToolResult {
            tool_call_id: tool_call.id.clone(),
            success: true,
            output: serde_json::json!({"deleted_key": entry.key}).to_string(),
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

fn execute_list_kv(tool_call: &ParsedToolCall, workspace: &Workspace) -> ToolResult {
    let category = tool_call.arguments["category"].as_str();

    let entries = if let Some(cat) = category {
        workspace.list_kv_by_category(cat)
    } else {
        workspace.key_value_store.iter().collect()
    };

    let list: Vec<serde_json::Value> = entries.iter().map(|e| {
        serde_json::json!({
            "key": e.key,
            "value": e.value,
            "category": e.category,
            "notes": e.notes
        })
    }).collect();

    ToolResult {
        tool_call_id: String::new(),
        success: true,
        output: serde_json::to_string(&list).unwrap_or_default(),
        error: None,
    }
}