import { useState } from "react";
import { EducationEditor } from "../components/core/EducationEditor";
import { DeleteConfirmationDialog } from "../components/daily/DeleteConfirmationDialog";
import { ModuleIntro } from "../components/layout/ModuleIntro";
import { useAzrielData } from "../contexts/useAzrielData";
import type { EducationInput, EducationItem } from "../types";

const statusLabels = { completed: "CONCLUÍDA", in_progress: "EM ANDAMENTO", planned: "PLANEJADA" } as const;

export function EducationPage() {
  const { education, saveEducation, deleteEducation } = useAzrielData();
  const [editing, setEditing] = useState<EducationItem | "new" | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<EducationItem | null>(null);
  const [deleting, setDeleting] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);
  async function save(input: EducationInput) { await saveEducation(input); setEditing(null); setActionError(null); }
  async function remove() { if (!deleteTarget) return; setDeleting(true); setActionError(null); try { await deleteEducation(deleteTarget.id); setDeleteTarget(null); } catch (reason) { setActionError(reason instanceof Error ? reason.message : String(reason)); setDeleteTarget(null); } finally { setDeleting(false); } }

  return <>
    <ModuleIntro code="EDU-05" title="Formação" description="Trajetória acadêmica orientada pelas lacunas e pelos problemas encontrados." metric={`${education.length} REGISTROS`} />
    <div className="core-actions"><button onClick={() => setEditing("new")}>＋ NOVA FORMAÇÃO</button>{actionError && <p className="form-error">{actionError}</p>}</div>
    {education.length ? <div className="education-timeline">{education.map((item, index) => <article className={`education-node education-node--${item.status}`} key={item.id}><div className="education-node__rail"><span>{String(index + 1).padStart(2, "0")}</span></div><div className="education-node__content"><header><span>{item.period || item.institution || "PERÍODO NÃO INFORMADO"}</span><i>{statusLabels[item.status]}</i></header><h2>{item.name}</h2>{item.institution && <small>{item.institution}</small>}<p>{item.description || "Descrição ainda não informada."}</p><div className="tag-row">{item.domains.map((domain) => <i key={domain}>{domain}</i>)}</div><div className="education-node__actions"><button onClick={() => setEditing(item)}>EDITAR</button><button className="danger-link" onClick={() => setDeleteTarget(item)}>EXCLUIR</button></div></div></article>)}</div> : <div className="core-empty">Nenhuma formação registrada.</div>}
    {editing && <EducationEditor item={editing === "new" ? null : editing} onCancel={() => setEditing(null)} onSave={save} />}
    {deleteTarget && <DeleteConfirmationDialog kind="formação" title={deleteTarget.name} description="A formação será removida definitivamente do banco local. Esta ação não altera conhecimentos, projetos ou arquivos externos." busy={deleting} onCancel={() => setDeleteTarget(null)} onConfirm={() => void remove()} />}
  </>;
}
