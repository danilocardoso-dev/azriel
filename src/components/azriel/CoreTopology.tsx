import type { AzrielState } from "../../types";
import { AzrielCore } from "./AzrielCore";

interface CoreTopologyProps {
  state: AzrielState;
  aiOnline: boolean;
  systemOnline: boolean;
  automationOnline: boolean;
  knowledgeCount: number;
  projectCount: number;
  dailyCount: number;
  onOpenAI: () => void;
  onOpenDaily: () => void;
}

interface TopologyNodeProps {
  className: string;
  code: string;
  label: string;
  value: string;
  state?: "online" | "standby" | "warning";
  onActivate?: () => void;
}

function TopologyNode({ className, code, label, value, state = "online", onActivate }: TopologyNodeProps) {
  const content = <><small>{code}</small><strong>{label}</strong><span data-state={state}>{value}</span></>;
  const classes = `core-topology__node ${className}`;
  return onActivate
    ? <button className={classes} onClick={onActivate}>{content}</button>
    : <div className={classes}>{content}</div>;
}

export function CoreTopology({ state, aiOnline, systemOnline, automationOnline, knowledgeCount, projectCount, dailyCount, onOpenAI, onOpenDaily }: CoreTopologyProps) {
  return (
    <section className="core-topology" aria-label="Topologia dos núcleos do Azriel">
      <header><span>CORE TOPOLOGY // 01</span><strong>AZRIEL CORE</strong><i>LINK MATRIX / LIVE</i></header>
      <span className="core-topology__coordinate core-topology__coordinate--north">N 00° / REF 01</span>
      <span className="core-topology__coordinate core-topology__coordinate--west">X -042.17</span>
      <span className="core-topology__coordinate core-topology__coordinate--east">X +042.17</span>
      <svg className="core-topology__links" viewBox="0 0 1000 610" preserveAspectRatio="none" aria-hidden="true">
        <g className="core-topology__grid"><path d="M500 72V538M95 305H905"/><circle cx="500" cy="305" r="218"/><circle cx="500" cy="305" r="150"/></g>
        <g className="core-topology__paths">
          <path d="M255 130H365L425 225"/><path d="M745 130H635L575 225"/>
          <path d="M205 305H390"/><path d="M795 305H610"/>
          <path d="M255 480H365L425 385"/><path d="M745 480H635L575 385"/>
        </g>
        <g className="core-topology__junctions">
          <circle cx="365" cy="130" r="4"/><circle cx="635" cy="130" r="4"/>
          <circle cx="390" cy="305" r="4"/><circle cx="610" cy="305" r="4"/>
          <circle cx="365" cy="480" r="4"/><circle cx="635" cy="480" r="4"/>
        </g>
      </svg>

      <TopologyNode className="core-topology__node--ai" code="AIC-02" label="AI CORE" value={aiOnline ? "ONLINE" : "OFFLINE"} state={aiOnline ? "online" : "warning"} onActivate={onOpenAI} />
      <TopologyNode className="core-topology__node--system" code="SYS-09" label="SYSTEM CORE" value={systemOnline ? "TELEMETRY LIVE" : "UNAVAILABLE"} state={systemOnline ? "online" : "warning"} />
      <TopologyNode className="core-topology__node--knowledge" code="KNO-05" label="KNOWLEDGE CORE" value={`${knowledgeCount.toString().padStart(2, "0")} DOMAINS`} />
      <TopologyNode className="core-topology__node--automation" code="AUT-10" label="AUTOMATION CORE" value={automationOnline ? "SAFE MODE" : "OFFLINE"} state={automationOnline ? "online" : "warning"} />
      <TopologyNode className="core-topology__node--projects" code="PRJ-04" label="PROJECT CORE" value={`${projectCount.toString().padStart(2, "0")} NODES`} />
      <TopologyNode className="core-topology__node--daily" code="OPS-03" label="DAILY OPERATIONS" value={`${dailyCount.toString().padStart(2, "0")} ACTIVE`} state={dailyCount > 0 ? "warning" : "standby"} onActivate={onOpenDaily} />

      <div className="core-topology__center"><AzrielCore state={state} onClick={onOpenAI} /></div>
      <footer><span>BUS // 6 LINKS</span><span>LATENCY // LOCAL</span><span>AUTHORITY // CONTROLLED</span></footer>
    </section>
  );
}
