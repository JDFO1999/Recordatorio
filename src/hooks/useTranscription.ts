import { useCallback } from 'react';
import { useAppStore } from '../store/appStore';
import { getModelStatus, transcribeAudio } from '../services/transcriptionService';
import { saveAudioFile, deleteAudioFile, blobToWavBlob } from '../services/audioService';

export function useTranscription() {
  const { setTranscriptionState, setPendingTranscriptionText, setCurrentView } = useAppStore();

  const ensureModelReady = useCallback(async (): Promise<boolean> => {
    const status = await getModelStatus();
    if (status === 'Ready') return true;
    if (status === 'NotDownloaded') {
      setTranscriptionState({ status: 'error', error: 'El modelo Whisper no está descargado. Ve a Configuración para descargarlo.' });
      return false;
    }
    if (typeof status === 'object' && 'Downloading' in status) {
      setTranscriptionState({ status: 'error', error: 'El modelo se está descargando. Espera a que termine.' });
      return false;
    }
    setTranscriptionState({ status: 'error', error: 'Error con el modelo de transcripción.' });
    return false;
  }, [setTranscriptionState]);

  const processAudioBlob = useCallback(async (blob: Blob) => {
    try {
      const ready = await ensureModelReady();
      if (!ready) return;

      setTranscriptionState({ status: 'transcribing' });

      const wavBlob = await blobToWavBlob(blob);
      const audioPath = await saveAudioFile(wavBlob);
      const text = await transcribeAudio(audioPath);

      await deleteAudioFile(audioPath);

      if (!text || text.trim().length === 0) {
        setTranscriptionState({ status: 'error', error: 'No se detectó voz en el audio. Intenta de nuevo.' });
        return;
      }

      setPendingTranscriptionText(text.trim());
      setTranscriptionState({ status: 'done' });
      setCurrentView('confirm');
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setTranscriptionState({ status: 'error', error: message });
    }
  }, [setTranscriptionState, setPendingTranscriptionText, setCurrentView, ensureModelReady]);

  return { processAudioBlob, ensureModelReady };
}
