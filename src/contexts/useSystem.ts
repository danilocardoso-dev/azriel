import { useContext } from "react";
import { SystemContext } from "./system-context";

export function useSystem() {
  const context = useContext(SystemContext);
  if (!context) throw new Error("useSystem precisa estar dentro de SystemProvider");
  return context;
}
