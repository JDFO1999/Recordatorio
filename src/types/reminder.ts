export type ReminderStatus = 'pending' | 'notified' | 'completed' | 'cancelled' | 'snoozed';

export type TranscriptionProvider = 'whisper';

export type ModelStatus =
  | 'NotDownloaded'
  | { Downloading: number }
  | 'Ready'
  | { Error: string };

export type ViewType = 'dashboard' | 'create' | 'confirm' | 'history' | 'settings';

export interface Reminder {
  id: string;
  title: string;
  description: string | null;
  original_text: string | null;
  due_at: string;
  status: ReminderStatus;
  created_at: string;
  updated_at: string;
  notified_at: string | null;
  completed_at: string | null;
  cancelled_at: string | null;
  snooze_count: number;
  last_snoozed_at: string | null;
  parsed_time_expression: string | null;
  source: string;
  repeat_interval_seconds: number | null;
  created_by: string | null;
}

export interface ParsedReminder {
  title: string;
  original_text: string;
  due_at: string;
  confidence: number;
  parsed_time_expression: string | null;
  repeat_interval_seconds: number | null;
}

export interface AppSetting {
  key: string;
  value: string;
  updated_at: string;
}

export interface Shortcut {
  id: string;
  action: string;
  accelerator: string;
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

export interface NotificationEvent {
  id: string;
  reminder_id: string;
  event_type: string;
  created_at: string;
  metadata: string | null;
}

export interface TranscriptionState {
  status: 'idle' | 'recording' | 'transcribing' | 'done' | 'error';
  error?: string;
}

export interface SqlServerConfig {
  host: string;
  port: number;
  database: string;
  user: string;
  password: string;
  trust_certificate: boolean;
}
