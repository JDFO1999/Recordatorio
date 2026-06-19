import { useState, useRef, useCallback } from 'react';
import { useAppStore } from '../store/appStore';
import { useTranscription } from '../hooks/useTranscription';

interface VoiceRecorderProps {
  onRecordingComplete: () => void;
}

export function VoiceRecorder({ onRecordingComplete }: VoiceRecorderProps) {
  const { transcriptionState, setTranscriptionState } = useAppStore();
  const [recording, setRecording] = useState(false);
  const mediaRecorderRef = useRef<MediaRecorder | null>(null);
  const chunksRef = useRef<Blob[]>([]);
  const { processAudioBlob } = useTranscription();

  const startRecording = useCallback(async () => {
    try {
      if (!navigator.mediaDevices?.getUserMedia) {
        setTranscriptionState({
          status: 'error',
          error: 'Tu navegador no soporta grabación de audio.',
        });
        return;
      }

      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      const mimeType = MediaRecorder.isTypeSupported('audio/webm;codecs=opus')
        ? 'audio/webm;codecs=opus'
        : 'audio/webm';

      const mediaRecorder = new MediaRecorder(stream, { mimeType });
      mediaRecorderRef.current = mediaRecorder;
      chunksRef.current = [];

      mediaRecorder.ondataavailable = (e) => {
        if (e.data.size > 0) chunksRef.current.push(e.data);
      };

      mediaRecorder.onstop = async () => {
        stream.getTracks().forEach((t) => t.stop());
        const blob = new Blob(chunksRef.current, { type: mimeType });

        if (blob.size < 1000) {
          setTranscriptionState({
            status: 'error',
            error: 'El audio grabado está vacío. Habla más cerca del micrófono.',
          });
          return;
        }

        await processAudioBlob(blob);
        onRecordingComplete();
      };

      mediaRecorder.onerror = () => {
        setTranscriptionState({
          status: 'error',
          error: 'Error durante la grabación. Intenta de nuevo.',
        });
      };

      mediaRecorder.start();
      setRecording(true);
      setTranscriptionState({ status: 'recording' });
    } catch (e) {
      const message =
        e instanceof DOMException && e.name === 'NotAllowedError'
          ? 'Permiso de micrófono denegado. Habilita el micrófono en la configuración del sistema.'
          : e instanceof Error
            ? e.message
            : 'No se pudo acceder al micrófono.';
      setTranscriptionState({ status: 'error', error: message });
    }
  }, [processAudioBlob, onRecordingComplete, setTranscriptionState]);

  const stopRecording = useCallback(() => {
    if (mediaRecorderRef.current && mediaRecorderRef.current.state === 'recording') {
      mediaRecorderRef.current.stop();
      setRecording(false);
    }
  }, []);

  const isProcessing = transcriptionState.status === 'transcribing';
  const hasError = transcriptionState.status === 'error';

  return (
    <div className="mb-6 p-4 bg-gray-50 dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
      <div className="flex items-center gap-4">
        <button
          onClick={recording ? stopRecording : startRecording}
          disabled={isProcessing}
          className={`w-16 h-16 rounded-full flex items-center justify-center text-white text-2xl transition-all ${
            isProcessing
              ? 'bg-yellow-500 cursor-wait'
              : recording
                ? 'bg-red-500 animate-pulse shadow-lg shadow-red-500/50'
                : 'bg-blue-600 hover:bg-blue-700'
          } disabled:opacity-70`}
          title={
            isProcessing
              ? 'Transcribiendo...'
              : recording
                ? 'Detener grabación'
                : 'Iniciar grabación'
          }
        >
          {isProcessing ? '⏳' : recording ? '⏹' : '🎤'}
        </button>
        <div className="flex-1">
          {isProcessing ? (
            <div className="flex items-center gap-2">
              <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-blue-600" />
              <p className="text-sm font-medium text-gray-700 dark:text-gray-300">
                Transcribiendo audio...
              </p>
            </div>
          ) : hasError ? (
            <div>
              <p className="text-sm font-medium text-red-600 dark:text-red-400">
                {transcriptionState.error}
              </p>
              <button
                onClick={() => setTranscriptionState({ status: 'idle' })}
                className="text-xs text-blue-600 hover:underline mt-1"
              >
                Reintentar
              </button>
            </div>
          ) : (
            <>
              <p className="text-sm font-medium text-gray-700 dark:text-gray-300">
                {recording
                  ? 'Grabando... di tu recordatorio'
                  : 'Presiona para grabar'}
              </p>
              <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                Ej: "Recuérdame llamar a Juan en 30 minutos"
              </p>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
