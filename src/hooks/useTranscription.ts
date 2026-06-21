import { useCallback } from 'react';
import { useAppStore } from '../store/appStore';
import { getModelStatus, transcribeAudio } from '../services/transcriptionService';
import { saveAudioFile, deleteAudioFile, blobToWavBlob } from '../services/audioService';

export function useTranscription() {
  const { setTranscriptionState, setPendingTranscriptionText, setCurrentView } = useAppStore();

  const ensureModelReady = useCallback(async (): Promise<boolean> => {
    try {
      const status = await getModelStatus();
      if (status === 'Ready') return true;
      if (status === 'NotDownloaded') return true;
      if (typeof status === 'object' && 'Downloading' in status) {
        setTranscriptionState({ status: 'error', error: 'El modelo se está descargando. Espera a que termine.' });
        return false;
      }
      return true;
    } catch {
      return true;
    }
  }, [setTranscriptionState]);

  const processAudioBlob = useCallback(async (blob: Blob) => {
    const ready = await ensureModelReady();
    if (!ready) return;

    setTranscriptionState({ status: 'transcribing' });

    let audioPath = '';
    try {
      const wavBlob = await blobToWavBlob(blob);
      audioPath = await saveAudioFile(wavBlob);
      const text = await transcribeAudio(audioPath);

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
    } finally {
      if (audioPath) {
        await deleteAudioFile(audioPath);
      }
    }
  }, [setTranscriptionState, setPendingTranscriptionText, setCurrentView, ensureModelReady]);

  return { processAudioBlob, ensureModelReady };
}
