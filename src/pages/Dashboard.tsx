import { useAppStore } from '../store/appStore';
import { ReminderCard } from '../components/ReminderCard';
import { VoiceRecorder } from '../components/VoiceRecorder';
import { getPendingReminders, markCompleted, markCancelled, deleteReminder } from '../services/reminderService';
import { useEffect, useState } from 'react';

export function Dashboard() {
  const { reminders, setReminders, setCurrentView } = useAppStore();
  const [pending, setPending] = useState<typeof reminders>([]);
  const [overdue, setOverdue] = useState<typeof reminders>([]);
  type ReminderType = typeof reminders[number];

  const loadReminders = async () => {
    try {
      const all = await getPendingReminders();
      setReminders(all);
      const now = new Date();
      setPending(all.filter((r) => new Date(r.due_at) > now));
      setOverdue(all.filter((r) => new Date(r.due_at) <= now));
    } catch (e) {
      console.error('Failed to load reminders', e);
    }
  };

  useEffect(() => {
    loadReminders();
    const interval = setInterval(loadReminders, 30000);
    return () => clearInterval(interval);
  }, []);

  const handleComplete = async (id: string) => {
    try {
      await markCompleted(id);
      loadReminders();
    } catch (e) {
      console.error(e);
    }
  };

  const handleCancel = async (id: string) => {
    try {
      await markCancelled(id);
      loadReminders();
    } catch (e) {
      console.error(e);
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await deleteReminder(id);
      loadReminders();
    } catch (e) {
      console.error(e);
    }
  };

  return (
    <div className="p-6 max-w-4xl mx-auto">
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold text-gray-800 dark:text-white">Recordatorios</h1>
        <div className="flex gap-2">
          <button
            onClick={() => setCurrentView('create')}
            className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 flex items-center gap-2"
          >
            + Nuevo
          </button>
        </div>
      </div>

      <VoiceRecorder onRecordingComplete={() => setCurrentView('confirm')} />

      {overdue.length > 0 && (
        <div className="mb-6">
          <h2 className="text-lg font-semibold text-red-600 dark:text-red-400 mb-3">
            Vencidos ({overdue.length})
          </h2>
          <div className="space-y-2">
            {overdue.map((r: ReminderType) => (
              <ReminderCard
                key={r.id}
                reminder={r}
                onComplete={handleComplete}
                onCancel={handleCancel}
                onDelete={handleDelete}
              />
            ))}
          </div>
        </div>
      )}

      <div>
        <h2 className="text-lg font-semibold text-gray-700 dark:text-gray-300 mb-3">
          Pendientes ({pending.length})
        </h2>
        {pending.length === 0 ? (
          <p className="text-gray-500 dark:text-gray-400 text-center py-8">
            No hay recordatorios pendientes
          </p>
        ) : (
          <div className="space-y-2">
            {pending.map((r: ReminderType) => (
              <ReminderCard
                key={r.id}
                reminder={r}
                onComplete={handleComplete}
                onCancel={handleCancel}
                onDelete={handleDelete}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
