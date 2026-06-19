import { useEffect } from 'react';
import { getModelStatus } from '../services/transcriptionService';
import { useAppStore } from '../store/appStore';

export function ModelDownloader() {
  const { modelStatus, setModelStatus } = useAppStore();

  useEffect(() => {
    loadStatus();
  }, [setModelStatus]);

  const loadStatus = async () => {
    try {
      const status = await getModelStatus();
      setModelStatus(status);
    } catch {
      // ignore
    }
  };

  if (modelStatus === 'Ready') {
    return (
      <div className="flex items-center gap-2 text-sm text-green-600 dark:text-green-400">
        <span className="w-2 h-2 rounded-full bg-green-500" />
        Transcripción de voz disponible
      </div>
    );
  }

  if (typeof modelStatus === 'object' && 'Error' in modelStatus) {
    return (
      <div className="space-y-2">
        <p className="text-sm text-orange-600 dark:text-orange-400">
          {modelStatus.Error}
        </p>
        <p className="text-xs text-gray-500 dark:text-gray-400">
          La transcripción sigue funcionando con el reconocedor de voz integrado de Windows (SAPI).
          No necesita descarga.
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-2">
      <p className="text-sm text-gray-600 dark:text-gray-400">
        Transcripción: se usa el reconocedor de voz integrado de Windows (SAPI). Funciona offline y en español.
      </p>
      <p className="text-xs text-gray-400 dark:text-gray-500">
        Whisper (~1.5 GB, requiere internet) está disponible como alternativa pero no es necesario.
      </p>
    </div>
  );
}
