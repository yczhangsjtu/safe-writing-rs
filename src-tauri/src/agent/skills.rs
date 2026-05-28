use serde::{Deserialize, Serialize};

/// A Skill is a preset task template for specific writing workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub system_prompt_addition: String,
    pub first_message: String,
}

/// Get all built-in skills
pub fn get_builtin_skills() -> Vec<Skill> {
    vec![
        Skill {
            id: "define_background".to_string(),
            name: "确定故事背景".to_string(),
            description: "通过问答引导用户定义故事的世界观、时代背景和环境设定".to_string(),
            system_prompt_addition: r#"You are guiding the user to define the story background. Your goal is to:

1. Ask questions about the world, time period, and setting
2. Listen to the user's answers and summarize them
3. Use update_ai_settings to record the background information
4. Be conversational and encouraging

Questions to explore:
- What kind of world is this? (fantasy, sci-fi, historical, modern, etc.)
- What time period or era?
- What is the geography or location?
- What are the key features of this world?
- Any special rules, systems, or elements?

When the user provides information, acknowledge it and ask follow-up questions.
Once you have enough information, use update_ai_settings to save the background."#.to_string(),
            first_message: "您好！让我们一起来确定您故事的背景。请问这是一个什么样的世界？比如是现代都市、古代历史、科幻未来，还是奇幻魔法世界？".to_string(),
        },
        Skill {
            id: "define_characters".to_string(),
            name: "设定人物".to_string(),
            description: "引导用户创建主要角色，包括名字、性格、外貌和背景".to_string(),
            system_prompt_addition: r#"You are guiding the user to define characters for their story. Your goal is to:

1. Ask questions about the main characters
2. Listen to descriptions and ask for clarifications
3. Use create_character to add characters to the workspace
4. Help develop rich, consistent character profiles

Questions to explore:
- Who are the main characters?
- What are their names, ages, and roles in the story?
- What are their personalities and traits?
- What do they look like?
- What is their background/history?
- What are their motivations/goals?

When you have enough information about a character, use create_character to add them.
Ask about relationships between characters as you discover them."#.to_string(),
            first_message: "您好！让我们来设定您故事中的人物。请问谁是您故事的主角？可以告诉我他们的名字和基本情况吗？".to_string(),
        },
        Skill {
            id: "define_relationships".to_string(),
            name: "设定关系".to_string(),
            description: "建立角色之间的关系，包括友情、爱情、敌对等".to_string(),
            system_prompt_addition: r#"You are guiding the user to define relationships between characters. Your goal is to:

1. Understand existing characters in the workspace
2. Ask about relationships between them
3. Use create_relationship to record connections
4. Explore the dynamics and history of each relationship

First, use list_characters to see existing characters.
Then ask questions like:
- How do these characters know each other?
- What kind of relationship do they have? (friend, enemy, family, lover, colleague, etc.)
- What is the history of their relationship?
- How does this relationship affect the story?

Use create_relationship to record each relationship."#.to_string(),
            first_message: "您好！让我们来设定角色之间的关系。首先让我看看现有的人物... 请告诉我这些角色之间有什么联系？比如他们是朋友、家人，还是有什么矛盾？".to_string(),
        },
        Skill {
            id: "outline_plot".to_string(),
            name: "规划情节大纲".to_string(),
            description: "帮助用户设计故事的主线情节和关键转折点".to_string(),
            system_prompt_addition: r#"You are helping the user outline their story plot. Your goal is to:

1. Understand the background and characters (use get_ai_settings and list_characters)
2. Ask about the main storyline
3. Help identify key plot points, conflicts, and resolutions
4. Use set_kv to store plot points with category "plot"

Questions to explore:
- What is the main conflict or problem in the story?
- What is the beginning situation?
- What are the key turning points?
- How does the story end?
- What are the subplots?

Summarize the plot outline and use set_kv with keys like "main_plot", "beginning", "middle", "end" to store it."#.to_string(),
            first_message: "您好！让我们来规划您的故事情节。首先让我了解一下现有的背景和人物... 请告诉我您故事的主线是什么？主角遇到了什么问题或挑战？".to_string(),
        },
        Skill {
            id: "writing_session".to_string(),
            name: "写作会话".to_string(),
            description: "专注于续写当前章节，根据背景和人物信息创作".to_string(),
            system_prompt_addition: r#"You are in a writing session focused on continuing the current passage. Your goal is to:

1. Read the current passage to understand context (use read_passage)
2. Check the background and settings (use get_ai_settings)
3. Review relevant characters (use list_characters and get_character)
4. Write content that fits the established context

Guidelines:
- Maintain consistency with background, settings, and character traits
- Use the established writing style
- Continue naturally from the existing text
- Use continue_writing to append new content

Before writing, always read the current passage and check settings.
Ask the user what they want to focus on if unclear."#.to_string(),
            first_message: "您好！让我们开始写作。首先让我读取当前章节的内容和背景信息... 请告诉我您想继续写什么内容？或者有什么特别的要求吗？".to_string(),
        },
    ]
}

/// Get a skill by ID
pub fn get_skill_by_id(id: &str) -> Option<Skill> {
    get_builtin_skills().into_iter().find(|s| s.id == id)
}

/// Custom skill stored in user settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomSkill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub system_prompt_addition: String,
    pub first_message: String,
    /// If set, replaces the system prompt of the built-in skill with this ID
    pub replaces_builtin: Option<String>,
}

impl From<CustomSkill> for Skill {
    fn from(custom: CustomSkill) -> Self {
        Skill {
            id: custom.id,
            name: custom.name,
            description: custom.description,
            system_prompt_addition: custom.system_prompt_addition,
            first_message: custom.first_message,
        }
    }
}

/// Merge built-in skills with custom skills (custom can override built-in)
pub fn get_all_skills(custom_skills: &[CustomSkill]) -> Vec<Skill> {
    let builtin = get_builtin_skills();

    // Start with built-in skills
    let mut result: Vec<Skill> = builtin.clone();

    // Apply custom overrides or add new skills
    for custom in custom_skills {
        if let Some(replaces) = &custom.replaces_builtin {
            // Override existing built-in skill
            if let Some(pos) = result.iter().position(|s| &s.id == replaces) {
                result[pos] = Skill::from(custom.clone());
            }
        } else {
            // Add as new skill (if not duplicate)
            if !result.iter().any(|s| s.id == custom.id) {
                result.push(Skill::from(custom.clone()));
            }
        }
    }

    result
}