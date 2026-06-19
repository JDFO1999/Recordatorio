import { useEffect } from 'react';
import { useAppStore } from '../store/appStore';
import { onGlobalShortcut } from '../services/notificationService';

export function useShortcuts() {
  const { setCurrentView, transcriptionState, setTranscriptionState } = useAppStore();

  useEffect(() => {
    const unlisteners: (() => void)[] = [];

    unlisteners.push(
      onGlobalShortcut('start-recording', () => {
        if (transcriptionState.status === 'recording') {
          return;
        }
        setTranscriptionState({ status: 'idle' });
        setCurrentView('dashboard');
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
  }, [setCurrentView, transcriptionState.status, setTranscriptionState]);
}
