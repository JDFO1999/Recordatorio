import { invoke } from '@tauri-apps/api/core';
import type { ModelStatus } from '../types/reminder';

export interface TranscriptionProvider {
  transcribe(audioPath: string): Promise<string>;
}

class WhisperLocalProvider implements TranscriptionProvider {
  async transcribe(audioPath: string): Promise<string> {
    const text = await invoke<string>('transcribe_audio', { path: audioPath });
    return text;
  }
}

export const transcriptionProvider: TranscriptionProvider = new WhisperLocalProvider();

export async function getModelStatus(): Promise<ModelStatus> {
  return invoke<ModelStatus>('get_model_status');
}

export async function downloadModel(): Promise<void> {
  return invoke<void>('download_model');
}

export async function transcribeAudio(audioPath: string): Promise<string> {
  return transcriptionProvider.transcribe(audioPath);
}
