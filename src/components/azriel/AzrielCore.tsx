import type { AzrielState } from "../../types";
import { azrielStates } from "../../data/system";

interface AzrielCoreProps {
  state: AzrielState;
  onClick: () => void;
  compact?: boolean;
}

export function AzrielCore({ state, onClick, compact = false }: AzrielCoreProps) {
  return (
    <button
      className={`azriel-core azriel-core--${state} ${compact ? "azriel-core--compact" : ""}`}
      onClick={onClick}
      aria-label={`Abrir estado do Azriel. Estado atual: ${azrielStates[state].label}`}
    >
      <span className="azriel-core__ticks" />
      <span className="azriel-core__ring azriel-core__ring--one" />
      <span className="azriel-core__ring azriel-core__ring--two" />
      <span className="azriel-core__ring azriel-core__ring--three" />
      <span className="azriel-core__ring azriel-core__ring--four" />
      <span className="azriel-core__center">
        <strong>AZRIEL</strong>
        <small>{azrielStates[state].label}</small>
      </span>
    </button>
  );
}
