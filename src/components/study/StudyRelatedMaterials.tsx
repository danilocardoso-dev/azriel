import { useEffect, useState } from "react";
import { studyLabService } from "../../services/studyLabService";
import type { StudyMaterialRelationType, StudyMaterialSummary } from "../../types";

type Props = {
  relationType: StudyMaterialRelationType;
  relationId: string;
  onOpenLibrary?: (materialId?: string) => void;
};

export function StudyRelatedMaterials({ relationType, relationId, onOpenLibrary }: Props) {
  const [materials, setMaterials] = useState<StudyMaterialSummary[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    void studyLabService.materials({ relationType, relationId, limit: 20, offset: 0 })
      .then((items) => { if (active) setMaterials(items); })
      .catch((reason: unknown) => { if (active) setError(reason instanceof Error ? reason.message : String(reason)); });
    return () => { active = false; };
  }, [relationId, relationType]);

  return <section className="study-related-materials">
    <header><div><strong>MATERIAIS RELACIONADOS</strong><span>{materials.length} NA BIBLIOTECA</span></div>{onOpenLibrary && <button onClick={() => onOpenLibrary()}>ABRIR BIBLIOTECA</button>}</header>
    {error && <small className="study-material-error">{error}</small>}
    <div>{materials.map((material) => <button key={material.id} onClick={() => onOpenLibrary ? onOpenLibrary(material.id) : void studyLabService.openMaterial(material.id)}><b>{material.materialType}</b><span>{material.title}</span><i>{material.status}</i></button>)}{!materials.length && !error && <small>Nenhum material associado.</small>}</div>
  </section>;
}
