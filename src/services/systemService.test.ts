import { describe, expect, it } from "vitest";
import { filterAndSortProcesses, formatBytes, formatUptime } from "./systemService";

const processes = [
  { pid: 20, name: "Ollama", cpuPercent: 3, memoryBytes: 500 },
  { pid: 10, name: "Azriel", cpuPercent: 8, memoryBytes: 200 },
];

describe("System Service", () => {
  it("formata o tempo ativo do dispositivo", () => {
    expect(formatUptime(90)).toBe("1m");
    expect(formatUptime(7380)).toBe("2h 3m");
    expect(formatUptime(183900)).toBe("2d 3h 5m");
  });
  it("formata bytes sem confundir zero com indisponível", () => {
    expect(formatBytes(0)).toBe("0 B");
    expect(formatBytes(1024 ** 3)).toBe("1.0 GB");
  });
  it("filtra por nome e PID e ordena sem alterar a origem", () => {
    expect(filterAndSortProcesses(processes, "olla", "memory")[0].name).toBe("Ollama");
    expect(filterAndSortProcesses(processes, "10", "cpu")[0].name).toBe("Azriel");
    expect(filterAndSortProcesses(processes, "", "cpu")[0].cpuPercent).toBe(8);
    expect(processes[0].name).toBe("Ollama");
  });
});
