import { useState, useEffect, useCallback } from 'react';
import type { Shortcut } from '../types/reminder';

interface ShortcutEditorProps {
  shortcut: Shortcut;
  allShortcuts: Shortcut[];
  onUpdate: (id: string, accelerator: string) => void;
  onToggle: (id: string, enabled: boolean) => void;
}

export function ShortcutEditor({ shortcut, allShortcuts, onUpdate, onToggle }: ShortcutEditorProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [keys, setKeys] = useState<string[]>([]);
  const [error, setError] = useState('');

  const actionLabels: Record<string, string> = {
    start_recording: 'Iniciar grabación',
    toggle_window: 'Abrir/Ocultar ventana',
    new_reminder: 'Nuevo recordatorio',
    snooze_last: 'Posponer último',
    complete_last: 'Completar último',
  };

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    e.preventDefault();
    const mods: string[] = [];
    if (e.ctrlKey) mods.push('Ctrl');
    if (e.altKey) mods.push('Alt');
    if (e.shiftKey) mods.push('Shift');
    if (e.metaKey) mods.push('Super');

    const key = e.key;
    if (['Control', 'Alt', 'Shift', 'Meta'].includes(key)) return;

    if (mods.length === 0) {
      setError('Debes incluir al menos una tecla modificadora (Ctrl, Alt, Shift)');
      return;
    }

    const combo = [...mods, key.toUpperCase()].join('+');
    setKeys([...mods, key.toUpperCase()]);

    const conflict = allShortcuts.find(
      (s) => s.id !== shortcut.id && s.accelerator === combo && s.enabled
    );
    if (conflict) {
      setError(`Combinación usada por: ${actionLabels[conflict.action] || conflict.action}`);
      return;
    }

    setError('');
    onUpdate(shortcut.id, combo);
    setIsEditing(false);
  }, [shortcut.id, allShortcuts, onUpdate]);

  useEffect(() => {
    if (isEditing) {
      const handler = (e: KeyboardEvent) => e.preventDefault();
      window.addEventListener('keydown', handler);
      return () => window.removeEventListener('keydown', handler);
    }
  }, [isEditing]);

  return (
    <div className="flex items-center justify-between py-2">
      <div className="flex-1">
        <p className="text-sm font-medium text-gray-700 dark:text-gray-300">
          {actionLabels[shortcut.action] || shortcut.action}
        </p>
      </div>
      <div className="flex items-center gap-3">
        {isEditing ? (
          <div
            className="px-3 py-1.5 bg-gray-100 dark:bg-gray-700 border-2 border-blue-500 rounded text-sm font-mono text-gray-800 dark:text-white min-w-[120px] text-center"
            onKeyDown={handleKeyDown}
            tabIndex={0}
            autoFocus
          >
            {keys.length > 0 ? keys.join('+') : 'Presiona teclas...'}
          </div>
        ) : (
          <span
            onClick={() => { setIsEditing(true); setKeys([]); setError(''); }}
            className="px-3 py-1.5 bg-gray-100 dark:bg-gray-700 rounded text-sm font-mono text-gray-800 dark:text-white cursor-pointer hover:bg-gray-200 dark:hover:bg-gray-600 min-w-[120px] text-center"
          >
            {shortcut.accelerator}
          </span>
        )}
        <label className="relative inline-flex items-center cursor-pointer">
          <input
            type="checkbox"
            checked={shortcut.enabled}
            onChange={(e) => onToggle(shortcut.id, e.target.checked)}
            className="sr-only peer"
          />
          <div className="w-9 h-5 bg-gray-200 peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-blue-300 dark:peer-focus:ring-blue-800 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all dark:border-gray-600 peer-checked:bg-blue-600"></div>
        </label>
      </div>
      {error && <p className="text-red-500 text-xs mt-1">{error}</p>}
    </div>
  );
}
