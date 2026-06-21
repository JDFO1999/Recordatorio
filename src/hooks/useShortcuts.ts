import { useEffect, useRef } from 'react';
import { useAppStore } from '../store/appStore';
import { onGlobalShortcut } from '../services/notificationService';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { sendNotification, isPermissionGranted, requestPermission } from '@tauri-apps/plugin-notification';

export function useShortcuts() {
  const {
    setCurrentView,
    transcriptionState,
    setTranscriptionState,
    setPendingRecording,
    setPendingStopRecording,
  } = useAppStore();
  const statusRef = useRef(transcriptionState.status);
  statusRef.current = transcriptionState.status;

  useEffect(() => {
    const unlisteners: (() => void)[] = [];

    unlisteners.push(
      onGlobalShortcut('start-recording', async () => {
        if (statusRef.current === 'transcribing') return;

        if (statusRef.current === 'recording') {
          // Toggle off: stop recording
          setPendingStopRecording(true);
          return;
        }

        // Toggle on: start recording
        try {
          const win = getCurrentWindow();
          await win.show();
          await win.setFocus();
        } catch { /* ignore */ }

        try {
          let granted = await isPermissionGranted();
          if (!granted) {
            const perm = await requestPermission();
            granted = perm === 'granted';
          }
          if (granted) {
            sendNotification({ title: '🎤 Grabando...', body: 'Di tu recordatorio' });
          }
        } catch { /* ignore */ }

        setTranscriptionState({ status: 'idle' });
        setCurrentView('dashboard');
        setPendingRecording(true);
      })
    );

    unlisteners.push(
      onGlobalShortcut('new-reminder', () => {
        setCurrentView('create');
      })
    );

    return () => {
      unlisteners.forEach((fn) => fn());
    };
  }, [setCurrentView, setTranscriptionState, setPendingRecording, setPendingStopRecording]);
}
