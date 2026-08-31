import { ModuleIntro } from "../components/layout/ModuleIntro";
import { researchItems } from "../data/research";

export function ResearchPage() {
  return (
    <>
      <ModuleIntro code="R&D-06" title="Pesquisa" description="Fila operacional de projetos, estudos e investigações interdisciplinares." metric="06 FRENTES" />
      <div className="research-table-wrap">
        <table className="research-table">
          <thead><tr><th>ID</th><th>Frente</th><th>Domínio</th><th>Objetivo</th><th>Status</th><th>Impacto</th></tr></thead>
          <tbody>{researchItems.map((item) => <tr key={item.id}><td>{item.id}</td><td>{item.title}</td><td>{item.domain}</td><td>{item.objective}</td><td><span data-status={item.status}>{item.status}</span></td><td>{item.impact}</td></tr>)}</tbody>
        </table>
      </div>
      <div className="research-footer"><span>Os registros são simulados nesta versão.</span><span>Persistência prevista para v0.5</span></div>
    </>
  );
}
