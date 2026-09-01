import type { EngineeringCalibration, EngineeringCalibrationInput } from "../../engineering/types";

interface EngineeringSettingsProps {
  value: EngineeringCalibration;
  saving: boolean;
  onChange: (input: EngineeringCalibrationInput) => void;
  onSave: () => void;
  onReset: () => void;
}

export function EngineeringSettings({ value, saving, onChange, onSave, onReset }: EngineeringSettingsProps) {
  const setNumber = (key: keyof EngineeringCalibrationInput, number: number) => onChange({ ...value, [key]: number });
  return (
    <section className="engineering-register engineering-settings">
      <header><strong>INTERACTION SETTINGS</strong><span>CALIBRATION // V0.2.1</span></header>
      <div className="engineering-settings__fields">
        <label>PINCH START<input type="number" min="0.015" max="0.1" step="0.005" value={value.pinchStartThreshold} onChange={(event) => setNumber("pinchStartThreshold", Number(event.target.value))} /></label>
        <label>PINCH RELEASE<input type="number" min="0.025" max="0.14" step="0.005" value={value.pinchReleaseThreshold} onChange={(event) => setNumber("pinchReleaseThreshold", Number(event.target.value))} /></label>
        <label>SMOOTHING<input type="range" min="0.05" max="1" step="0.05" value={value.smoothingAlpha} onChange={(event) => setNumber("smoothingAlpha", Number(event.target.value))} /><output>{value.smoothingAlpha.toFixed(2)}</output></label>
        <label>ROTATION<input type="range" min="0.25" max="8" step="0.25" value={value.rotationSensitivity} onChange={(event) => setNumber("rotationSensitivity", Number(event.target.value))} /><output>{value.rotationSensitivity.toFixed(2)}</output></label>
        <label>MIN SCALE<input type="number" min="0.2" max="1" step="0.05" value={value.minScale} onChange={(event) => setNumber("minScale", Number(event.target.value))} /></label>
        <label>MAX SCALE<input type="number" min="1" max="5" step="0.1" value={value.maxScale} onChange={(event) => setNumber("maxScale", Number(event.target.value))} /></label>
      </div>
      <p>CALIBRAÇÃO AUTOMÁTICA ADIADA // VALORES MANUAIS DISPONÍVEIS</p>
      <div className="engineering-settings__actions">
        <button onClick={onSave} disabled={saving}>SALVAR</button>
        <button disabled>CALIBRAR V0.2.1</button>
        <button onClick={onReset} disabled={saving}>RESETAR</button>
      </div>
    </section>
  );
}
