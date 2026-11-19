import { useState } from "react";
import { summarizeSteps } from "../../ai/agent/steps";
import type { AgentStep } from "../../types";

const ICONS: Record<AgentStep["status"], string> = { running: "", ok: "✓", error: "✕", denied: "⊘" };
const PATH_TOOLS = ["read_file", "list_folder", "write_file", "edit_file", "create_folder", "delete_path"];

/** Caminho com o nome do arquivo em destaque e a pasta apagada, como no Claude. */
function Target({ step }: { step: AgentStep }) {
  if (!step.target) return null;
  if (PATH_TOOLS.includes(step.tool)) {
    const cut = step.target.replace(/[\\/]+$/, "").search(/[^\\/]+$/);
    const folder = cut > 0 ? step.target.slice(0, cut) : "";
    const name = cut >= 0 ? step.target.slice(cut) : step.target;
    return <code className="agent-step__target" title={step.target}><span>{folder}</span>{name}</code>;
  }
  return <code className="agent-step__target" title={step.target}>{step.target}</code>;
}

function StepRow({ step }: { step: AgentStep }) {
  const [open, setOpen] = useState(false);
  const expandable = Boolean(step.detail);
  return <li className="agent-step" data-status={step.status}>
    <button type="button" className="agent-step__line" onClick={() => expandable && setOpen((value) => !value)} aria-expanded={expandable ? open : undefined} disabled={!expandable}>
      <i aria-hidden="true">{ICONS[step.status]}</i>
      <b>{step.label}</b>
      <Target step={step} />
      {(step.added || step.removed) ? <span className="agent-step__diff">{step.added ? <ins>+{step.added}</ins> : null}{step.removed ? <del>−{step.removed}</del> : null}</span> : null}
      {expandable && <em aria-hidden="true">{open ? "▾" : "▸"}</em>}
    </button>
    {open && step.detail && <pre className="agent-step__detail">{step.detail}</pre>}
  </li>;
}

/**
 * O que o Azriel fez numa resposta. Ao vivo mostra tudo; no histórico, listas longas começam
 * recolhidas num resumo ("8 passos · 2 arquivos alterados · 3 comandos").
 */
export function AgentSteps({ steps, live = false }: { steps: AgentStep[]; live?: boolean }) {
  const [expanded, setExpanded] = useState(false);
  if (!steps.length) return null;
  const collapsible = !live && steps.length > 4;
  const visible = collapsible && !expanded ? [] : steps;
  return <div className="agent-steps" data-live={live}>
    {collapsible && <button type="button" className="agent-steps__summary" onClick={() => setExpanded((value) => !value)} aria-expanded={expanded}>
      <em aria-hidden="true">{expanded ? "▾" : "▸"}</em>{summarizeSteps(steps)}
    </button>}
    {visible.length > 0 && <ul>{visible.map((step) => <StepRow key={step.id} step={step} />)}</ul>}
  </div>;
}
