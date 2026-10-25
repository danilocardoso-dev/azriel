import { useEffect, useState } from "react";
import { AISettingsPanel } from "../components/ai/AISettingsPanel";
import { ModuleIntro } from "../components/layout/ModuleIntro";
import { getDatabaseInfo } from "../services/databaseService";
import type { DatabaseInfo } from "../types";

export function SettingsPage() {
  const [databaseInfo, setDatabaseInfo] = useState<DatabaseInfo | null>(null);

  useEffect(() => {
    let active = true;
    void getDatabaseInfo()
      .then((info) => {
        if (active) setDatabaseInfo(info);
      })
      .catch(() => undefined);
    return () => {
      active = false;
    };
  }, []);

  return (
    <section className="module-page settings-page">
      <ModuleIntro
        code="CFG-09"
        title="Configurações"
        description="Persistência local e parâmetros do modelo de inteligência artificial."
        metric="OLLAMA // LOCAL"
      />
      <div className="settings-layout">
        <section className="settings-system-panel">
          <header>
            <span>SISTEMA LOCAL</span>
            <i>V0.8.4</i>
          </header>
          <div className="config-readout">
            <span>
              Interface
              <strong>AZRIEL DARK // CYBER HUD</strong>
            </span>
            <span>
              AI Provider
              <strong>OLLAMA LOCAL</strong>
            </span>
            <span>
              Dados Operacionais
              <strong>SQLITE NATIVO</strong>
            </span>
            <span>
              Persistência
              <strong>ATIVA / SCHEMA {databaseInfo?.schemaVersion ?? "—"}</strong>
            </span>
            <span>
              Arquivo do Banco
              <strong>{databaseInfo?.path ?? "Conectando ao banco..."}</strong>
            </span>
          </div>
        </section>

        <AISettingsPanel />
      </div>
    </section>
  );
}
