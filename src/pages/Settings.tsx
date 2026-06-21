import { useState, useEffect } from 'react';
import { useAppStore } from '../store/appStore';
import { getSettings, setSetting, getShortcuts, updateShortcut, testNotification } from '../services/settingsService';
import { ShortcutEditor } from '../components/ShortcutEditor';
import { ModelDownloader } from '../components/ModelDownloader';
import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart';
import { open } from '@tauri-apps/plugin-dialog';

export function Settings() {
  const { settings, setSettings, shortcuts, setShortcuts, setCurrentView, darkMode, setDarkMode, toggleTheme } = useAppStore();
  const [autostart, setAutostart] = useState(false);

  useEffect(() => {
    loadSettings();
    checkAutostart();
    const savedDark = localStorage.getItem('darkMode') === 'true';
    if (savedDark !== darkMode) {
      setDarkMode(savedDark);
    }
  }, []);

  const loadSettings = async () => {
    try {
      const s = await getSettings();
      setSettings(s);
      const sc = await getShortcuts();
      setShortcuts(sc);
    } catch {
      // ignore
    }
  };

  const checkAutostart = async () => {
    try {
      const enabled = await isEnabled();
      setAutostart(enabled);
    } catch {
      // ignore
    }
  };

  const toggleAutostart = async () => {
    try {
      if (autostart) {
        await disable();
      } else {
        await enable();
      }
      setAutostart(!autostart);
    } catch {
      // ignore
    }
  };

  const handleShortcutUpdate = async (id: string, accelerator: string) => {
    try {
      await updateShortcut(id, accelerator, true);
      loadSettings();
    } catch {
      // ignore
    }
  };

  const handleShortcutToggle = async (id: string, enabled: boolean) => {
    try {
      const sc = shortcuts.find((s) => s.id === id);
      if (sc) {
        await updateShortcut(id, sc.accelerator, enabled);
        loadSettings();
      }
    } catch {
      // ignore
    }
  };

  const getSettingValue = (key: string): string => {
    return settings.find((s) => s.key === key)?.value || '';
  };

  const updateSetting = async (key: string, value: string) => {
    try {
      await setSetting(key, value);
      loadSettings();
    } catch {
      // ignore
    }
  };

  const browseSoundFile = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: 'Audio WAV', extensions: ['wav'] }],
      });
      if (selected) {
        await updateSetting('notification_sound_path', selected);
      }
    } catch {
      // ignore
    }
  };

  const resetSoundToDefault = async () => {
    await updateSetting('notification_sound_path', '');
  };

  const toggleDarkMode = () => {
    toggleTheme();
  };

  return (
    <div className="p-6 max-w-2xl mx-auto">
      <div className="flex items-center gap-4 mb-6">
        <button
          onClick={() => setCurrentView('dashboard')}
          className="text-gray-600 dark:text-gray-400 hover:text-gray-800"
        >
          ← Volver
        </button>
        <h1 className="text-2xl font-bold text-gray-800 dark:text-white">Configuración</h1>
      </div>

      <div className="space-y-6">
        {/* General */}
        <section className="bg-white dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700">
          <h2 className="text-lg font-semibold text-gray-800 dark:text-white mb-4">General</h2>
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="font-medium text-gray-700 dark:text-gray-300">Iniciar con Windows</p>
                <p className="text-xs text-gray-500 dark:text-gray-400">Abrir automáticamente al iniciar sesión</p>
              </div>
              <label className="relative inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  checked={autostart}
                  onChange={toggleAutostart}
                  className="sr-only peer"
                />
                <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-blue-300 dark:peer-focus:ring-blue-800 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-blue-600" />
              </label>
            </div>

            <div className="flex items-center justify-between">
              <div>
                <p className="font-medium text-gray-700 dark:text-gray-300">Modo oscuro</p>
                <p className="text-xs text-gray-500 dark:text-gray-400">Cambiar entre tema claro y oscuro</p>
              </div>
              <label className="relative inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  checked={darkMode}
                  onChange={toggleDarkMode}
                  className="sr-only peer"
                />
                <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-blue-300 dark:peer-focus:ring-blue-800 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-blue-600" />
              </label>
            </div>

            <div className="flex items-center justify-between">
              <div>
                <p className="font-medium text-gray-700 dark:text-gray-300">Sonido de notificación</p>
                <p className="text-xs text-gray-500 dark:text-gray-400">Archivo .wav para el sonido de alerta</p>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-xs text-gray-500 dark:text-gray-400 truncate max-w-[120px]">
                  {getSettingValue('notification_sound_path')
                    ? getSettingValue('notification_sound_path').split('\\').pop()
                    : 'Predeterminado (ritmo.wav)'}
                </span>
                <button
                  onClick={browseSoundFile}
                  className="px-2 py-1 bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded hover:bg-gray-300 dark:hover:bg-gray-600 text-xs"
                >
                  Examinar
                </button>
                <button
                  onClick={resetSoundToDefault}
                  className="px-2 py-1 bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded hover:bg-gray-300 dark:hover:bg-gray-600 text-xs"
                >
                  Default
                </button>
              </div>
            </div>

            <div className="flex items-center justify-between">
              <div>
                <p className="font-medium text-gray-700 dark:text-gray-300">Intervalo de revisión</p>
                <p className="text-xs text-gray-500 dark:text-gray-400">Cada cuántos segundos revisar recordatorios</p>
              </div>
              <input
                type="number"
                min={10}
                max={300}
                value={getSettingValue('scheduler_interval_secs') || '30'}
                onChange={(e) => updateSetting('scheduler_interval_secs', e.target.value)}
                className="w-20 px-2 py-1 text-sm border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-white text-center"
              />
            </div>

            <div className="flex items-center justify-between">
              <div>
                <p className="font-medium text-gray-700 dark:text-gray-300">Aviso anticipado</p>
                <p className="text-xs text-gray-500 dark:text-gray-400">Minutos antes para notificar</p>
              </div>
              <div className="flex items-center gap-3">
                <input
                  type="range"
                  min={0}
                  max={30}
                  value={getSettingValue('notify_before_minutes') || '2'}
                  onChange={(e) => updateSetting('notify_before_minutes', e.target.value)}
                  className="w-24"
                />
                <span className="text-sm font-mono text-gray-700 dark:text-gray-300 w-8 text-center">
                  {getSettingValue('notify_before_minutes') || '2'}m
                </span>
              </div>
            </div>

            <div className="flex items-center justify-between">
              <div>
                <p className="font-medium text-gray-700 dark:text-gray-300">Probar notificación</p>
                <p className="text-xs text-gray-500 dark:text-gray-400">Envía una notificación de prueba</p>
              </div>
              <button
                onClick={() => testNotification()}
                className="px-3 py-1 bg-blue-600 text-white rounded hover:bg-blue-700 text-sm"
              >
                Probar
              </button>
            </div>
          </div>
        </section>

        {/* Transcripción */}
        <section className="bg-white dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700">
          <h2 className="text-lg font-semibold text-gray-800 dark:text-white mb-4">Transcripción de voz</h2>
          <ModelDownloader />
        </section>

        {/* Atajos */}
        <section className="bg-white dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700">
          <h2 className="text-lg font-semibold text-gray-800 dark:text-white mb-4">Atajos de teclado</h2>
          <p className="text-xs text-gray-500 dark:text-gray-400 mb-3">Haz clic en un atajo para cambiarlo</p>
          <div className="divide-y divide-gray-200 dark:divide-gray-700">
            {shortcuts.map((sc) => (
              <ShortcutEditor
                key={sc.id}
                shortcut={sc}
                allShortcuts={shortcuts}
                onUpdate={handleShortcutUpdate}
                onToggle={handleShortcutToggle}
              />
            ))}
          </div>
        </section>

        {/* Información */}
        <section className="bg-white dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700">
          <h2 className="text-lg font-semibold text-gray-800 dark:text-white mb-4">Información</h2>
          <div className="text-sm text-gray-600 dark:text-gray-400 space-y-1">
            <p>Versión: 0.1.0</p>
            <p>Base de datos: SQLite local</p>
            <p>Transcripción: Whisper local (offline)</p>
          </div>
        </section>
      </div>
    </div>
  );
}
