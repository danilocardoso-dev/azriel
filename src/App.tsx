import { useCallback, useEffect, useState } from "react";
import { getCurrentWindow, Window } from "@tauri-apps/api/window";
import { azrielStates, modules } from "./data/system";
import { NucleoPage } from "./pages/NucleoPage";
import { SettingsPage } from "./pages/SettingsPage";
import { DailyOperationsPage } from "./pages/DailyOperationsPage";
import { AICorePage } from "./pages/AICorePage";
import { MusicPage } from "./pages/MusicPage";
import { MoneyPage } from "./pages/MoneyPage";
import { useSpotifyWebPlayer } from "./music/useSpotifyWebPlayer";
import { spotifyRepository } from "./repositories/spotifyRepository";
import type { ModuleId } from "./types";
import { useAI } from "./contexts/useAI";
import { MiniPlayer } from "./components/media/MiniPlayer";
import "./styles/jarvis.css";

function App() {
  const { coreState, engineLabel, engineAvailable } = useAI();
  const [activeModule, setActiveModule] = useState<ModuleId>("command");
  const [isFullscreen, setIsFullscreen] = useState(false);
  const [now, setNow] = useState(() => new Date());
  const webPlayer = useSpotifyWebPlayer();
  const { start: startWebPlayer } = webPlayer;
  // Com o Spotify conectado, o player do Azriel sobe junto com o app: ele já é a saída padrão quando você tocar algo.
  useEffect(() => {
    const timer = window.setTimeout(() => { void spotifyRepository.status().then((status) => { if (status.connected) void startWebPlayer(); }).catch(() => undefined); }, 1_500);
    return () => window.clearTimeout(timer);
  }, [startWebPlayer]);
  const [playerOpen, setPlayerOpen] = useState(false);

  const changeFullscreen = useCallback(async (forcedState?: boolean) => {
    const window = getCurrentWindow();
    const state = forcedState ?? !(await window.isFullscreen());
    await window.setFullscreen(state);
    setIsFullscreen(state);
  }, []);

  const enterOrbMode = useCallback(async () => {
    const window = getCurrentWindow();
    const orb = await Window.getByLabel("orb");
    if (!orb) {
      await window.minimize();
      return;
    }
    await orb.show();
    await window.hide();
  }, []);

  useEffect(() => {
    const timer = window.setInterval(() => setNow(new Date()), 1000);
    return () => window.clearInterval(timer);
  }, []);

  useEffect(() => {
    let active = true;
    const appWindow = getCurrentWindow();
    void appWindow.isFullscreen().then((value) => active && setIsFullscreen(value)).catch(() => undefined);
    const shortcut = (event: KeyboardEvent) => {
      if (event.key === "F11") {
        event.preventDefault();
        void changeFullscreen();
      } else if (event.key === "Escape" && isFullscreen) {
        void changeFullscreen(false);
      }
    };
    globalThis.window.addEventListener("keydown", shortcut);
    return () => {
      active = false;
      globalThis.window.removeEventListener("keydown", shortcut);
    };
  }, [changeFullscreen, isFullscreen]);

  const content =
    activeModule === "ai" ? <AICorePage />
    : activeModule === "daily" ? <DailyOperationsPage />
    : activeModule === "music" ? <MusicPage webPlayer={webPlayer} />
    : activeModule === "money" ? <MoneyPage />
    : activeModule === "settings" ? <SettingsPage />
    : <NucleoPage
      coreState={coreState}
      onOpenAI={() => setActiveModule("ai")}
      onOpenDaily={() => setActiveModule("daily")}
      onOpenYouTube={() => setPlayerOpen(true)}
      onOpenMusic={() => setActiveModule("music")}
      azrielDeviceId={webPlayer.status === "ready" ? webPlayer.deviceId : null}
      azrielState={webPlayer.state}
    />;

  return (
    <div className={`jv-shell jv-shell--${coreState}`} data-module={activeModule}>
      <div className="jv-backdrop" aria-hidden="true" />

      <header className="jv-top">
        <button type="button" className="jv-brand" onClick={() => setActiveModule("command")} aria-label="Voltar ao núcleo">
          <span className="jv-brand__mark"><i /></span>
          <strong>AZRIEL</strong>
        </button>
        <div className="jv-clock">
          <strong>{now.toLocaleTimeString("pt-BR", { hour: "2-digit", minute: "2-digit" })}<small>{now.toLocaleTimeString("pt-BR", { second: "2-digit" }).padStart(2, "0")}</small></strong>
          <span>{now.toLocaleDateString("pt-BR", { weekday: "long", day: "2-digit", month: "long" })}</span>
        </div>
        <div className="jv-top__right">
          <span className="jv-status" data-online={engineAvailable}><i />{engineLabel} · {azrielStates[coreState].label}</span>
          <button type="button" className="jv-icon" onClick={() => void enterOrbMode()} title="Minimizar para o orbe" aria-label="Minimizar para o orbe">−</button>
          <button type="button" className="jv-icon" data-active={isFullscreen} onClick={() => void changeFullscreen()} title={isFullscreen ? "Sair da tela cheia" : "Tela cheia"} aria-label="Tela cheia">⛶</button>
        </div>
      </header>

      <nav className="jv-rail" aria-label="Módulos">
        {modules.map((module) => (
          <button
            key={module.id}
            type="button"
            className="jv-rail__node"
            data-active={activeModule === module.id}
            onClick={() => setActiveModule(module.id)}
            aria-label={module.label}
          >
            <span className="jv-rail__glyph" aria-hidden="true">{module.glyph}</span>
            <span className="jv-rail__label"><b>{module.label}</b><small>{module.code}</small></span>
          </button>
        ))}
      </nav>

      <main className="jv-stage" key={activeModule}>
        {content}
      </main>

      {playerOpen && <MiniPlayer onClose={() => setPlayerOpen(false)} />}
    </div>
  );
}

export default App;
