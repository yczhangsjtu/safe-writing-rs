use crate::agent::tools::get_tool_definitions;
use crate::agent::session::{ToolCall, ToolCallStatus};
use crate::agent::workspace::Workspace;
use crate::commands::copilot::{CopilotSettings, get_default_system_prompt};

/// Marker for tool calls in model output
pub const TOOL_CALL_START: &str = "<<TOOL_CALL";
pub const TOOL_CALL_END: &str = "TOOL_CALL>>";

/// A parsed tool call from model output
#[derive(Debug, Clone)]
pub struct ParsedToolCall {
    pub id: String,
    pub tool_name: String,
    pub arguments: serde_json::Value,
}

/// Parse tool calls from model output content
/// Returns (remaining_content_without_markers, parsed_tool_calls)
pub fn parse_tool_calls(content: &str) -> (String, Vec<ParsedToolCall>) {
    let mut remaining = content.to_string();
    let mut tool_calls = Vec::new();

    while let Some(start_idx) = remaining.find(TOOL_CALL_START) {
        let after_start = &remaining[start_idx + TOOL_CALL_START.len()..];

        if let Some(end_idx) = after_start.find(TOOL_CALL_END) {
            let json_str = &after_start[..end_idx].trim();

            // Parse the JSON
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_str) {
                let tool_name = parsed["name"].as_str().unwrap_or("").to_string();
                let arguments = parsed["arguments"].clone();

                tool_calls.push(ParsedToolCall {
                    id: format!("call_{}", uuid::Uuid::new_v4().to_string().replace("-", "").chars().take(8).collect::<String>()),
                    tool_name,
                    arguments,
                });
            }

            // Remove the tool call marker from content
            let total_len = TOOL_CALL_START.len() + end_idx + TOOL_CALL_END.len();
            remaining = remaining[..start_idx].to_string() + &remaining[start_idx + total_len..];
        } else {
            break;
        }
    }

    (remaining.trim().to_string(), tool_calls)
}

/// Build the system prompt for the agent
pub fn build_agent_system_prompt(workspace: &Workspace, settings: &CopilotSettings) -> String {
    let tools = get_tool_definitions();
    let tools_json = serde_json::to_string_pretty(&tools).unwrap_or_default();

    // Get base system prompt (custom or default)
    let base_prompt = if settings.use_custom_prompt && !settings.custom_system_prompt.is_empty() {
        &settings.custom_system_prompt
    } else {
        &get_default_system_prompt()
    };

    // Build writing guidance section
    let guidance_section = build_guidance_section(settings);

    // Build skills section
    let skills_section = build_skills_section(&settings.custom_skills);

    // Build workspace context
    let workspace_context = build_workspace_context(workspace);

    format!(
        r#"{base_prompt}

{workspace_context}

{guidance_section}

{skills_section}

AVAILABLE TOOLS:
{tools_json}

TOOL USAGE FORMAT:
To call a tool, use this exact format in your response:
<<TOOL_CALL{{"name": "tool_name", "arguments": {{...}}}}TOOL_CALL>>

CRITICAL: YOU MUST PROACTIVELY USE TOOLS IN THESE SCENARIOS:

## Memory & Knowledge Management (HIGH PRIORITY)
You MUST use tools to store and retrieve information to maintain consistency:

1. **When user provides new information about the story:**
   - User mentions a new character detail → use `update_character` or `create_character`
   - User describes world rules/magic/technology → use `update_ai_settings` to record in "settings"
   - User specifies preferences → use `update_ai_settings` to record in "notes" or "requirements"
   - User reveals plot elements → use `set_kv` with category "plot"

2. **Before writing content:**
   - ALWAYS call `list_characters` and `get_ai_settings` first to check existing context
   - If writing about a specific character, call `get_character` to verify their traits
   - If relationships are involved, call `query_character_relationships`

3. **After writing content:**
   - If you introduced new elements (names, places, rules), store them with `set_kv` or `create_character`
   - If plot progressed, update relevant `set_kv` entries with category "plot"

4. **When you discover inconsistencies:**
   - User contradicts stored information → update with correct data
   - You notice conflicting details → query existing data and suggest corrections

## Recommended Tool Call Patterns

**Starting a writing session:**
```
1. <<TOOL_CALL{{"name": "get_ai_settings", "arguments": {{}}}}TOOL_CALL>>
2. <<TOOL_CALL{{"name": "list_characters", "arguments": {{}}}}TOOL_CALL>>
3. <<TOOL_CALL{{"name": "list_kv", "arguments": {{}}}}TOOL_CALL>>
→ Then write with full context awareness
```

**User provides new story details:**
```
1. Acknowledge the information
2. <<TOOL_CALL{{"name": "update_ai_settings", "arguments": {{...}}}}TOOL_CALL>> OR
   <<TOOL_CALL{{"name": "create_character", "arguments": {{...}}}}TOOL_CALL>> OR
   <<TOOL_CALL{{"name": "set_kv", "arguments": {{...}}}}TOOL_CALL>>
3. Confirm what was stored
```

**Writing about characters:**
```
1. <<TOOL_CALL{{"name": "get_character", "arguments": {{...}}}}TOOL_CALL>>
2. <<TOOL_CALL{{"name": "query_character_relationships", "arguments": {{...}}}}TOOL_CALL>>
→ Write with accurate character knowledge
```

## Tool Categories

- **Auto-execute tools (no confirmation needed):** read_passage, list_passages, list_characters, get_character, list_relationships, query_character_relationships, get_kv, list_kv, get_ai_settings
  → Use freely to gather information

- **Confirmation-required tools:** continue_writing, insert_text, replace_text, create_character, update_character, delete_character, create_relationship, update_relationship, delete_relationship, set_kv, delete_kv, update_ai_settings
  → User will approve before execution

NEVER assume you know the current state - always query first. An informed agent writes better content."#,
        base_prompt = base_prompt,
        workspace_context = workspace_context,
        guidance_section = guidance_section,
        skills_section = skills_section,
        tools_json = tools_json
    )
}

/// Build workspace context summary
fn build_workspace_context(workspace: &Workspace) -> String {
    let char_list: Vec<String> = workspace.characters.iter()
        .map(|c| {
            let desc_preview = if c.description.is_empty() {
                String::new()
            } else {
                let preview: String = c.description.chars().take(50).collect();
                format!(": {}...", preview)
            };
            format!("  - [{}] {}{}", c.id, c.name, desc_preview)
        })
        .collect();

    let rel_list: Vec<String> = workspace.relationships.iter()
        .map(|r| format!("  - [{}] {} ↔ {} ({})", r.id,
            workspace.characters.iter().find(|c| c.id == r.character_a_id).map(|c| c.name.as_str()).unwrap_or("?"),
            workspace.characters.iter().find(|c| c.id == r.character_b_id).map(|c| c.name.as_str()).unwrap_or("?"),
            r.relationship_type))
        .collect();

    let kv_categories: std::collections::HashMap<String, Vec<&str>> = workspace.key_value_store.iter()
        .fold(std::collections::HashMap::new(), |mut acc, e| {
            acc.entry(e.category.clone()).or_default().push(&e.key);
            acc
        });
    let kv_summary: Vec<String> = kv_categories.iter()
        .map(|(cat, keys)| format!("  - {}: {}", cat, keys.join(", ")))
        .collect();

    let chars_str = if char_list.is_empty() { "  (none)" } else { &char_list.join("\n") };
    let rels_str = if rel_list.is_empty() { "  (none)" } else { &rel_list.join("\n") };
    let kvs_str = if kv_summary.is_empty() { "  (none)" } else { &kv_summary.join("\n") };

    format!(
        r#"CURRENT WORKSPACE STATE:
Characters ({count}):
{chars}
Relationships ({rel_count}):
{rels}
Key-Value Store ({kv_count}):
{kvs}
"#,
        count = workspace.characters.len(),
        chars = chars_str,
        rel_count = workspace.relationships.len(),
        rels = rels_str,
        kv_count = workspace.key_value_store.len(),
        kvs = kvs_str
    )
}

/// Build the skills section for the system prompt
fn build_skills_section(custom_skills: &[crate::agent::skills::CustomSkill]) -> String {
    let all_skills = crate::agent::skills::get_all_skills(custom_skills);

    if all_skills.is_empty() {
        return String::new();
    }

    let skills_info: Vec<String> = all_skills.iter().map(|skill| {
        format!(
            "- {} (id: '{}'): {}",
            skill.name,
            skill.id,
            skill.description
        )
    }).collect();

    format!(
        "AVAILABLE SKILLS:\n{}\n\nUse activate_skill to enter a skill mode. Each skill provides specialized guidance for specific writing tasks.",
        skills_info.join("\n")
    )
}

/// Build the writing guidance section from settings
fn build_guidance_section(settings: &CopilotSettings) -> String {
    let mut sections = Vec::new();

    if !settings.background.is_empty() {
        sections.push(format!("**STORY BACKGROUND** (stored in AI settings):\n{}\n→ Reference this when setting scenes. Update via `update_ai_settings` if user provides new world context.", settings.background));
    }
    if !settings.settings.is_empty() {
        sections.push(format!("**WORLD SETTINGS** (stored in AI settings):\n{}\n→ Reference for consistency. Update via `update_ai_settings` when user defines new rules/systems.", settings.settings));
    }
    if !settings.notes.is_empty() {
        sections.push(format!("**IMPORTANT NOTES** (stored in AI settings):\n{}\n→ MUST follow these. Update via `update_ai_settings` when user specifies preferences to remember.", settings.notes));
    }
    if !settings.requirements.is_empty() {
        sections.push(format!("**REQUIREMENTS** (stored in AI settings):\n{}\n→ Output must meet these criteria. Update via `update_ai_settings` if user adds constraints.", settings.requirements));
    }
    if !settings.writing_style.is_empty() {
        sections.push(format!("**WRITING STYLE** (stored in AI settings):\n{}\n→ Apply this style consistently. Update via `update_ai_settings` if user changes preferences.", settings.writing_style));
    }

    if sections.is_empty() {
        return String::from("WRITING GUIDANCE:\n(No guidance configured yet. Use `update_ai_settings` to store background, settings, notes, requirements, and writing_style when user provides them.)");
    }

    format!("WRITING GUIDANCE:\n{}\n\nREMINDER: All guidance above is stored in AI settings. Always check `get_ai_settings` before writing, and update via `update_ai_settings` when user provides new information.", sections.join("\n\n"))
}

/// Convert parsed tool calls to session ToolCall structs
pub fn to_session_tool_calls(parsed: Vec<ParsedToolCall>) -> Vec<ToolCall> {
    parsed.into_iter().map(|p| {
        let status = if crate::agent::tools::requires_confirmation(&p.tool_name) {
            ToolCallStatus::Pending
        } else {
            ToolCallStatus::Confirmed  // Auto-execute
        };

        ToolCall {
            id: p.id,
            tool_name: p.tool_name,
            arguments: p.arguments,
            status,
        }
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::workspace::Workspace;

    #[test]
    fn test_parse_single_tool_call() {
        let content = "Let me read the passage. <<TOOL_CALL{\"name\": \"read_passage\", \"arguments\": {\"passage_index\": 0}}TOOL_CALL>> Then I'll continue.";
        let (remaining, calls) = parse_tool_calls(content);

        assert!(!remaining.contains("TOOL_CALL"));
        assert!(remaining.contains("Let me read"));
        assert!(remaining.contains("Then I'll continue"));
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].tool_name, "read_passage");
    }

    #[test]
    fn test_parse_multiple_tool_calls() {
        let content = "<<TOOL_CALL{\"name\": \"read_passage\", \"arguments\": {\"passage_index\": 0}}TOOL_CALL>> <<TOOL_CALL{\"name\": \"list_characters\", \"arguments\": {}}TOOL_CALL>>";
        let (_, calls) = parse_tool_calls(content);
        assert_eq!(calls.len(), 2);
    }

    #[test]
    fn test_parse_no_tool_calls() {
        let content = "This is a regular response without any tool calls.";
        let (remaining, calls) = parse_tool_calls(content);
        assert_eq!(remaining, content);
        assert!(calls.is_empty());
    }

    #[test]
    fn test_build_system_prompt() {
        let workspace = Workspace::default();
        let settings = CopilotSettings::default();
        let prompt = build_agent_system_prompt(&workspace, &settings);
        assert!(prompt.contains("AVAILABLE TOOLS"));
        assert!(prompt.contains("read_passage"));
        assert!(prompt.contains("TOOL_CALL"));
    }

    #[test]
    fn test_build_system_prompt_with_guidance() {
        let workspace = Workspace::default();
        let settings = CopilotSettings {
            background: "A fantasy world with dragons".to_string(),
            settings: "Magic exists".to_string(),
            notes: String::new(),
            requirements: String::new(),
            writing_style: "Descriptive prose".to_string(),
            custom_system_prompt: String::new(),
            use_custom_prompt: false,
            custom_skills: Vec::new(),
        };
        let prompt = build_agent_system_prompt(&workspace, &settings);
        assert!(prompt.contains("STORY BACKGROUND"));
        assert!(prompt.contains("fantasy world"));
        assert!(prompt.contains("WORLD SETTINGS"));
        assert!(prompt.contains("WRITING STYLE"));
    }
}