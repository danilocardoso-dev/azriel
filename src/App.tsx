import { lazy, Suspense, useCallback, useEffect, useMemo, useState } from "react";
import { getCurrentWindow, Window } from "@tauri-apps/api/window";
import { modules } from "./data/system";
import { CommandCenter } from "./pages/CommandCenter";
import { EducationPage } from "./pages/EducationPage";
import { ProjectsPage } from "./pages/ProjectsPage";
import { SettingsPage } from "./pages/SettingsPage";
import { StarkMapPage } from "./pages/StarkMapPage";
import { SystemPage } from "./pages/SystemPage";
import { DailyOperationsPage } from "./pages/DailyOperationsPage";
import { AICorePage } from "./pages/AICorePage";
import { AutomationPage } from "./pages/AutomationPage";
import type { ModuleId } from "./types";
import { useAzrielData } from "./contexts/useAzrielData";
import { DataState } from "./components/layout/DataState";
import { useAI } from "./contexts/useAI";
import { useAutomation } from "./contexts/useAutomation";
import { RoutineConfirmationDialog } from "./components/automation/RoutineConfirmationDialog";

const EngineeringViewPage = lazy(() => import("./pages/EngineeringViewPage").then((module) => ({ default: module.EngineeringViewPage })));

function App() {
  const { loading, error, reload, databaseInfo, knowledgeAreas } = useAzrielData();
  const { coreState, status: aiStatus } = useAI();
  const { state: automationState, pendingRoutine } = useAutomation();
  const displayState = automationState === "executing" ? (pendingRoutine ? "routine" : "executing") : coreState;
  const [activeModule, setActiveModule] = useState<ModuleId>("command");
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
  const [isFullscreen, setIsFullscreen] = useState(false);
  const [moduleAction, setModuleAction] = useState<"new-project" | "new-task" | "new-note" | null>(null);
  const [now, setNow] = useState(() => new Date());

  const changeFullscreen = useCallback(async (forcedState?: boolean) => {
    const appWindow = getCurrentWindow();
    const nextState = forcedState ?? !(await appWindow.isFullscreen());
    await appWindow.setFullscreen(nextState);
    setIsFullscreen(nextState);
  }, []);

  const enterOrbMode = useCallback(async () => {
    const appWindow = getCurrentWindow();
    const orbWindow = await Window.getByLabel("orb");
    if (!orbWindow) {
      await appWindow.minimize();
      return;
    }
    await orbWindow.show();
    await appWindow.hide();
  }, []);

  useEffect(() => {
    const timer = window.setInterval(() => setNow(new Date()), 1000);
    return () => window.clearInterval(timer);
  }, []);

  useEffect(() => {
    let active = true;
    const appWindow = getCurrentWindow();
    void appWindow.isFullscreen().then((fullscreen) => {
      if (active) setIsFullscreen(fullscreen);
    }).catch(() => undefined);
    const handleFullscreenShortcut = (event: KeyboardEvent) => {
      if (event.key === "F11") {
        event.preventDefault();
        void changeFullscreen();
      } else if (event.key === "Escape" && isFullscreen) {
        void changeFullscreen(false);
      }
    };
    window.addEventListener("keydown", handleFullscreenShortcut);
    return () => {
      active = false;
      window.removeEventListener("keydown", handleFullscreenShortcut);
    };
  }, [changeFullscreen, isFullscreen]);

  const currentModule = useMemo(() => modules.find((module) => module.id === activeModule)!, [activeModule]);

  const renderModule = () => {
    const empty = activeModule === "stark" && knowledgeAreas.length === 0;
    if (["command", "projects", "stark", "education"].includes(activeModule) && (loading || error || empty)) {
      return <DataState loading={loading} error={error} empty={empty} onRetry={() => void reload()} />;
    }
    switch (activeModule) {
      case "engineering": return <Suspense fallback={<div className="data-state"><span className="live-dot" /><strong>INICIALIZANDO ENGINEERING CORE</strong><p>Carregando o renderer local.</p></div>}><EngineeringViewPage /></Suspense>;
      case "projects": return <ProjectsPage openCreate={moduleAction === "new-project"} />;
      case "ai": return <AICorePage />;
      case "daily": return <DailyOperationsPage initialCapture={moduleAction === "new-task" ? "task" : moduleAction === "new-note" ? "note" : undefined} />;
      case "stark": return <StarkMapPage />;
      case "education": return <EducationPage />;
      case "system": return <SystemPage coreState={coreState} onOpenAI={() => setActiveModule("ai")} />;
      case "automation": return <AutomationPage />;
      case "settings": return <SettingsPage />;
      default: return <CommandCenter coreState={displayState} onOpenAI={() => { setModuleAction(null); setActiveModule("ai"); }} onOpenStark={() => { setModuleAction(null); setActiveModule("stark"); }} onOpenDaily={() => { setModuleAction(null); setActiveModule("daily"); }} onNewProject={() => { setModuleAction("new-project"); setActiveModule("projects"); }} onNewTask={() => { setModuleAction("new-task"); setActiveModule("daily"); }} onNewNote={() => { setModuleAction("new-note"); setActiveModule("daily"); }} />;
    }
  };

  return (
    <div className={`app-shell app-shell--${displayState} ${sidebarCollapsed ? "app-shell--sidebar-collapsed" : ""}`}>
      <header className="topbar">
        <button className="brand" onClick={() => { setModuleAction(null); setActiveModule("command"); }} aria-label="Abrir Command Center">
          <span className="brand__mark"><i /></span><strong>AZRIEL</strong><small>PERSONAL INTELLIGENCE SYSTEM</small>
        </button>
        <div className="topbar__context"><span>{currentModule.code}</span>{currentModule.description}</div>
        <div className="topbar__system">
          <div className="topbar__status"><span className="live-dot" /><div><strong>SISTEMA ONLINE</strong><small>HUD V0.8.3 / SQLite {databaseInfo ? `S${databaseInfo.schemaVersion}` : ""}</small></div></div>
          <div className="topbar__window-actions">
            <button className="topbar__window-control" onClick={() => void enterOrbMode()} aria-label="Minimizar para o orbe" title="Minimizar para o orbe">&minus;</button>
            <button className={`topbar__window-control ${isFullscreen ? "active" : ""}`} onClick={() => void changeFullscreen()} aria-label={isFullscreen ? "Sair da tela cheia" : "Entrar em tela cheia"} aria-pressed={isFullscreen} title={isFullscreen ? "Sair da tela cheia (Esc)" : "Tela cheia (F11)"}>⛶</button>
          </div>
        </div>
      </header>

      <aside className="sidebar">
        <div className="sidebar__head">
          <span className="sidebar__label">MÓDULOS DO SISTEMA</span>
          <button
            className="sidebar-toggle"
            onClick={() => setSidebarCollapsed((collapsed) => !collapsed)}
            aria-label={sidebarCollapsed ? "Expandir menu lateral" : "Recolher menu lateral"}
            aria-expanded={!sidebarCollapsed}
            title={sidebarCollapsed ? "Expandir menu" : "Recolher menu"}
          >
            <span>{sidebarCollapsed ? "»" : "«"}</span>
          </button>
        </div>
        <nav aria-label="Módulos principais">
          {modules.map((module, index) => (
            <button className={activeModule === module.id ? "active" : ""} onClick={() => { setModuleAction(null); setActiveModule(module.id); }} key={module.id} title={sidebarCollapsed ? module.label : undefined}>
              <span>{String(index + 1).padStart(2, "0")}</span><i>{module.code}</i><strong>{module.label}</strong>
            </button>
          ))}
        </nav>
        <div className="sidebar__telemetry">
          <span>SESSION</span><strong>LOCAL / SQLITE</strong>
          <span>DATABASE</span><strong>{databaseInfo ? `SCHEMA ${databaseInfo.schemaVersion}` : "CONECTANDO"}</strong>
          <span>AI CORE</span><strong>{aiStatus?.available ? "OLLAMA ONLINE" : "OFFLINE"}</strong>
          <span>SECURITY</span><strong>SAFE ACTIONS</strong>
        </div>
      </aside>

      <main className="workspace" key={activeModule}>{renderModule()}</main>

      <footer className="statusbar">
        <span>AZRIEL // KNOWLEDGE · ENGINEERING · INTELLIGENCE · IMPACT</span>
        <span>{now.toLocaleDateString("pt-BR", { day: "2-digit", month: "short", year: "numeric" }).toUpperCase()}</span>
        <strong>{now.toLocaleTimeString("pt-BR", { hour12: false })}</strong>
      </footer>
      <RoutineConfirmationDialog key={pendingRoutine?.historyId ?? "routine-none"} />
    </div>
  );
}

export default App;
