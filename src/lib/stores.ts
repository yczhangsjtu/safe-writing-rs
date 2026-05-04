import { writable } from 'svelte/store';
import type { Passage, Config, CopilotSettings } from '../types';

export const theme = writable<'light' | 'dark'>('dark');
export const files = writable<string[]>([]);
export const currentFile = writable<string | null>(null);
export const passages = writable<Passage[]>([]);
export const currentPassageIndex = writable<number>(0);
export const isDirty = writable<boolean>(false);
export const config = writable<Config>({
  font_size: 24,
  data_dir: '',
  llamacpp_url: 'http://localhost:8080',
  theme: 'dark'
});
export const copilotSettings = writable<CopilotSettings>({
  system_prompt: 'You are a helpful writing assistant.',
  buffers: Array(10).fill(''),
  favorite_prompts: [],
  messages: []
});
export const copilotVisible = writable<boolean>(false);
export const isLoading = writable<boolean>(false);
export const error = writable<string | null>(null);
export const success = writable<string | null>(null);