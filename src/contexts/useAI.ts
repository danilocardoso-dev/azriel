import { useContext } from "react";
import { AIContext } from "./ai-context";

export function useAI() {
  const context = useContext(AIContext);
  if (!context) throw new Error("useAI deve ser utilizado dentro de AIProvider");
  return context;
}
