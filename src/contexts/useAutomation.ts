import { useContext } from "react";
import { AutomationContext } from "./automation-context";

export function useAutomation() {
  const value = useContext(AutomationContext);
  if (!value) throw new Error("useAutomation precisa estar dentro de AutomationProvider");
  return value;
}
