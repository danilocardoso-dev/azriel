import { useState } from "react";
import { ModuleIntro } from "../components/layout/ModuleIntro";

export function SettingsPage() {
  const [scanlines, setScanlines] = useState(true);
  const [dense, setDense] = useState(true);
  const [telemetry, setTelemetry] = useState(true);
  return (
    <>
      <ModuleIntro code="CFG-08" title="Configurações" description="Preferências demonstrativas da interface. Persistência entra na v0.5." metric="SESSÃO LOCAL" />
      <div className="settings-layout">
        <section><header><span>INTERFACE</span><i>HUD</i></header>
          <label><span><strong>Scan lines</strong><small>Textura técnica sobre a interface</small></span><input type="checkbox" checked={scanlines} onChange={(event) => setScanlines(event.target.checked)} /></label>
          <label><span><strong>Densidade elevada</strong><small>Prioriza dados e painéis compactos</small></span><input type="checkbox" checked={dense} onChange={(event) => setDense(event.target.checked)} /></label>
          <label><span><strong>Telemetria visual</strong><small>Exibe códigos e indicadores simulados</small></span><input type="checkbox" checked={telemetry} onChange={(event) => setTelemetry(event.target.checked)} /></label>
        </section>
        <section><header><span>SISTEMA</span><i>V0.4</i></header><div className="config-readout"><span>Tema<strong>AZRIEL DARK</strong></span><span>Cor primária<strong>CYAN / #46E9FF</strong></span><span>Dados<strong>MOCK LOCAL</strong></span><span>Persistência<strong>INATIVA</strong></span><span>Permissões nativas<strong>NENHUMA</strong></span></div></section>
      </div>
      <p className="settings-note">As preferências alteram apenas os controles desta demonstração e serão restauradas ao recarregar a aplicação.</p>
    </>
  );
}
