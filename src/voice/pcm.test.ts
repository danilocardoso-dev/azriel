import { describe, expect, it } from "vitest";
import { base64ToBytes, bytesToBase64, bytesToPcm16, floatToPcm16, level, pcm16ToBytes, pcm16ToFloat } from "./pcm";

describe("PCM do Gemini Live", () => {
  it("converte float em PCM 16-bit com saturação", () => {
    expect(Array.from(floatToPcm16(new Float32Array([0, 1, -1, 2, -2, 0.5])))).toEqual([0, 32767, -32768, 32767, -32768, 16384]);
  });

  it("ida e volta preserva o sinal", () => {
    const samples = new Float32Array([0, 0.25, -0.25, 0.9]);
    const back = pcm16ToFloat(floatToPcm16(samples));
    back.forEach((value, index) => expect(value).toBeCloseTo(samples[index], 3));
  });

  it("lê e grava little-endian e base64 sem perder bytes", () => {
    const pcm = new Int16Array([1, -2, 300, -32768]);
    const bytes = pcm16ToBytes(pcm);
    expect(Array.from(bytes.slice(0, 4))).toEqual([1, 0, 254, 255]);
    expect(Array.from(bytesToPcm16(base64ToBytes(bytesToBase64(bytes))))).toEqual([1, -2, 300, -32768]);
  });

  it("mede silêncio e fala para animar o núcleo", () => {
    expect(level(new Float32Array(160))).toBe(0);
    expect(level(new Float32Array(160).fill(0.1))).toBeCloseTo(0.4, 5);
    expect(level(new Float32Array(160).fill(0.9))).toBe(1);
  });
});
