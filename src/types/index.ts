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
  system_prompt: string;
  buffers: string[];
  favorite_prompts: FavoritePrompt[];
  messages: CopilotMessage[];
}