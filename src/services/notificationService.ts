import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export interface NotificationEvent {
  reminder_id: string;
  title: string;
  body: string;
}

export async function testNotification(): Promise<void> {
  return invoke('test_notification');
}

export function onNotificationAction(callback: (event: NotificationEvent) => void): () => void {
  const unlisten = listen<NotificationEvent>('notification-action', (event) => {
    callback(event.payload);
  });
  return () => {
    unlisten.then((fn) => fn());
  };
}

export function onGlobalShortcut(action: string, callback: () => void): () => void {
  const unlisten = listen(`global-shortcut:${action}`, () => {
    callback();
  });
  return () => {
    unlisten.then((fn) => fn());
  };
}
