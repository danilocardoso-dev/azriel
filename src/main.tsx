import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import App from "./App";
import { FloatingOrb } from "./components/layout/FloatingOrb";
import { DataProvider } from "./contexts/DataContext";
import { DailyOperationsProvider } from "./contexts/DailyOperationsContext";
import { AIProvider } from "./contexts/AIContext";
import { SystemProvider } from "./contexts/SystemContext";
import { AutomationProvider } from "./contexts/AutomationContext";
import "./styles/global.css";

const isOrbWindow = getCurrentWindow().label === "orb";
if (isOrbWindow) document.documentElement.classList.add("orb-window");

createRoot(document.getElementById("root")!).render(isOrbWindow ? (
  <StrictMode><FloatingOrb /></StrictMode>
) : (
  <StrictMode>
    <DataProvider><DailyOperationsProvider><SystemProvider><AutomationProvider><AIProvider><App /></AIProvider></AutomationProvider></SystemProvider></DailyOperationsProvider></DataProvider>
  </StrictMode>
));
