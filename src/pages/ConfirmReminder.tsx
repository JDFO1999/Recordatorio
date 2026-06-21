import { useState, useEffect } from 'react';
import { useAppStore } from '../store/appStore';
import { parseText, createReminder } from '../services/reminderService';
import type { ParsedReminder } from '../types/reminder';

export function ConfirmReminder() {
  const { setCurrentView, pendingTranscriptionText, setPendingTranscriptionText } = useAppStore();
  const [parsed, setParsed] = useState<ParsedReminder | null>(null);
  const [title, setTitle] = useState('');
  const [dueDate, setDueDate] = useState('');
  const [dueTime, setDueTime] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!pendingTranscriptionText) return;
    parseText(pendingTranscriptionText)
      .then((result) => {
        setParsed(result);
        setTitle(result.title);
        setDueDate(result.due_at.slice(0, 10));
        setDueTime(result.due_at.slice(11, 16));
      })
      .catch(() => {
        setTitle(pendingTranscriptionText);
        const future = new Date(Date.now() + 3600000);
        setDueDate(future.toISOString().slice(0, 10));
        setDueTime(future.toISOString().slice(11, 16));
      });
  }, [pendingTranscriptionText]);

  const handleConfirm = async () => {
    if (!title.trim()) {
      setError('El título es obligatorio');
      return;
    }
    const dueAt = `${dueDate}T${dueTime}:00`;
    setLoading(true);
    try {
      await createReminder(title.trim(), null, dueAt, 'voice');
      setPendingTranscriptionText(null);
      setCurrentView('dashboard');
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const handleCancel = () => {
    setPendingTranscriptionText(null);
    setCurrentView('dashboard');
  };

  if (!pendingTranscriptionText) {
    return (
      <div className="p-6 max-w-2xl mx-auto text-center">
        <p className="text-gray-500 dark:text-gray-400">No hay texto pendiente de confirmación</p>
        <button
          onClick={() => setCurrentView('dashboard')}
          className="mt-4 px-4 py-2 bg-blue-600 text-white rounded-lg"
        >
          Volver
        </button>
      </div>
    );
  }

  return (
    <div className="p-6 max-w-2xl mx-auto">
      <div className="flex items-center gap-4 mb-6">
        <button
          onClick={() => { setPendingTranscriptionText(null); setCurrentView('dashboard'); }}
          className="text-gray-600 dark:text-gray-400 hover:text-gray-800"
        >
          ← Volver
        </button>
        <h1 className="text-2xl font-bold text-gray-800 dark:text-white">Confirmar recordatorio</h1>
      </div>

      <div className="space-y-4">
        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
            Texto original
          </label>
          <p className="p-3 bg-gray-100 dark:bg-gray-700 rounded-lg text-gray-700 dark:text-gray-300 italic">
            "{pendingTranscriptionText}"
          </p>
        </div>

        {parsed && parsed.confidence < 0.5 && (
          <p className="text-yellow-600 bg-yellow-50 dark:bg-yellow-900/20 p-3 rounded-lg text-sm">
            No se pudo determinar la fecha/hora con precisión. Por favor verifica.
          </p>
        )}

        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
            Título
          </label>
          <input
            type="text"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
          />
        </div>

        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Fecha
            </label>
            <input
              type="date"
              value={dueDate}
              onChange={(e) => setDueDate(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Hora
            </label>
            <input
              type="time"
              value={dueTime}
              onChange={(e) => setDueTime(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
            />
          </div>
        </div>

        {error && <p className="text-red-500 text-sm">{error}</p>}

        <div className="flex gap-3 pt-2">
          <button
            onClick={handleConfirm}
            disabled={loading}
            className="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50"
          >
            {loading ? 'Guardando...' : 'Confirmar'}
          </button>
          <button
            onClick={handleCancel}
            className="px-6 py-2 bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-300"
          >
            Cancelar
          </button>
        </div>
      </div>
    </div>
  );
}
