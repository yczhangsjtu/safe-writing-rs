import { invoke } from '@tauri-apps/api/core';
import type { Passage, Config, FileInfo, OpenFileResult, DecryptResult, ImageInfo, CopilotSettings, CopilotMessage, Workspace, Session, Character, Relationship, KeyValueEntry, SessionMessage, ToolDefinition, ToolCall, ToolResult } from '../types';

// Check if running in Tauri environment
const isTauri = typeof window !== 'undefined' && '__TAURI__' in window;

export interface AppStateResponse {
  current_file: string | null;
  passages: Passage[];
  current_passage_index: number;
  is_dirty: boolean;
  num_images: number;
  copilot_settings: CopilotSettings;
  workspace?: Workspace;
  session?: Session;
}

export async function listFiles(): Promise<string[]> {
  return invoke('list_files');
}

export async function createFile(filename: string, password: string): Promise<void> {
  return invoke('create_file', { filename, password });
}

export async function openFile(filename: string): Promise<OpenFileResult> {
  return invoke('open_file', { filename });
}

export async function deleteFile(filename: string): Promise<void> {
  return invoke('delete_file', { filename });
}

export async function decryptFile(filename: string, ciphertext: string, password: string): Promise<DecryptResult> {
  return invoke('decrypt_file', { filename, ciphertext, password });
}

export async function encryptAndSave(): Promise<void> {
  return invoke('encrypt_and_save');
}

export async function closeFile(): Promise<void> {
  return invoke('close_file');
}

export async function changePassword(oldPassword: string, newPassword: string): Promise<void> {
  return invoke('change_password', { oldPassword, newPassword });
}

export async function getPassages(): Promise<Passage[]> {
  return invoke('get_passages');
}

export async function updatePassageContent(index: number, content: string): Promise<void> {
  return invoke('update_passage_content', { index, content });
}

export async function updatePassageTitle(index: number, title: string): Promise<void> {
  return invoke('update_passage_title', { index, title });
}

export async function addPassage(title: string): Promise<number> {
  return invoke('add_passage', { title });
}

export async function removePassage(index: number): Promise<void> {
  return invoke('remove_passage', { index });
}

export async function movePassage(from: number, to: number): Promise<void> {
  return invoke('move_passage', { from, to });
}

export async function insertImage(imageDataB64: string): Promise<string> {
  return invoke('insert_image', { imageDataB64 });
}

export async function getImages(): Promise<ImageInfo[]> {
  return invoke('get_images');
}

export async function getImageMetadata(index: number): Promise<string> {
  return invoke('get_image_metadata', { index });
}

export async function deleteImage(digest: string): Promise<void> {
  return invoke('delete_image', { digest });
}

export async function findReferencedDigests(): Promise<string[]> {
  return invoke('find_referenced_digests');
}

export async function getCurrentFile(): Promise<string> {
  return invoke('get_current_file');
}

export async function setCurrentPassage(index: number): Promise<void> {
  return invoke('set_current_passage', { index });
}

export async function getCurrentPassageIndex(): Promise<number> {
  return invoke('get_current_passage_index');
}

export async function isDirty(): Promise<boolean> {
  return invoke('is_dirty');
}

export async function getConfig(): Promise<Config> {
  return invoke('get_config');
}

export async function updateConfig(config: Partial<Config>): Promise<Config> {
  return invoke('update_config', config);
}

export async function getDataDir(): Promise<string> {
  return invoke('get_data_dir');
}

export async function getAppState(): Promise<AppStateResponse> {
  return invoke('get_app_state');
}

export async function loadCopilotSettings(): Promise<CopilotSettings> {
  return invoke('load_ai_settings');
}

export async function saveCopilotSettings(settings: CopilotSettings): Promise<void> {
  return invoke('save_ai_settings', { settings });
}

export async function clearCopilot(): Promise<CopilotSettings> {
  return invoke('clear_copilot');
}

export async function updateCopilotSettings(settings: CopilotSettings): Promise<void> {
  return invoke('update_copilot_settings', { settings });
}

export async function readClipboardImages(): Promise<{ name: string; data: string }[]> {
  return invoke('read_clipboard_images');
}

export async function readImageFiles(uris: string[]): Promise<{ name: string; data: string }[]> {
  return invoke('read_image_files', { uris });
}

export async function abortGeneration(): Promise<void> {
  return invoke('abort_generation');
}

export async function resetCopilotSettings(): Promise<CopilotSettings> {
  return invoke('reset_copilot_settings');
}

export async function sendMessage(
  prompt: string,
  currentPassage: string
): Promise<void> {
  return invoke('send_message', {
    prompt,
    currentPassage
  });
}

// ========== Workspace API ==========

export async function getWorkspace(): Promise<Workspace> {
  return invoke('get_workspace');
}

export async function getCharacters(): Promise<Character[]> {
  return invoke('get_characters');
}

export async function addCharacter(name: string, description: string): Promise<Character> {
  return invoke('add_character', { name, description });
}

export async function updateCharacter(
  characterId: number,
  name?: string,
  description?: string,
  aliases?: string[],
  traits?: string[],
  notes?: string
): Promise<Character> {
  return invoke('update_character', { characterId, name, description, aliases, traits, notes });
}

export async function removeCharacter(characterId: number): Promise<Character> {
  return invoke('remove_character', { characterId });
}

export async function getRelationships(): Promise<Relationship[]> {
  return invoke('get_relationships');
}

export async function addRelationship(
  characterAId: number,
  characterBId: number,
  relationshipType: string,
  description: string
): Promise<Relationship> {
  return invoke('add_relationship', { characterAId, characterBId, relationshipType, description });
}

export async function updateRelationship(
  relationshipId: number,
  relationshipType?: string,
  description?: string
): Promise<Relationship> {
  return invoke('update_relationship', { relationshipId, relationshipType, description });
}

export async function removeRelationship(relationshipId: number): Promise<Relationship> {
  return invoke('remove_relationship', { relationshipId });
}

export async function getKvStore(): Promise<KeyValueEntry[]> {
  return invoke('get_kv_store');
}

export async function setKv(key: string, value: string, category: string, notes: string): Promise<void> {
  return invoke('set_kv', { key, value, category, notes });
}

export async function deleteKv(key: string): Promise<KeyValueEntry> {
  return invoke('delete_kv', { key });
}

// ========== Session API ==========

export async function getSession(): Promise<Session> {
  return invoke('get_session');
}

export async function editSessionMessage(messageId: number, newContent: string): Promise<void> {
  return invoke('edit_session_message', { messageId, newContent });
}

export async function deleteSessionMessage(messageId: number): Promise<SessionMessage> {
  return invoke('delete_session_message', { messageId });
}

export async function compressSession(summary: string): Promise<void> {
  return invoke('compress_session', { summary });
}

export async function clearSession(): Promise<void> {
  return invoke('clear_session');
}

// ========== Agent API ==========

export async function getToolDefinitions(): Promise<ToolDefinition[]> {
  return invoke('get_tool_definitions');
}

export async function confirmToolCall(toolCallId: string, confirmed: boolean): Promise<void> {
  return invoke('confirm_tool_call', { toolCallId, confirmed });
}

export async function executeConfirmedTool(toolCallId: string): Promise<ToolResult> {
  return invoke('execute_confirmed_tool', { toolCallId });
}

export async function getPendingToolCalls(): Promise<ToolCall[]> {
  return invoke('get_pending_tool_calls');
}

// ========== Agent Message ==========

export async function sendAgentMessage(prompt: string, currentPassage: string): Promise<void> {
  return invoke('send_agent_message', { prompt, currentPassage });
}