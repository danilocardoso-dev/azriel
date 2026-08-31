import { useEffect, useMemo, useState } from "react";
import { modules } from "./data/system";
import { CommandCenter } from "./pages/CommandCenter";
import { EducationPage } from "./pages/EducationPage";
import { KnowledgePage } from "./pages/KnowledgePage";
import { ProjectsPage } from "./pages/ProjectsPage";
import { ResearchPage } from "./pages/ResearchPage";
import { SettingsPage } from "./pages/SettingsPage";
import { StarkMapPage } from "./pages/StarkMapPage";
import { SystemPage } from "./pages/SystemPage";
import type { AzrielState, ModuleId } from "./types";
import { useAzrielData } from "./contexts/useAzrielData";
import { DataState } from "./components/layout/DataState";

function App() {
  const { loading, error, reload, databaseInfo, projects, knowledgeAreas, education } = useAzrielData();
  const [activeModule, setActiveModule] = useState<ModuleId>("command");
  const [coreState, setCoreState] = useState<AzrielState>("idle");
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
  const [now, setNow] = useState(() => new Date());

  useEffect(() => {
    const timer = window.setInterval(() => setNow(new Date()), 1000);
    return () => window.clearInterval(timer);
  }, []);

  const currentModule = useMemo(() => modules.find((module) => module.id === activeModule)!, [activeModule]);

  const renderModule = () => {
    const empty = activeModule === "projects" ? projects.length === 0
      : activeModule === "education" ? education.length === 0
      : ["knowledge", "stark"].includes(activeModule) ? knowledgeAreas.length === 0
      : activeModule === "command" ? projects.length === 0 && knowledgeAreas.length === 0 : false;
    if (["command", "projects", "knowledge", "stark", "education"].includes(activeModule) && (loading || error || empty)) {
      return <DataState loading={loading} error={error} empty={empty} onRetry={() => void reload()} />;
    }
    switch (activeModule) {
      case "projects": return <ProjectsPage />;
      case "knowledge": return <KnowledgePage />;
      case "stark": return <StarkMapPage />;
      case "education": return <EducationPage />;
      case "research": return <ResearchPage />;
      case "system": return <SystemPage coreState={coreState} />;
      case "settings": return <SettingsPage />;
      default: return <CommandCenter coreState={coreState} setCoreState={setCoreState} />;
    }
  };

  return (
    <div className={`app-shell app-shell--${coreState} ${sidebarCollapsed ? "app-shell--sidebar-collapsed" : ""}`}>
      <header className="topbar">
        <button className="brand" onClick={() => setActiveModule("command")} aria-label="Abrir Command Center">
          <span className="brand__mark"><i /></span><strong>AZRIEL</strong><small>PERSONAL INTELLIGENCE SYSTEM</small>
        </button>
        <div className="topbar__context"><span>{currentModule.code}</span>{currentModule.description}</div>
        <div className="topbar__status"><span className="live-dot" /><div><strong>SISTEMA ONLINE</strong><small>HUD V0.5 / SQLite {databaseInfo ? `S${databaseInfo.schemaVersion}` : ""}</small></div></div>
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
            <button className={activeModule === module.id ? "active" : ""} onClick={() => setActiveModule(module.id)} key={module.id} title={sidebarCollapsed ? module.label : undefined}>
              <span>{String(index + 1).padStart(2, "0")}</span><i>{module.code}</i><strong>{module.label}</strong>
            </button>
          ))}
        </nav>
        <div className="sidebar__telemetry">
          <span>SESSION</span><strong>LOCAL / SQLITE</strong>
          <span>DATABASE</span><strong>{databaseInfo ? `SCHEMA ${databaseInfo.schemaVersion}` : "CONECTANDO"}</strong>
          <span>AI CORE</span><strong>STANDBY</strong>
          <span>SECURITY</span><strong>NO NATIVE I/O</strong>
        </div>
      </aside>

      <main className="workspace" key={activeModule}>{renderModule()}</main>

      <footer className="statusbar">
        <span>AZRIEL // KNOWLEDGE · ENGINEERING · INTELLIGENCE · IMPACT</span>
        <span>{now.toLocaleDateString("pt-BR", { day: "2-digit", month: "short", year: "numeric" }).toUpperCase()}</span>
        <strong>{now.toLocaleTimeString("pt-BR", { hour12: false })}</strong>
      </footer>
    </div>
  );
}

export default App;
