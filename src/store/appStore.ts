import { create } from 'zustand';
import type { Reminder, AppSetting, Shortcut, ViewType, TranscriptionState, ModelStatus } from '../types/reminder';
import { setSetting } from '../services/settingsService';

interface AppState {
  reminders: Reminder[];
  settings: AppSetting[];
  shortcuts: Shortcut[];
  currentView: ViewType;
  transcriptionState: TranscriptionState;
  isMinimized: boolean;
  modelStatus: ModelStatus;
  pendingTranscriptionText: string | null;
  pendingRecording: boolean;
  pendingStopRecording: boolean;
  darkMode: boolean;
  setReminders: (reminders: Reminder[]) => void;
  addReminder: (reminder: Reminder) => void;
  removeReminder: (id: string) => void;
  updateReminderInList: (id: string, updates: Partial<Reminder>) => void;
  setSettings: (settings: AppSetting[]) => void;
  setShortcuts: (shortcuts: Shortcut[]) => void;
  setCurrentView: (view: ViewType) => void;
  setTranscriptionState: (state: TranscriptionState) => void;
  setIsMinimized: (minimized: boolean) => void;
  setModelStatus: (status: ModelStatus) => void;
  setPendingTranscriptionText: (text: string | null) => void;
  setPendingRecording: (pending: boolean) => void;
  setPendingStopRecording: (pending: boolean) => void;
  setDarkMode: (dark: boolean) => void;
  toggleTheme: () => void;
}

export const useAppStore = create<AppState>((set) => ({
  reminders: [],
  settings: [],
  shortcuts: [],
  currentView: 'dashboard',
  transcriptionState: { status: 'idle' },
  isMinimized: false,
  modelStatus: 'NotDownloaded',
  pendingTranscriptionText: null,
  pendingRecording: false,
  pendingStopRecording: false,
  darkMode: false,
  setReminders: (reminders) => set({ reminders }),
  addReminder: (reminder) => set((state) => ({ reminders: [reminder, ...state.reminders] })),
  removeReminder: (id) => set((state) => ({ reminders: state.reminders.filter((r) => r.id !== id) })),
  updateReminderInList: (id, updates) =>
    set((state) => ({
      reminders: state.reminders.map((r) => (r.id === id ? { ...r, ...updates } : r)),
    })),
  setSettings: (settings) => set({ settings }),
  setShortcuts: (shortcuts) => set({ shortcuts }),
  setCurrentView: (currentView) => set({ currentView }),
  setTranscriptionState: (transcriptionState) => set({ transcriptionState }),
  setIsMinimized: (isMinimized) => set({ isMinimized }),
  setModelStatus: (modelStatus) => set({ modelStatus }),
  setPendingTranscriptionText: (pendingTranscriptionText) => set({ pendingTranscriptionText }),
  setPendingRecording: (pendingRecording) => set({ pendingRecording }),
  setPendingStopRecording: (pendingStopRecording) => set({ pendingStopRecording }),
  setDarkMode: (darkMode) => set({ darkMode }),
  toggleTheme: () => {
    set((state) => {
      const newDark = !state.darkMode;
      localStorage.setItem('darkMode', String(newDark));
      if (newDark) {
        document.documentElement.classList.add('dark');
      } else {
        document.documentElement.classList.remove('dark');
      }
      setSetting('theme', newDark ? 'dark' : 'light').catch(() => {});
      return { darkMode: newDark };
    });
  },
}));
