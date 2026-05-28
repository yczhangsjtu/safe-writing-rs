use serde::{Deserialize, Serialize};

/// Definition of a tool available to the agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,  // JSON Schema for parameters
    pub confirmation_required: bool,   // Whether user confirmation is needed
}

/// Get all available tool definitions
pub fn get_tool_definitions() -> Vec<ToolDefinition> {
    vec![
        // Passage reading tools (auto-execute)
        ToolDefinition {
            name: "read_passage".to_string(),
            description: "Read the content of a specific passage by index or title. CALL THIS before writing to understand existing content and maintain consistency.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "passage_index": {
                        "type": "integer",
                        "description": "Index of the passage (0-based)"
                    },
                    "passage_title": {
                        "type": "string",
                        "description": "Title of the passage (alternative to index)"
                    },
                    "start_offset": {
                        "type": "integer",
                        "description": "Start reading from this character offset"
                    },
                    "length": {
                        "type": "integer",
                        "description": "Number of characters to read (default: all)"
                    }
                },
                "required": []
            }),
            confirmation_required: false,
        },
        ToolDefinition {
            name: "list_passages".to_string(),
            description: "List all passages in the document with their titles and content lengths. CALL THIS at the start of a session to understand document structure.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
            confirmation_required: false,
        },

        // Writing tools (require confirmation)
        ToolDefinition {
            name: "continue_writing".to_string(),
            description: "Append text to the end of a passage. Use this to extend the story. Ensure you've read existing content first.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "passage_index": {
                        "type": "integer",
                        "description": "Index of the passage to append to"
                    },
                    "text": {
                        "type": "string",
                        "description": "Text to append"
                    }
                },
                "required": ["passage_index", "text"]
            }),
            confirmation_required: true,
        },
        ToolDefinition {
            name: "insert_text".to_string(),
            description: "Insert text at a specific position in a passage.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "passage_index": {
                        "type": "integer",
                        "description": "Index of the passage"
                    },
                    "position": {
                        "type": "integer",
                        "description": "Character position to insert at"
                    },
                    "text": {
                        "type": "string",
                        "description": "Text to insert"
                    }
                },
                "required": ["passage_index", "position", "text"]
            }),
            confirmation_required: true,
        },
        ToolDefinition {
            name: "replace_text".to_string(),
            description: "Replace a span of text in a passage with new text.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "passage_index": {
                        "type": "integer",
                        "description": "Index of the passage"
                    },
                    "start": {
                        "type": "integer",
                        "description": "Start position of text to replace"
                    },
                    "end": {
                        "type": "integer",
                        "description": "End position of text to replace"
                    },
                    "new_text": {
                        "type": "string",
                        "description": "New text to replace with"
                    }
                },
                "required": ["passage_index", "start", "end", "new_text"]
            }),
            confirmation_required: true,
        },

        // Character tools (read auto-execute, write require confirmation)
        ToolDefinition {
            name: "create_character".to_string(),
            description: "Create a new character in the workspace. MUST CALL THIS when: (1) user introduces a new character, (2) you write about a character not yet stored, (3) user describes someone new. Proactive storage ensures consistency in future writing.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Character name"
                    },
                    "description": {
                        "type": "string",
                        "description": "Character description - appearance, personality, background"
                    },
                    "aliases": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Alternative names for the character"
                    },
                    "traits": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Character traits - personality keywords"
                    }
                },
                "required": ["name", "description"]
            }),
            confirmation_required: true,
        },
        ToolDefinition {
            name: "update_character".to_string(),
            description: "Update an existing character's details. MUST CALL THIS when: (1) user provides new info about existing character, (2) character develops in the story, (3) you learn more about their traits/background. Keep character data current for consistency.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "character_id": {
                        "type": "integer",
                        "description": "ID of the character to update"
                    },
                    "name": {
                        "type": "string",
                        "description": "New name (optional)"
                    },
                    "description": {
                        "type": "string",
                        "description": "New description (optional)"
                    },
                    "aliases": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "New aliases (optional)"
                    },
                    "traits": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "New traits (optional)"
                    },
                    "notes": {
                        "type": "string",
                        "description": "New notes (optional)"
                    }
                },
                "required": ["character_id"]
            }),
            confirmation_required: true,
        },
        ToolDefinition {
            name: "delete_character".to_string(),
            description: "Delete a character from the workspace. Also removes all relationships involving this character.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "character_id": {
                        "type": "integer",
                        "description": "ID of the character to delete"
                    }
                },
                "required": ["character_id"]
            }),
            confirmation_required: true,
        },
        ToolDefinition {
            name: "list_characters".to_string(),
            description: "List all characters in the workspace. CALL THIS at the start of any writing session to know who exists in the story world.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
            confirmation_required: false,
        },
        ToolDefinition {
            name: "get_character".to_string(),
            description: "Get details of a specific character by ID or name. CALL THIS before writing about a character to ensure accurate portrayal of their traits, description, and notes.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "character_id": {
                        "type": "integer",
                        "description": "ID of the character"
                    },
                    "character_name": {
                        "type": "string",
                        "description": "Name of the character (alternative to ID)"
                    }
                },
                "required": []
            }),
            confirmation_required: false,
        },

        // Relationship tools
        ToolDefinition {
            name: "create_relationship".to_string(),
            description: "Create a relationship between two characters. MUST CALL THIS when: (1) user describes character connections, (2) you establish new relationships in writing, (3) characters interact meaningfully. Track relationships for coherent character dynamics.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "character_a_id": {
                        "type": "integer",
                        "description": "ID of the first character"
                    },
                    "character_b_id": {
                        "type": "integer",
                        "description": "ID of the second character"
                    },
                    "relationship_type": {
                        "type": "string",
                        "description": "Type of relationship (e.g., friend, enemy, family, lover, rival, mentor)"
                    },
                    "description": {
                        "type": "string",
                        "description": "Description of the relationship dynamics"
                    }
                },
                "required": ["character_a_id", "character_b_id", "relationship_type"]
            }),
            confirmation_required: true,
        },
        ToolDefinition {
            name: "update_relationship".to_string(),
            description: "Update an existing relationship. CALL THIS when relationships evolve or user provides new details about character connections.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "relationship_id": {
                        "type": "integer",
                        "description": "ID of the relationship to update"
                    },
                    "relationship_type": {
                        "type": "string",
                        "description": "New relationship type (optional)"
                    },
                    "description": {
                        "type": "string",
                        "description": "New description (optional)"
                    }
                },
                "required": ["relationship_id"]
            }),
            confirmation_required: true,
        },
        ToolDefinition {
            name: "delete_relationship".to_string(),
            description: "Delete a relationship.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "relationship_id": {
                        "type": "integer",
                        "description": "ID of the relationship to delete"
                    }
                },
                "required": ["relationship_id"]
            }),
            confirmation_required: true,
        },
        ToolDefinition {
            name: "list_relationships".to_string(),
            description: "List all relationships in the workspace. CALL THIS to understand the social web between characters.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
            confirmation_required: false,
        },
        ToolDefinition {
            name: "query_character_relationships".to_string(),
            description: "Query all relationships involving a specific character. CALL THIS before writing interactions to understand how this character relates to others.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "character_id": {
                        "type": "integer",
                        "description": "ID of the character to query"
                    },
                    "character_name": {
                        "type": "string",
                        "description": "Name of the character (alternative to ID)"
                    }
                },
                "required": []
            }),
            confirmation_required: false,
        },

        // Key-value store tools
        ToolDefinition {
            name: "set_kv".to_string(),
            description: "Set a key-value entry in the workspace. MUST CALL THIS to store: (1) plot points and story events (category: 'plot'), (2) locations and places (category: 'location'), (3) themes and motifs (category: 'theme'), (4) timeline events (category: 'timeline'), (5) any important story element to remember. Proactive storage prevents contradictions and enables consistent writing.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "key": {
                        "type": "string",
                        "description": "Key name - use descriptive names like 'main_plot', 'protagonist_goal', 'chapter_1_events'"
                    },
                    "value": {
                        "type": "string",
                        "description": "Value - detailed description of the element"
                    },
                    "category": {
                        "type": "string",
                        "description": "Category for grouping: 'plot', 'location', 'theme', 'timeline', 'event', 'general'",
                        "default": "general"
                    },
                    "notes": {
                        "type": "string",
                        "description": "Additional notes or context"
                    }
                },
                "required": ["key", "value"]
            }),
            confirmation_required: true,
        },
        ToolDefinition {
            name: "get_kv".to_string(),
            description: "Get a value from the key-value store. CALL THIS to check previously stored story elements.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "key": {
                        "type": "string",
                        "description": "Key to retrieve"
                    }
                },
                "required": ["key"]
            }),
            confirmation_required: false,
        },
        ToolDefinition {
            name: "delete_kv".to_string(),
            description: "Delete a key-value entry.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "key": {
                        "type": "string",
                        "description": "Key to delete"
                    }
                },
                "required": ["key"]
            }),
            confirmation_required: true,
        },
        ToolDefinition {
            name: "list_kv".to_string(),
            description: "List all key-value entries, optionally filtered by category. CALL THIS at session start to review stored plot points, locations, themes, etc.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "category": {
                        "type": "string",
                        "description": "Category to filter by (optional): 'plot', 'location', 'theme', 'timeline', 'event'"
                    }
                },
                "required": []
            }),
            confirmation_required: false,
        },

        // AI Settings tools
        ToolDefinition {
            name: "get_ai_settings".to_string(),
            description: "Get the current AI writing settings (background, settings, notes, requirements, writing_style). ALWAYS CALL THIS at the start of a writing session to understand the story world, rules, and user preferences.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
            confirmation_required: false,
        },
        ToolDefinition {
            name: "update_ai_settings".to_string(),
            description: "Update AI writing settings. MUST CALL THIS when: (1) user describes world/setting → update 'background' or 'settings', (2) user gives preferences/rules → update 'notes' or 'requirements', (3) user specifies style → update 'writing_style', (4) during guided skill sessions to record user's answers. Proactive storage ensures all future writing respects these guidelines.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "background": {
                        "type": "string",
                        "description": "Story background - the world, time period, setting context. Update when user describes the world."
                    },
                    "settings": {
                        "type": "string",
                        "description": "World settings - rules, systems, technology, magic, social structures. Update when user defines world mechanics."
                    },
                    "notes": {
                        "type": "string",
                        "description": "Important notes for writing - things to remember, avoid, or emphasize. Update when user gives specific instructions."
                    },
                    "requirements": {
                        "type": "string",
                        "description": "Specific requirements for output - length, format, style constraints. Update when user specifies output expectations."
                    },
                    "writing_style": {
                        "type": "string",
                        "description": "Preferred writing style, tone, voice, literary devices. Update when user describes their stylistic preferences."
                    }
                },
                "required": []
            }),
            confirmation_required: true,
        },
    ]
}

/// Check if a tool requires confirmation
pub fn requires_confirmation(tool_name: &str) -> bool {
    get_tool_definitions()
        .iter()
        .find(|t| t.name == tool_name)
        .map(|t| t.confirmation_required)
        .unwrap_or(true)  // Default to requiring confirmation
}

/// Get tool definitions as JSON for the system prompt
pub fn get_tools_json() -> String {
    let tools = get_tool_definitions();
    serde_json::to_string_pretty(&tools).unwrap_or_default()
}