import { useEffect, useRef } from "react";

type Props = {
  hasRoadmap: boolean;
  onNew: () => void;
  onImport: () => void;
  onExport: () => void;
  onEdit: () => void;
  onDelete: () => void;
};

export function RoadmapManagementMenu({ hasRoadmap, onNew, onImport, onExport, onEdit, onDelete }: Props) {
  const detailsRef = useRef<HTMLDetailsElement>(null);
  useEffect(() => {
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape" && detailsRef.current?.open) detailsRef.current.open = false;
    };
    window.addEventListener("keydown", closeOnEscape);
    return () => window.removeEventListener("keydown", closeOnEscape);
  }, []);
  const run = (action: () => void) => { if (detailsRef.current) detailsRef.current.open = false; action(); };
  return <details ref={detailsRef} className="roadmap-management">
    <summary aria-label="Gerenciar roadmaps" title="Gerenciar roadmaps">⋯</summary>
    <nav>
      <button onClick={() => run(onNew)}>＋ NOVO ROADMAP</button>
      <button onClick={() => run(onImport)}>IMPORTAR JSON</button>
      <button disabled={!hasRoadmap} onClick={() => run(onExport)}>EXPORTAR JSON</button>
      <button disabled={!hasRoadmap} onClick={() => run(onEdit)}>EDITAR ROADMAP</button>
      <button disabled={!hasRoadmap} className="danger-link" onClick={() => run(onDelete)}>EXCLUIR ROADMAP</button>
    </nav>
  </details>;
}
