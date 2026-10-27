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

export class Microphone {
  private context: AudioContext | null = null;
  private stream: MediaStream | null = null;
  muted = false;

  async start(onChunk: (base64: string) => void, onLevel: (value: number) => void) {
    this.stream = await navigator.mediaDevices.getUserMedia({ audio: { channelCount: 1, echoCancellation: true, noiseSuppression: true, autoGainControl: true } });
    // O navegador reamostra o microfone para 16 kHz, a taxa que o Gemini Live espera.
    this.context = new AudioContext({ sampleRate: INPUT_RATE });
    const url = URL.createObjectURL(new Blob([WORKLET], { type: "application/javascript" }));
    try { await this.context.audioWorklet.addModule(url); } finally { URL.revokeObjectURL(url); }
    const source = this.context.createMediaStreamSource(this.stream);
    const node = new AudioWorkletNode(this.context, "azriel-mic");
    node.port.onmessage = (event: MessageEvent<Float32Array>) => {
      if (this.muted) { onLevel(0); return; }
      onLevel(level(event.data));
      onChunk(bytesToBase64(pcm16ToBytes(floatToPcm16(event.data))));
    };
    source.connect(node);
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
