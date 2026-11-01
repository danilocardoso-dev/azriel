import { beforeEach, describe, expect, it, vi } from "vitest";
import type { SpotifyPlayer } from "../repositories/spotifyRepository";

const transfer = vi.fn().mockResolvedValue(undefined);
const control = vi.fn().mockResolvedValue(undefined);
vi.mock("../repositories/spotifyRepository", () => ({ spotifyRepository: { transfer: (...args: unknown[]) => transfer(...args), control: (...args: unknown[]) => control(...args) } }));

const { shouldAdoptAzriel, spotifyControl } = await import("./controls");

const player = (device: string, isPlaying: boolean): SpotifyPlayer => ({ isPlaying, progressMs: 0, shuffle: false, device: { name: device, type: "Computer", volume: 50 }, item: { uri: "spotify:track:1", name: "x", subtitle: "", image: null, durationMs: 1 } });

beforeEach(() => { transfer.mockClear(); control.mockClear(); });

describe("Azriel como saída padrão", () => {
  it("assume a saída só quando está pronto, não é o ativo e ninguém está tocando", () => {
    expect(shouldAdoptAzriel(null, true, "dev1")).toBe(true);
    expect(shouldAdoptAzriel(player("DESKTOP", false), true, "dev1")).toBe(true);
    expect(shouldAdoptAzriel(player("DESKTOP", true), true, "dev1")).toBe(false);
    expect(shouldAdoptAzriel(player("Azriel", false), true, "dev1")).toBe(false);
    expect(shouldAdoptAzriel(null, false, "dev1")).toBe(false);
    expect(shouldAdoptAzriel(null, true, null)).toBe(false);
  });

  it("tocar vai para o Azriel quando não há dispositivo ativo ou o ativo é outro", async () => {
    await spotifyControl("play", undefined, { active: false, deviceId: "dev1" });
    await spotifyControl("play", undefined, { active: true, activeDevice: "DESKTOP-PC", deviceId: "dev1" });
    expect(transfer).toHaveBeenCalledTimes(2);
    expect(transfer).toHaveBeenCalledWith("dev1", true);
    expect(control).not.toHaveBeenCalled();
  });

  it("com o Azriel já ativo, ou sem player embutido, usa o comando normal", async () => {
    await spotifyControl("play", undefined, { active: true, activeDevice: "Azriel", deviceId: "dev1" });
    await spotifyControl("play", undefined, { active: false, deviceId: null });
    await spotifyControl("next", undefined, { active: false, deviceId: "dev1" });
    await spotifyControl("pause", undefined, { active: true, activeDevice: "DESKTOP-PC", deviceId: "dev1" });
    expect(transfer).not.toHaveBeenCalled();
    expect(control).toHaveBeenCalledTimes(4);
  });
});
