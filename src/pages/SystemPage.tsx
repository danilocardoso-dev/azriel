import { AzrielCore } from "../components/azriel/AzrielCore";
import { HudPanel } from "../components/hud/HudPanel";
import { ModuleIntro } from "../components/layout/ModuleIntro";
import { systemNodes } from "../data/system";
import type { AzrielState } from "../types";

interface SystemPageProps { coreState: AzrielState; }

export function SystemPage({ coreState }: SystemPageProps) {
  return (
    <>
      <ModuleIntro code="SYS-07" title="Sistema" description="Diagnóstico simulado da arquitetura planejada do Azriel." metric="SEM ACESSO NATIVO // V0.4" />
      <div className="system-layout">
        <HudPanel title="Topologia dos núcleos" code="ARCH/01" className="topology-panel">
          <div className="topology-core"><AzrielCore state={coreState} onClick={() => undefined} compact /></div>
          <div className="topology-lines" />
          <div className="system-nodes">{systemNodes.map((node) => <article key={node.name}><span data-state={node.state} /><div><strong>{node.name}</strong><small>{node.detail}</small></div><i>{node.state}</i></article>)}</div>
        </HudPanel>
        <HudPanel title="Limites da versão" code="SECURITY">
          <div className="limits-list"><p><strong>SEM TELEMETRIA REAL</strong>CPU, RAM, processos e arquivos entram na v0.7.</p><p><strong>SEM AUTOMAÇÃO</strong>Nenhum comando do sistema operacional é executado.</p><p><strong>SEM REDE IoT</strong>MQTT e dispositivos pertencem à v0.9.</p><p><strong>SEM IA REAL</strong>Os estados do núcleo são apenas demonstrações visuais.</p></div>
        </HudPanel>
      </div>
    </>
  );
}
