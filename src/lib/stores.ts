import { writable } from 'svelte/store';
import type { Passage, Config, CopilotSettings, Workspace, Session, ToolCall, ToolDefinition } from '../types';

export const theme = writable<'light' | 'dark'>('dark');
export const files = writable<string[]>([]);
export const currentFile = writable<string | null>(null);
export const passages = writable<Passage[]>([]);
export const currentPassageIndex = writable<number>(0);
export const isDirty = writable<boolean>(false);
export const numImages = writable<number>(0);
export const config = writable<Config>({
  font_size: 24,
  data_dir: '',
  llamacpp_url: 'http://localhost:8080',
  theme: 'dark',
  sidebar_width: 200,
  passage_list_width: 160,
  copilot_width: 320
});
export const sidebarWidth = writable<number>(200);
export const passageListWidth = writable<number>(160);
export const copilotWidth = writable<number>(320);
export const copilotSettings = writable<CopilotSettings>({
  background: '',
  settings: '',
  notes: '',
  requirements: '',
  writing_style: '',
  custom_system_prompt: '',
  use_custom_prompt: false,
  custom_skills: []
});
export const copilotVisible = writable<boolean>(false);
export const galleryVisible = writable<boolean>(false);
export const isLoading = writable<boolean>(false);
export const error = writable<string | null>(null);
export const success = writable<string | null>(null);

// Agent stores
export const workspace = writable<Workspace>({
  next_character_id: 0,
  characters: [],
  next_relationship_id: 0,
  relationships: [],
  key_value_store: []
});
export const session = writable<Session>({
  next_message_id: 0,
  messages: [],
  summary: undefined,
  summary_cutoff_id: undefined
});
export const pendingToolCalls = writable<ToolCall[]>([]);
export const toolDefinitions = writable<ToolDefinition[]>([]);
export const workspacePanelVisible = writable<boolean>(false);
export const sessionPanelVisible = writable<boolean>(false);