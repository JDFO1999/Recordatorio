import { invoke } from '@tauri-apps/api/core';
import type { Reminder, ParsedReminder } from '../types/reminder';

export async function createReminder(title: string, description: string | null, dueAt: string, source?: string, repeatIntervalSeconds?: number | null): Promise<Reminder> {
  return invoke('create_reminder', { title, description, dueAt, source, repeatIntervalSeconds });
}

export async function createReminderFromVoice(text: string): Promise<Reminder> {
  return invoke('create_reminder_from_voice', { text });
}

export async function getPendingReminders(): Promise<Reminder[]> {
  return invoke('get_pending_reminders');
}

export async function getAllReminders(statusFilter?: string): Promise<Reminder[]> {
  return invoke('get_all_reminders', { statusFilter: statusFilter || null });
}

export async function getReminderById(id: string): Promise<Reminder | null> {
  return invoke('get_reminder_by_id', { id });
}

export async function updateReminder(id: string, title: string, description: string | null, dueAt: string): Promise<void> {
  return invoke('update_reminder', { id, title, description, dueAt });
}

export async function markCompleted(id: string): Promise<void> {
  return invoke('mark_completed', { id });
}

export async function markCancelled(id: string): Promise<void> {
  return invoke('mark_cancelled', { id });
}

export async function snoozeReminder(id: string, minutes: number): Promise<void> {
  return invoke('snooze_reminder', { id, minutes });
}

export async function deleteReminder(id: string): Promise<void> {
  return invoke('delete_reminder', { id });
}

export async function parseText(text: string): Promise<ParsedReminder> {
  return invoke('parse_text', { text });
}
