import { invoke } from '@tauri-apps/api/core';
import { appDataDir, join } from '@tauri-apps/api/path';

export async function saveAudioFile(blob: Blob): Promise<string> {
  const dir = await appDataDir();
  const filename = `recording_${Date.now()}.wav`;
  const filePath = await join(dir, filename);

  const arrayBuffer = await blob.arrayBuffer();
  const uint8Array = new Uint8Array(arrayBuffer);

  await invoke('save_file', { path: filePath, data: Array.from(uint8Array) });
  return filePath;
}

export async function deleteAudioFile(path: string): Promise<void> {
  try {
    await invoke('delete_file', { path });
  } catch {
    // ignore cleanup errors
  }
}

export async function blobToWavBlob(blob: Blob): Promise<Blob> {
  const arrayBuffer = await blob.arrayBuffer();
  const audioCtx = new AudioContext();
  try {
    const audioBuffer = await audioCtx.decodeAudioData(arrayBuffer);

    const targetSampleRate = 16000;
    const numInputSamples = audioBuffer.length;
    const inputSampleRate = audioBuffer.sampleRate;

    const duration = numInputSamples / inputSampleRate;
    const numOutputSamples = Math.round(duration * targetSampleRate);

    const inputData = audioBuffer.getChannelData(0);
    const outputData = new Float32Array(numOutputSamples);

    if (inputSampleRate === targetSampleRate) {
      outputData.set(inputData.subarray(0, numOutputSamples));
    } else {
      const ratio = inputSampleRate / targetSampleRate;
      for (let i = 0; i < numOutputSamples; i++) {
        const pos = i * ratio;
        const idx = Math.floor(pos);
        const frac = pos - idx;
        if (idx + 1 < numInputSamples) {
          outputData[i] = inputData[idx] * (1 - frac) + inputData[idx + 1] * frac;
        } else {
          outputData[i] = inputData[Math.min(idx, numInputSamples - 1)];
        }
      }
    }

    return encodeWav(outputData, targetSampleRate);
  } finally {
    audioCtx.close();
  }
}

function encodeWav(samples: Float32Array, sampleRate: number): Blob {
  const bitsPerSample = 16;
  const numChannels = 1;
  const bytesPerSample = bitsPerSample / 8;
  const dataLength = samples.length * bytesPerSample;
  const headerLength = 44;
  const totalLength = headerLength + dataLength;

  const buffer = new ArrayBuffer(totalLength);
  const view = new DataView(buffer);

  writeStr(view, 0, 'RIFF');
  view.setUint32(4, totalLength - 8, true);
  writeStr(view, 8, 'WAVE');
  writeStr(view, 12, 'fmt ');
  view.setUint32(16, 16, true);
  view.setUint16(20, 1, true);
  view.setUint16(22, numChannels, true);
  view.setUint32(24, sampleRate, true);
  view.setUint32(28, sampleRate * numChannels * bytesPerSample, true);
  view.setUint16(32, numChannels * bytesPerSample, true);
  view.setUint16(34, bitsPerSample, true);
  writeStr(view, 36, 'data');
  view.setUint32(40, dataLength, true);

  let offset = 44;
  for (let i = 0; i < samples.length; i++) {
    const s = Math.max(-1, Math.min(1, samples[i]));
    view.setInt16(offset, s < 0 ? s * 0x8000 : s * 0x7FFF, true);
    offset += 2;
  }

  return new Blob([buffer], { type: 'audio/wav' });
}

function writeStr(view: DataView, offset: number, str: string) {
  for (let i = 0; i < str.length; i++) {
    view.setUint8(offset + i, str.charCodeAt(i));
  }
}
