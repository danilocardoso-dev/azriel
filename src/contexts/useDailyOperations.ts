import { useContext } from "react";
import { DailyContext } from "./daily-context";

export function useDailyOperations() {
  const context = useContext(DailyContext);
  if (!context) throw new Error("useDailyOperations precisa estar dentro de DailyOperationsProvider");
  return context;
}
