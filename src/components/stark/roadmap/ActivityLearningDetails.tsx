import type { RoadmapActivity, RoadmapActivityResource } from "../../../types";

type Props = {
  activity: RoadmapActivity;
  selectedResourceIds: Set<string>;
  onToggleResource: (resource: RoadmapActivityResource) => void;
  onOpenResource: (resource: RoadmapActivityResource) => Promise<void>;
  onAddToLibrary: (resource: RoadmapActivityResource) => Promise<void>;
};

function Section({ label, children }: { label: string; children: React.ReactNode }) {
  return <section className="activity-learning-section"><strong>{label}</strong><div>{children}</div></section>;
}

export function ActivityLearningDetails({ activity, selectedResourceIds, onToggleResource, onOpenResource, onAddToLibrary }: Props) {
  const resources = activity.resources ?? [];
  return <div className="activity-learning-model">
    {activity.isValidation && <span className="activity-checkpoint">CHECKPOINT</span>}
    {activity.learningObjective && <Section label="OBJETIVO"><p>{activity.learningObjective}</p></Section>}
    {activity.learningMethod && <Section label="MÉTODO"><p><b>{activity.learningMethod.type}</b>{activity.learningMethod.instructions ? ` · ${activity.learningMethod.instructions}` : ""}</p></Section>}
    {activity.instructions && <Section label="INSTRUÇÕES"><p>{activity.instructions}</p></Section>}
    {resources.length > 0 && <Section label="RECURSOS"><div className="activity-resources">{resources.map((resource) => <article key={resource.id}>
      <label title="Selecionar metadados deste recurso para o contexto da IA"><input type="checkbox" checked={selectedResourceIds.has(resource.id)} onChange={() => onToggleResource(resource)} /> IA</label>
      <span><b>[{resource.type}]</b> {resource.title}{resource.provider ? ` · ${resource.provider}` : ""}{resource.language ? ` · ${resource.language.toUpperCase()}` : ""}{resource.required ? " · OBRIGATÓRIO" : ""}</span>
      <nav>{resource.url && <button type="button" onClick={() => void onOpenResource(resource)}>ABRIR</button>}<button type="button" onClick={() => void onAddToLibrary(resource)}>{resource.studyMaterialId ? "NA BIBLIOTECA" : "+ BIBLIOTECA"}</button></nav>
    </article>)}</div></Section>}
    {activity.deliverable && <Section label="ENTREGA"><p>{activity.deliverable}</p></Section>}
    {activity.completionCriteria && <Section label="CRITÉRIO DE CONCLUSÃO"><p>{activity.completionCriteria}</p></Section>}
    {activity.reflectionPrompt && <Section label="REFLEXÃO"><p>{activity.reflectionPrompt}</p></Section>}
    {activity.estimatedMinutes && <Section label="ESTIMATIVA"><p>{activity.estimatedMinutes} minutos planejados · o cronômetro mantém sua configuração atual.</p></Section>}
  </div>;
}
