import { useContext } from "react";
import { DataContext } from "./data-context";

export function useAzrielData() {
  const context = useContext(DataContext);
  if (!context) throw new Error("useAzrielData precisa estar dentro de DataProvider");
  return context;
}
