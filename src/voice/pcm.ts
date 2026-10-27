// Conversões entre o áudio do navegador (Float32 −1..1) e o PCM 16-bit little-endian do Gemini Live.

export function floatToPcm16(samples: Float32Array): Int16Array {
  const pcm = new Int16Array(samples.length);
  for (let i = 0; i < samples.length; i += 1) {
    const value = Math.max(-1, Math.min(1, samples[i]));
    pcm[i] = value < 0 ? Math.round(value * 0x8000) : Math.round(value * 0x7fff);
  }
  return pcm;
}

export function pcm16ToFloat(pcm: Int16Array): Float32Array<ArrayBuffer> {
  const samples = new Float32Array(pcm.length);
  for (let i = 0; i < pcm.length; i += 1) samples[i] = pcm[i] / (pcm[i] < 0 ? 0x8000 : 0x7fff);
  return samples;
}

export function bytesToBase64(bytes: Uint8Array): string {
  let binary = "";
  for (let i = 0; i < bytes.length; i += 0x8000) binary += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  return btoa(binary);
}

export function base64ToBytes(base64: string): Uint8Array {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) bytes[i] = binary.charCodeAt(i);
  return bytes;
}

/** PCM do Gemini (little-endian) → Int16Array, sem depender da endianness da máquina. */
export function bytesToPcm16(bytes: Uint8Array): Int16Array {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const pcm = new Int16Array(Math.floor(bytes.byteLength / 2));
  for (let i = 0; i < pcm.length; i += 1) pcm[i] = view.getInt16(i * 2, true);
  return pcm;
}

export function pcm16ToBytes(pcm: Int16Array): Uint8Array {
  const bytes = new Uint8Array(pcm.length * 2);
  const view = new DataView(bytes.buffer);
  for (let i = 0; i < pcm.length; i += 1) view.setInt16(i * 2, pcm[i], true);
  return bytes;
}

/** Nível 0..1 para animar o núcleo (RMS com ganho, para a fala ocupar a faixa útil). */
export function level(samples: Float32Array): number {
  if (!samples.length) return 0;
  let sum = 0;
  for (let i = 0; i < samples.length; i += 1) sum += samples[i] * samples[i];
  return Math.min(1, Math.sqrt(sum / samples.length) * 4);
}
