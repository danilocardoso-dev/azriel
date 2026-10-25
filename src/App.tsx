import { useCallback, useEffect, useState } from "react";
import { getCurrentWindow, Window } from "@tauri-apps/api/window";
import { modules } from "./data/system";
import { CommandCenter } from "./pages/CommandCenter";
import { SettingsPage } from "./pages/SettingsPage";
import { DailyOperationsPage } from "./pages/DailyOperationsPage";
import { AICorePage } from "./pages/AICorePage";
import { TradingPage } from "./pages/TradingPage";
import type { ModuleId } from "./types";
import { useAI } from "./contexts/useAI";

function App() {
  const { coreState } = useAI();
  const [activeModule, setActiveModule] = useState<ModuleId>("command");
  const [isFullscreen, setIsFullscreen] = useState(false);
  const [moduleAction, setModuleAction] = useState<"new-task" | "new-note" | null>(null);
  const [now, setNow] = useState(() => new Date());

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

  const openDaily = (action: "new-task" | "new-note" | null = null) => {
    setModuleAction(action);
    setActiveModule("daily");
  };

  const content =
    activeModule === "ai" ? (
      <AICorePage />
    ) : activeModule === "daily" ? (
      <DailyOperationsPage
        initialCapture={moduleAction === "new-task" ? "task" : moduleAction === "new-note" ? "note" : undefined}
      />
    ) : activeModule === "trading" ? (
      <TradingPage />
    ) : activeModule === "settings" ? (
      <SettingsPage />
    ) : (
      <CommandCenter
        coreState={coreState}
        onOpenAI={() => setActiveModule("ai")}
        onOpenDaily={() => openDaily()}
        onNewTask={() => openDaily("new-task")}
        onNewNote={() => openDaily("new-note")}
      />
    );

  return (
    <div className={`stark-shell stark-shell--${coreState}`}>
      {/* Top Holographic Navigation Bar */}
      <header className="stark-topbar">
        <button
          className="stark-brand"
          onClick={() => {
            setModuleAction(null);
            setActiveModule("command");
          }}
          aria-label="Voltar ao Reator Central"
          type="button"
        >
          <span className="brand__mark"><i /></span>
          <div>
            <strong>AZRIEL</strong>
            <small>TACTICAL INTELLIGENCE</small>
          </div>
        </button>

        {/* Central HUD Mode Switcher */}
        <nav className="stark-nav" aria-label="Deck tático">
          {modules.map((module) => (
            <button
              key={module.id}
              type="button"
              className={`stark-nav__pill ${activeModule === module.id ? "active" : ""}`}
              onClick={() => {
                setModuleAction(null);
                setActiveModule(module.id);
              }}
            >
              <span>{module.code}</span>
              <strong>{module.label}</strong>
            </button>
          ))}
        </nav>

        {/* Right Status Telemetry */}
        <div className="stark-telemetry-head">
          <div className="stark-status-beacon">
            <span className="live-dot" />
            <span>NÚCLEO {coreState.toUpperCase()}</span>
          </div>
          <div className="stark-time">{now.toLocaleTimeString("pt-BR")}</div>
          <div className="stark-window-controls">
            <button
              className="stark-btn-control"
              onClick={() => void enterOrbMode()}
              title="Minimizar para o Orbe"
              aria-label="Minimizar para o Orbe"
              type="button"
            >
              −
            </button>
            <button
              className={`stark-btn-control ${isFullscreen ? "active" : ""}`}
              onClick={() => void changeFullscreen()}
              title={isFullscreen ? "Sair da tela cheia" : "Tela cheia"}
              aria-label="Tela cheia"
              type="button"
            >
              ⛶
            </button>
          </div>
        </div>
      </header>

      {/* Main Holographic Canvas */}
      <main className="stark-stage" key={activeModule}>
        {content}
      </main>

      {/* Holographic Bottom Stream */}
      <footer className="stark-statusbar">
        <div className="stark-statusbar__tag">
          <span>MARK-IV // AZRIEL OPERATING SYSTEM</span>
          <i>DIRECT TACTICAL LINK</i>
        </div>
        <div className="stark-statusbar__metrics">
          <span>SQLITE CONECTADO</span>
          <span>OLLAMA LOCAL DISPONÍVEL</span>
          <span>TELEMETRIA CONTÍNUA</span>
        </div>
        <div className="stark-statusbar__date">
          <strong>{now.toLocaleDateString("pt-BR", { day: "2-digit", month: "short", year: "numeric" }).toUpperCase()}</strong>
        </div>
      </footer>
    </div>
  );
}

export default App;
