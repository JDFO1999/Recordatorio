import { invoke } from '@tauri-apps/api/core';
import type { AppSetting, Shortcut } from '../types/reminder';

export async function getSettings(): Promise<AppSetting[]> {
  return invoke('get_settings');
}

export async function getSetting(key: string): Promise<string | null> {
  return invoke('get_setting', { key });
}

export async function setSetting(key: string, value: string): Promise<void> {
  return invoke('set_setting', { key, value });
}

export async function getShortcuts(): Promise<Shortcut[]> {
  return invoke('get_shortcuts');
}

export async function updateShortcut(id: string, accelerator: string, enabled: boolean): Promise<void> {
  return invoke('update_shortcut', { id, accelerator, enabled });
}

export async function refreshShortcuts(): Promise<void> {
  return invoke('refresh_shortcuts');
}

export async function testNotification(): Promise<void> {
  return invoke('test_notification');
}
