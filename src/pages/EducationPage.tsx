import { ModuleIntro } from "../components/layout/ModuleIntro";
import { useAzrielData } from "../contexts/useAzrielData";

export function EducationPage() {
  const { education } = useAzrielData();
  return (
    <>
      <ModuleIntro code="EDU-05" title="Formação" description="Trajetória acadêmica orientada pelas lacunas e pelos problemas encontrados." metric="2026 → FUTURO" />
      <div className="education-timeline">
        {education.map((item, index) => (
          <article className={`education-node education-node--${item.status}`} key={item.id}>
            <div className="education-node__rail"><span>{String(index + 1).padStart(2, "0")}</span></div>
            <div className="education-node__content">
              <header><span>{item.period}</span><i>{item.status.replaceAll("_", " ")}</i></header>
              <h2>{item.name}</h2><p>{item.description}</p>
              <div className="tag-row">{item.domains.map((domain) => <i key={domain}>{domain}</i>)}</div>
            </div>
          </article>
        ))}
      </div>
    </>
  );
}
