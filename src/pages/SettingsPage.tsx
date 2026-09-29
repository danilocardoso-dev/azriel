import { AISettingsPanel } from "../components/ai/AISettingsPanel";
import { ModuleIntro } from "../components/layout/ModuleIntro";
import { useAzrielData } from "../contexts/useAzrielData";

export function SettingsPage() {
  const { databaseInfo } = useAzrielData();
  return <>
    <ModuleIntro code="CFG-09" title="Configurações" description="Persistência local e conexão real do AI Core." metric="OLLAMA" />
    <div className="settings-layout">
      <section className="settings-system-panel"><header><span>SISTEMA</span><i>V0.8.4</i></header><div className="config-readout"><span>Interface<strong>AZRIEL DARK</strong></span><span>AI Provider<strong>OLLAMA</strong></span><span>Dados operacionais<strong>SQLITE LOCAL</strong></span><span>Persistência<strong>ATIVA / SCHEMA {databaseInfo?.schemaVersion ?? "-"}</strong></span><span>Arquivo do banco<strong>{databaseInfo?.path ?? "Conectando..."}</strong></span></div></section>
      <AISettingsPanel />
    </div>
  </>;
}
