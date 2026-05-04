import { invoke } from '@tauri-apps/api/core';
import type { Passage, Config, FileInfo, OpenFileResult, DecryptResult, ImageInfo, CopilotSettings, CopilotMessage } from '../types';

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

export async function insertImage(imageData: number[]): Promise<string> {
  return invoke('insert_image', { imageData });
}

export async function getImages(): Promise<ImageInfo[]> {
  return invoke('get_images');
}

export async function getImageMetadata(index: number): Promise<string> {
  return invoke('get_image_metadata', { index });
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

export async function loadCopilotSettings(): Promise<CopilotSettings> {
  return invoke('load_ai_settings');
}

export async function saveCopilotSettings(settings: CopilotSettings): Promise<void> {
  return invoke('save_ai_settings', { settings });
}

export async function clearCopilot(): Promise<CopilotSettings> {
  return invoke('clear_copilot');
}

export async function abortGeneration(): Promise<void> {
  return invoke('abort_generation');
}

export async function sendMessage(
  prompt: string,
  currentPassage: string,
  buffers: string[],
  systemPrompt: string,
  messages: CopilotMessage[]
): Promise<void> {
  return invoke('send_message', {
    prompt,
    currentPassage,
    buffers,
    systemPrompt,
    messages
  });
}