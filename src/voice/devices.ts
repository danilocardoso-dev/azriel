/** Escolha de microfone e saída para a chamada de voz. Funções puras para poder testar sem navegador. */

export interface AudioDevice { deviceId: string; groupId: string; kind: string; label: string }

// Dispositivos que aparecem como microfone mas só entregam silêncio ou o som do próprio PC.
const VIRTUAL = /steam streaming|virtual|vb-audio|cable output|voicemeeter|stereo mix|mixagem est[eé]reo|what u hear/i;

export const isVirtual = (label: string) => VIRTUAL.test(label);

/** Microfone de verdade: evita os virtuais e prefere headset (é onde a pessoa está falando). */
export function chooseInput(devices: AudioDevice[]): AudioDevice | null {
  const inputs = devices.filter((device) => device.kind === "audioinput" && device.deviceId && !["default", "communications"].includes(device.deviceId));
  const real = inputs.filter((device) => !isVirtual(device.label));
  return real.find((device) => /headset|fone|hands-free|headphone/i.test(device.label)) ?? real[0] ?? null;
}

/**
 * Saída do mesmo aparelho do microfone. No Bluetooth, abrir o microfone põe o fone em modo
 * "Hands-Free" e a saída "Stereo" emudece; a voz do Az precisa ir para o endpoint Hands-Free.
 */
export function pairedOutput(devices: AudioDevice[], input: AudioDevice | null): AudioDevice | null {
  if (!input) return null;
  const outputs = devices.filter((device) => device.kind === "audiooutput" && device.deviceId && !["default", "communications"].includes(device.deviceId));
  const same = outputs.filter((device) => device.groupId && device.groupId === input.groupId);
  return same.find((device) => /hands-free/i.test(device.label)) ?? same[0] ?? null;
}
