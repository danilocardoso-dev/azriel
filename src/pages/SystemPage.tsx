import { AzrielCore } from "../components/azriel/AzrielCore";
import { HudPanel } from "../components/hud/HudPanel";
import { ModuleIntro } from "../components/layout/ModuleIntro";
import { systemNodes } from "../data/system";
import type { AzrielState } from "../types";

interface SystemPageProps { coreState: AzrielState; onOpenAI: () => void; }

export function SystemPage({ coreState, onOpenAI }: SystemPageProps) {
  const nodes = systemNodes.map((node) => node.name === "AI Core" ? {
    ...node,
    state: coreState === "offline" ? "offline" : coreState === "alert" ? "alert" : coreState === "processing" || coreState === "tool" ? "processing" : "online",
    detail: coreState === "offline" ? "Ollama local indisponível" : "Ollama local / tools read-only",
  } : node);
  return (
    <>
      <ModuleIntro code="SYS-08" title="Sistema" description="Diagnóstico dos núcleos e limites operacionais do Azriel." metric="AI LOCAL // V0.6" />
      <div className="system-layout">
        <HudPanel title="Topologia dos núcleos" code="ARCH/01" className="topology-panel">
          <div className="topology-core"><AzrielCore state={coreState} onClick={onOpenAI} compact /></div>
          <div className="topology-lines" />
          <div className="system-nodes">{nodes.map((node) => <article key={node.name}><span data-state={node.state} /><div><strong>{node.name}</strong><small>{node.detail}</small></div><i>{node.state}</i></article>)}</div>
        </HudPanel>
        <HudPanel title="Limites da versão" code="SECURITY">
          <div className="limits-list"><p><strong>SEM TELEMETRIA REAL</strong>CPU, RAM, processos e arquivos entram na v0.7.</p><p><strong>SEM AUTOMAÇÃO</strong>Nenhum comando do sistema operacional é executado.</p><p><strong>SEM REDE IoT</strong>MQTT e dispositivos pertencem à v0.9.</p><p><strong>IA SOMENTE LEITURA</strong>O AI Core consulta tools controladas e não modifica dados.</p></div>
        </HudPanel>
      </div>
    </>
  );
}
