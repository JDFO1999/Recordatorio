import { invoke } from '@tauri-apps/api/core';
import type { Shortcut } from '../types/reminder';

export async function getShortcuts(): Promise<Shortcut[]> {
  return invoke<Shortcut[]>('get_shortcuts');
}

export async function updateShortcut(id: string, accelerator: string, enabled: boolean): Promise<void> {
  return invoke('update_shortcut', { id, accelerator, enabled });
}

export async function refreshShortcuts(): Promise<void> {
  return invoke('refresh_shortcuts');
}

export async function checkShortcutConflict(accelerator: string): Promise<boolean> {
  return invoke<boolean>('check_shortcut_conflict', { accelerator });
}
