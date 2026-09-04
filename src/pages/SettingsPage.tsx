import { useEffect, useState } from "react";
import { AISettingsPanel } from "../components/ai/AISettingsPanel";
import { ModuleIntro } from "../components/layout/ModuleIntro";
import { useAzrielData } from "../contexts/useAzrielData";
import { starkService } from "../services/starkService";
import type { LearningEngineStatus } from "../types";

export function SettingsPage() {
  const { databaseInfo, reload } = useAzrielData();
  const [scanlines, setScanlines] = useState(true);
  const [dense, setDense] = useState(true);
  const [telemetry, setTelemetry] = useState(true);
  const [learning, setLearning] = useState<LearningEngineStatus | null>(null);
  const [rebuilding, setRebuilding] = useState(false);
  const [learningMessage, setLearningMessage] = useState<string | null>(null);
  useEffect(() => { void starkService.learningStatus().then(setLearning).catch(() => setLearningMessage("Learning Engine indisponível")); }, []);
  async function rebuild() { setRebuilding(true); setLearningMessage(null); try { await starkService.rebuildLearning(); await reload(); setLearning(await starkService.learningStatus()); setLearningMessage("Reconstrução concluída sem criar novos eventos."); } catch (reason) { setLearningMessage(reason instanceof Error ? reason.message : String(reason)); } finally { setRebuilding(false); } }
  return <>
    <ModuleIntro code="CFG-09" title="Configurações" description="Preferências da interface, persistência, Learning Engine e conexão do AI Core." metric="OLLAMA LOCAL" />
    <div className="settings-layout">
      <section><header><span>INTERFACE</span><i>HUD</i></header>
        <label><span><strong>Scan lines</strong><small>Textura técnica sobre a interface</small></span><input type="checkbox" checked={scanlines} onChange={(event) => setScanlines(event.target.checked)} /></label>
        <label><span><strong>Densidade elevada</strong><small>Prioriza dados e painéis compactos</small></span><input type="checkbox" checked={dense} onChange={(event) => setDense(event.target.checked)} /></label>
        <label><span><strong>Telemetria visual</strong><small>Exibe códigos e indicadores simulados</small></span><input type="checkbox" checked={telemetry} onChange={(event) => setTelemetry(event.target.checked)} /></label>
      </section>
      <section><header><span>SISTEMA</span><i>V0.8.3</i></header><div className="config-readout"><span>Tema<strong>AZRIEL DARK</strong></span><span>Cor primária<strong>CYAN / #46E9FF</strong></span><span>Dados<strong>SQLITE LOCAL</strong></span><span>Persistência<strong>ATIVA / SCHEMA {databaseInfo?.schemaVersion ?? "-"}</strong></span><span>Arquivo<strong>{databaseInfo?.path ?? "Conectando..."}</strong></span></div></section>
      <section><header><span>LEARNING ENGINE</span><i>{learning?.formulaVersion ?? "V1"}</i></header><div className="config-readout"><span>Status<strong>{learning?.status.toUpperCase() ?? "CARREGANDO"}</strong></span><span>Eventos<strong>{learning?.eventCount ?? "—"}</strong></span><span>Integração<strong>{learning ? `${learning.currentIntegration.toFixed(2)}%` : "—"}</strong></span><span>Último cálculo<strong>{learning?.lastRecalculatedAt ? new Date(learning.lastRecalculatedAt).toLocaleString("pt-BR") : "AINDA NÃO EXECUTADO"}</strong></span></div><button disabled={rebuilding} onClick={() => void rebuild()}>{rebuilding ? "RECALCULANDO..." : "RECONSTRUIR A PARTIR DO LEDGER"}</button>{learningMessage && <p>{learningMessage}</p>}</section>
      <AISettingsPanel />
    </div>
    <p className="settings-note">Os dados dos núcleos e as evidências do Learning Engine são persistentes. As preferências visuais permanecem válidas apenas durante a sessão atual.</p>
  </>;
}
