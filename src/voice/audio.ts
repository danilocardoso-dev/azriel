import { chooseInput, isVirtual, pairedOutput } from "./devices";
import { bytesToBase64, bytesToPcm16, base64ToBytes, floatToPcm16, level, pcm16ToBytes, pcm16ToFloat } from "./pcm";

const INPUT_RATE = 16_000;
const OUTPUT_RATE = 24_000;
const CHUNK_SAMPLES = 1_600; // 100 ms

// Worklet inline: junta blocos de 100 ms do microfone e devolve para a thread principal.
const WORKLET = `
class AzrielMic extends AudioWorkletProcessor {
  constructor() { super(); this.buffer = new Float32Array(${CHUNK_SAMPLES}); this.size = 0; }
  process(inputs) {
    const input = inputs[0] && inputs[0][0];
    if (!input) return true;
    for (let i = 0; i < input.length; i++) {
      this.buffer[this.size++] = input[i];
      if (this.size === this.buffer.length) { this.port.postMessage(this.buffer.slice(0)); this.size = 0; }
    }
    return true;
  }
}
registerProcessor("azriel-mic", AzrielMic);
`;

const SILENCE_CHUNKS = 60; // 6 s sem nenhum sinal = microfone mudo
const SILENCE_LEVEL = 0.002;

const constraints = (deviceId?: string): MediaStreamConstraints => ({
  audio: { ...(deviceId ? { deviceId: { exact: deviceId } } : {}), channelCount: 1, echoCancellation: true, noiseSuppression: true, autoGainControl: true },
});

export interface MicrophoneInfo { label: string; outputId: string | null }

export class Microphone {
  private context: AudioContext | null = null;
  private stream: MediaStream | null = null;
  muted = false;

  /** Abre o padrão (para liberar os nomes dos aparelhos) e troca por um microfone de verdade se o padrão for virtual. */
  private async open(): Promise<MicrophoneInfo> {
    this.stream = await navigator.mediaDevices.getUserMedia(constraints());
    const devices = await navigator.mediaDevices.enumerateDevices();
    const current = this.stream.getAudioTracks()[0];
    const currentId = current?.getSettings().deviceId;
    const chosen = isVirtual(current?.label ?? "") || !currentId ? chooseInput(devices) : devices.find((device) => device.kind === "audioinput" && device.deviceId === currentId) ?? null;
    if (chosen && chosen.deviceId !== currentId) {
      this.stream.getTracks().forEach((track) => track.stop());
      this.stream = await navigator.mediaDevices.getUserMedia(constraints(chosen.deviceId));
    }
    const label = this.stream.getAudioTracks()[0]?.label || chosen?.label || "microfone";
    return { label, outputId: pairedOutput(devices, chosen)?.deviceId ?? null };
  }

  async start(onChunk: (base64: string) => void, onLevel: (value: number) => void, onSilent: (label: string) => void): Promise<MicrophoneInfo> {
    const info = await this.open();
    // O navegador reamostra o microfone para 16 kHz, a taxa que o Gemini Live espera.
    this.context = new AudioContext({ sampleRate: INPUT_RATE });
    // Criado fora do clique, o contexto pode nascer suspenso e o worklet nunca roda.
    if (this.context.state === "suspended") await this.context.resume().catch(() => undefined);
    const url = URL.createObjectURL(new Blob([WORKLET], { type: "application/javascript" }));
    try { await this.context.audioWorklet.addModule(url); } finally { URL.revokeObjectURL(url); }
    const source = this.context.createMediaStreamSource(this.stream!);
    const node = new AudioWorkletNode(this.context, "azriel-mic");
    let chunks = 0;
    let peak = 0;
    node.port.onmessage = (event: MessageEvent<Float32Array>) => {
      const value = level(event.data);
      if (chunks <= SILENCE_CHUNKS) {
        peak = Math.max(peak, value);
        if (++chunks === SILENCE_CHUNKS && peak < SILENCE_LEVEL && !this.muted) onSilent(info.label);
      }
      if (this.muted) { onLevel(0); return; }
      onLevel(value);
      onChunk(bytesToBase64(pcm16ToBytes(floatToPcm16(event.data))));
    };
    source.connect(node);
    return info;
  }

  stop() {
    this.stream?.getTracks().forEach((track) => track.stop());
    void this.context?.close().catch(() => undefined);
    this.stream = null;
    this.context = null;
  }
}

/** Fila de reprodução da voz do Az; `interrupt()` corta na hora quando o operador fala por cima. */
export class Speaker {
  private context = new AudioContext({ sampleRate: OUTPUT_RATE });
  private analyser = this.context.createAnalyser();
  private sources = new Set<AudioBufferSourceNode>();
  private nextTime = 0;
  private samples = new Float32Array(512);

  constructor() {
    this.analyser.fftSize = 512;
    this.analyser.connect(this.context.destination);
  }

  play(base64: string) {
    if (this.context.state === "suspended") void this.context.resume();
    const samples = pcm16ToFloat(bytesToPcm16(base64ToBytes(base64)));
    if (!samples.length) return;
    const buffer = this.context.createBuffer(1, samples.length, OUTPUT_RATE);
    buffer.copyToChannel(samples, 0);
    const source = this.context.createBufferSource();
    source.buffer = buffer;
    source.connect(this.analyser);
    const startAt = Math.max(this.context.currentTime + 0.02, this.nextTime);
    source.start(startAt);
    this.nextTime = startAt + buffer.duration;
    this.sources.add(source);
    source.onended = () => this.sources.delete(source);
  }

  /** Toca no aparelho indicado (ex.: Hands-Free do fone Bluetooth). Sem suporte, fica na saída padrão. */
  async useOutput(deviceId: string) {
    const context = this.context as AudioContext & { setSinkId?: (id: string) => Promise<void> };
    await context.setSinkId?.(deviceId).catch(() => undefined);
  }

  get speaking() { return this.sources.size > 0; }

  level() {
    this.analyser.getFloatTimeDomainData(this.samples);
    return level(this.samples);
  }

  interrupt() {
    this.sources.forEach((source) => { try { source.stop(); } catch { /* já terminou */ } });
    this.sources.clear();
    this.nextTime = 0;
  }

  close() {
    this.interrupt();
    void this.context.close().catch(() => undefined);
  }
}
