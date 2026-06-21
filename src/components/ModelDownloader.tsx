import { useEffect, useRef, useState } from 'react';
import { getModelStatus, getModelInfo, downloadModel } from '../services/transcriptionService';
import { useAppStore } from '../store/appStore';
import type { ModelInfo } from '../services/transcriptionService';

const VARIANTS: { key: string; label: string; size: string }[] = [
  { key: 'tiny', label: 'Tiny', size: '75 MB' },
  { key: 'base', label: 'Base', size: '142 MB' },
  { key: 'small', label: 'Small (recomendado)', size: '466 MB' },
  { key: 'large-v3-turbo', label: 'Turbo', size: '1.5 GB' },
];

export function ModelDownloader() {
  const { modelStatus, setModelStatus } = useAppStore();
  const [modelInfo, setModelInfo] = useState<ModelInfo | null>(null);
  const [selectedVariant, setSelectedVariant] = useState('small');
  const pollingRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    loadStatus();
    return () => {
      if (pollingRef.current) clearInterval(pollingRef.current);
    };
  }, [setModelStatus]);

  useEffect(() => {
    const isDownloading =
      typeof modelStatus === 'object' &&
      modelStatus !== null &&
      !Array.isArray(modelStatus) &&
      'Downloading' in modelStatus;

    if (isDownloading) {
      if (!pollingRef.current) {
        pollingRef.current = setInterval(loadStatus, 1000);
      }
    } else {
      if (pollingRef.current) {
        clearInterval(pollingRef.current);
        pollingRef.current = null;
      }
    }
  }, [modelStatus]);

  const loadStatus = async () => {
    try {
      const [status, info] = await Promise.all([getModelStatus(), getModelInfo()]);
      setModelStatus(status);
      setModelInfo(info);
      setSelectedVariant(info.variant);
    } catch {
      try {
        const status = await getModelStatus();
        setModelStatus(status);
      } catch { /* ignore */ }
    }
  };

  const handleDownload = async () => {
    try {
      await downloadModel(selectedVariant);
      loadStatus();
    } catch (e) {
      setModelStatus({ Error: String(e) });
    }
  };

  const isReady = modelStatus === 'Ready';
  const progress =
    typeof modelStatus === 'object' &&
    modelStatus !== null &&
    !Array.isArray(modelStatus) &&
    'Downloading' in modelStatus
      ? (modelStatus as { Downloading: number }).Downloading
      : null;
  const isError =
    typeof modelStatus === 'object' &&
    modelStatus !== null &&
    !Array.isArray(modelStatus) &&
    'Error' in modelStatus;
  const errMsg = isError ? (modelStatus as { Error: string }).Error : '';
  const currentVariant = modelInfo?.variant;
  const isDownloadedVariant = isReady && currentVariant === selectedVariant;

  return (
    <div className="space-y-4">
      <div className="space-y-2">
        <p className="text-sm font-medium text-gray-700 dark:text-gray-300">
          Modelo Whisper
        </p>
        <div className="space-y-1">
          {VARIANTS.map((v) => (
            <label
              key={v.key}
              className={`flex items-center gap-3 px-3 py-2 rounded-lg border cursor-pointer transition-colors ${
                selectedVariant === v.key
                  ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
                  : 'border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800'
              } ${isReady && currentVariant === v.key ? 'border-green-400' : ''}`}
            >
              <input
                type="radio"
                name="whisper-variant"
                value={v.key}
                checked={selectedVariant === v.key}
                onChange={() => setSelectedVariant(v.key)}
                className="accent-blue-600"
                disabled={progress !== null}
              />
              <div className="flex-1">
                <span className="text-sm font-medium text-gray-800 dark:text-gray-200">
                  {v.label}
                </span>
                <span className="text-xs text-gray-500 dark:text-gray-400 ml-2">
                  {v.size}
                </span>
              </div>
              {isReady && currentVariant === v.key && (
                <span className="text-xs text-green-600 dark:text-green-400">✓ Listo</span>
              )}
            </label>
          ))}
        </div>
      </div>

      {progress !== null && (
        <div className="space-y-2">
          <div className="flex items-center gap-2 text-sm text-blue-600 dark:text-blue-400">
            <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-blue-600" />
            Descargando {selectedVariant}... {progress.toFixed(0)}%
          </div>
          <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
            <div
              className="bg-blue-600 h-2 rounded-full transition-all"
              style={{ width: `${progress}%` }}
            />
          </div>
          <p className="text-xs text-gray-500 dark:text-gray-400">
            {VARIANTS.find((v) => v.key === selectedVariant)?.size}. No cierres la aplicación.
          </p>
        </div>
      )}

      {isError && (
        <div className="space-y-2">
          <p className="text-sm text-orange-600 dark:text-orange-400">{errMsg}</p>
        </div>
      )}

      {progress === null && !isError && (
        <button
          onClick={handleDownload}
          disabled={isDownloadedVariant}
          className={`w-full px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
            isDownloadedVariant
              ? 'bg-gray-200 dark:bg-gray-700 text-gray-400 dark:text-gray-500 cursor-default'
              : 'bg-blue-600 text-white hover:bg-blue-700'
          }`}
        >
          {isDownloadedVariant
            ? `${currentVariant?.toUpperCase()} descargado`
            : `Descargar ${selectedVariant.toUpperCase()}`}
        </button>
      )}

      {progress === null && isError && (
        <button
          onClick={handleDownload}
          className="w-full px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 text-sm font-medium"
        >
          Reintentar descarga
        </button>
      )}

      {isReady && (
        <p className="text-xs text-gray-500 dark:text-gray-400">
          Modelo {currentVariant?.toUpperCase()} cargado. Transcripción offline 100% en español.
        </p>
      )}
    </div>
  );
}
