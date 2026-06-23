import type { Reminder } from '../types/reminder';

interface ReminderCardProps {
  reminder: Reminder;
  onComplete: (id: string) => void;
  onCancel: (id: string) => void;
  onDelete: (id: string) => void;
  onEdit?: (reminder: Reminder) => void;
}

export function ReminderCard({ reminder, onComplete, onCancel, onDelete, onEdit }: ReminderCardProps) {
  const dueDate = new Date(reminder.due_at);
  const now = new Date();
  const isOverdue = dueDate <= now && reminder.status === 'pending';
  const isSoon = !isOverdue && (dueDate.getTime() - now.getTime()) < 30 * 60 * 1000;

  return (
    <div
      className={`p-4 rounded-lg border ${
        isOverdue
          ? 'bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800'
          : isSoon
          ? 'bg-yellow-50 dark:bg-yellow-900/20 border-yellow-200 dark:border-yellow-800'
          : 'bg-white dark:bg-gray-800 border-gray-200 dark:border-gray-700'
      }`}
    >
      <div className="flex items-start justify-between gap-4">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <h3 className="font-medium text-gray-900 dark:text-white truncate">{reminder.title}</h3>
            {reminder.source === 'voice' && (
              <span className="text-xs text-blue-500">🎤</span>
            )}
            {reminder.created_by && (
              <span className="text-xs px-1.5 py-0.5 bg-gray-200 dark:bg-gray-700 text-gray-600 dark:text-gray-400 rounded">
                {reminder.created_by}
              </span>
            )}
          </div>
          {reminder.description && (
            <p className="text-sm text-gray-500 dark:text-gray-400 mt-1 line-clamp-2">{reminder.description}</p>
          )}
          <div className="flex items-center gap-3 mt-2 text-xs text-gray-400 dark:text-gray-500">
            <span>{dueDate.toLocaleDateString('es-ES')} {dueDate.toLocaleTimeString('es-ES', { hour: '2-digit', minute: '2-digit' })}</span>
            {reminder.original_text && <span className="italic">"{reminder.original_text}"</span>}
          </div>
        </div>

        <div className="flex items-center gap-1 shrink-0">
          <button
            onClick={() => onComplete(reminder.id)}
            className="p-1.5 text-green-600 hover:bg-green-50 dark:hover:bg-green-900/30 rounded"
            title="Completado"
          >
            ✓
          </button>
          <button
            onClick={() => onCancel(reminder.id)}
            className="p-1.5 text-orange-600 hover:bg-orange-50 dark:hover:bg-orange-900/30 rounded"
            title="Cancelar"
          >
            ✗
          </button>
          {onEdit && (
            <button
              onClick={() => onEdit(reminder)}
              className="p-1.5 text-blue-600 hover:bg-blue-50 dark:hover:bg-blue-900/30 rounded"
              title="Editar"
            >
              ✎
            </button>
          )}
          <button
            onClick={() => onDelete(reminder.id)}
            className="p-1.5 text-red-600 hover:bg-red-50 dark:hover:bg-red-900/30 rounded"
            title="Eliminar"
          >
            🗑
          </button>
        </div>
      </div>
    </div>
  );
}
