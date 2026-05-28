export interface Passage {
  id: number;
  title: string;
  content: string;
}

export interface Config {
  font_size: number;
  data_dir: string;
  llamacpp_url: string;
  theme: string;
  sidebar_width: number;
  passage_list_width: number;
  copilot_width: number;
}

export interface FileInfo {
  name: string;
  is_encrypted: boolean;
}

export interface OpenFileResult {
  filename: string;
  is_new: boolean;
  ciphertext: string | null;
}

export interface DecryptResult {
  passages: Passage[];
  num_images: number;
  filename: string;
}

export interface ImageInfo {
  digest: string;
  index: number;
  data?: string; // base64 encoded image data
}

export interface CopilotMessage {
  role: string;
  content: string;
  display: string;
}

export interface FavoritePrompt {
  name: string;
  prompt: string;
}

export interface CopilotSettings {
  // Core writing guidance
  background: string;      // 故事背景
  settings: string;        // 设定
  notes: string;           // 注意事项
  requirements: string;    // 要求
  writing_style: string;   // 写作风格

  // User customizable system prompt
  custom_system_prompt: string;

  // Whether to use default prompt or custom
  use_custom_prompt: boolean;

  // Custom skills
  custom_skills: CustomSkill[];
}

// Skill types
export interface Skill {
  id: string;
  name: string;
  description: string;
  system_prompt_addition: string;
  first_message: string;
}

export interface CustomSkill {
  id: string;
  name: string;
  description: string;
  system_prompt_addition: string;
  first_message: string;
  replaces_builtin?: string;
}

// Agent types

export interface Character {
  id: number;
  name: string;
  description: string;
  aliases: string[];
  traits: string[];
  notes: string;
}

export interface Relationship {
  id: number;
  character_a_id: number;
  character_b_id: number;
  relationship_type: string;
  description: string;
}

export interface KeyValueEntry {
  key: string;
  value: string;
  category: string;
  notes: string;
}

export interface Workspace {
  next_character_id: number;
  characters: Character[];
  next_relationship_id: number;
  relationships: Relationship[];
  key_value_store: KeyValueEntry[];
}

export type ToolCallStatus = 'Pending' | 'Confirmed' | 'Executed' | 'Failed' | 'Cancelled';

export interface ToolCall {
  id: string;
  tool_name: string;
  arguments: object;
  status: ToolCallStatus;
}

export interface SessionMessage {
  id: number;
  role: string;
  content: string;
  tool_calls?: ToolCall[];
  tool_call_id?: string;
  timestamp: number;
  compressed: boolean;
}

export interface Session {
  next_message_id: number;
  messages: SessionMessage[];
  summary?: string;
  summary_cutoff_id?: number;
}

export interface ToolDefinition {
  name: string;
  description: string;
  parameters: object;
  confirmation_required: boolean;
}

export interface ToolResult {
  tool_call_id: string;
  success: boolean;
  output: string;
  error?: string;
}