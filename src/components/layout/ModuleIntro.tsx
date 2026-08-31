interface ModuleIntroProps {
  code: string;
  title: string;
  description: string;
  metric?: string;
}

export function ModuleIntro({ code, title, description, metric }: ModuleIntroProps) {
  return (
    <header className="module-intro">
      <div>
        <span className="eyebrow">MÓDULO // {code}</span>
        <h1>{title}</h1>
        <p>{description}</p>
      </div>
      {metric && <div className="module-intro__metric">{metric}</div>}
    </header>
  );
}
