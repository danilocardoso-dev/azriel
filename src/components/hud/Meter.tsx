interface MeterProps {
  label: string;
  value: number;
  tone?: "cyan" | "amber";
  compact?: boolean;
}

export function Meter({ label, value, tone = "cyan", compact = false }: MeterProps) {
  return (
    <div className={`meter ${compact ? "meter--compact" : ""}`}>
      <div className="meter__meta">
        <span>{label}</span>
        <strong>{value}%</strong>
      </div>
      <div className="meter__track" aria-label={`${label}: ${value}%`} role="meter" aria-valuenow={value} aria-valuemin={0} aria-valuemax={100}>
        <span className={`meter__value meter__value--${tone}`} style={{ width: `${value}%` }} />
      </div>
    </div>
  );
}
