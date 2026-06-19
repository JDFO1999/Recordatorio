import { useAppStore } from '../store/appStore';
import { getAllReminders } from '../services/reminderService';
import { useEffect, useState } from 'react';
import type { Reminder } from '../types/reminder';

export function History() {
  const { setCurrentView } = useAppStore();
  const [history, setHistory] = useState<Reminder[]>([]);
  const [filter, setFilter] = useState<string>('completed');

  useEffect(() => {
    loadHistory();
  }, [filter]);

  const loadHistory = async () => {
    try {
      const items = await getAllReminders(filter);
      setHistory(items);
    } catch (e) {
      console.error(e);
    }
  };

  const formatDate = (dateStr: string) => {
    return new Date(dateStr).toLocaleString('es-ES');
  };

  const getStatusBadge = (status: string) => {
    const colors: Record<string, string> = {
      completed: 'bg-green-100 text-green-800',
      cancelled: 'bg-red-100 text-red-800',
      notified: 'bg-yellow-100 text-yellow-800',
    };
    return colors[status] || 'bg-gray-100 text-gray-800';
  };

  return (
    <div className="p-6 max-w-4xl mx-auto">
      <div className="flex items-center gap-4 mb-6">
        <button
          onClick={() => setCurrentView('dashboard')}
          className="text-gray-600 dark:text-gray-400 hover:text-gray-800"
        >
          ← Volver
        </button>
        <h1 className="text-2xl font-bold text-gray-800 dark:text-white">Historial</h1>
      </div>

      <div className="flex gap-2 mb-4">
        {['completed', 'cancelled', 'notified'].map((f) => (
          <button
            key={f}
            onClick={() => setFilter(f)}
            className={`px-3 py-1 rounded-full text-sm ${
              filter === f
                ? 'bg-blue-600 text-white'
                : 'bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300'
            }`}
          >
            {f === 'completed' ? 'Completados' : f === 'cancelled' ? 'Cancelados' : 'Notificados'}
          </button>
        ))}
      </div>

      {history.length === 0 ? (
        <p className="text-gray-500 dark:text-gray-400 text-center py-8">Sin resultados</p>
      ) : (
        <div className="space-y-2">
          {history.map((r) => (
            <div
              key={r.id}
              className="p-4 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700"
            >
              <div className="flex items-start justify-between">
                <div>
                  <h3 className="font-medium text-gray-900 dark:text-white">{r.title}</h3>
                  {r.description && (
                    <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">{r.description}</p>
                  )}
                  <p className="text-xs text-gray-400 mt-1">Creado: {formatDate(r.created_at)}</p>
                  {r.completed_at && (
                    <p className="text-xs text-green-500">Completado: {formatDate(r.completed_at)}</p>
                  )}
                  {r.cancelled_at && (
                    <p className="text-xs text-red-500">Cancelado: {formatDate(r.cancelled_at)}</p>
                  )}
                </div>
                <span
                  className={`px-2 py-1 rounded-full text-xs font-medium ${getStatusBadge(r.status)}`}
                >
                  {r.status}
                </span>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
