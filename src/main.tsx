import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App";
import { DataProvider } from "./contexts/DataContext";
import { DailyOperationsProvider } from "./contexts/DailyOperationsContext";
import { AIProvider } from "./contexts/AIContext";
import "./styles/global.css";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <DataProvider><DailyOperationsProvider><AIProvider><App /></AIProvider></DailyOperationsProvider></DataProvider>
  </StrictMode>,
);
