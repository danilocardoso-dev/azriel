import { describe, expect, it } from "vitest";
import { chooseInput, pairedOutput, type AudioDevice } from "./devices";

const device = (kind: string, deviceId: string, label: string, groupId = ""): AudioDevice => ({ kind, deviceId, label, groupId });

const devices = [
  device("audioinput", "default", "Padrão - Microfone (Steam Streaming Microphone)", "steam"),
  device("audioinput", "steam", "Microfone (Steam Streaming Microphone)", "steam"),
  device("audioinput", "y36", "Headset (Y36 Hands-Free AG Audio)", "y36"),
  device("audiooutput", "y36-stereo", "Fones de ouvido (Y36 Stereo)", "y36"),
  device("audiooutput", "y36-hf", "Headset (Y36 Hands-Free AG Audio)", "y36"),
  device("audiooutput", "hdmi", "HDMI (Áudio Intel(R) para telas)", "hdmi"),
];

describe("dispositivos da chamada de voz", () => {
  it("ignora o microfone virtual do Steam e pega o headset", () => {
    expect(chooseInput(devices)?.deviceId).toBe("y36");
  });

  it("sem microfone de verdade, não escolhe nada", () => {
    expect(chooseInput(devices.filter((item) => item.deviceId !== "y36"))).toBeNull();
  });

  it("manda a voz do Az para o Hands-Free do mesmo fone", () => {
    expect(pairedOutput(devices, chooseInput(devices))?.deviceId).toBe("y36-hf");
  });

  it("microfone sem saída do mesmo aparelho fica na saída padrão", () => {
    expect(pairedOutput(devices, device("audioinput", "usb", "Microfone USB", "usb"))).toBeNull();
  });
});
