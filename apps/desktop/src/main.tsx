import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "@fontsource-variable/inter";
import "@fontsource-variable/jetbrains-mono";
import "./i18n";
import "./index.css";
import App from "./App";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { writePng, writeSpice, writeSvg } from "./lib/export";
import { fitToContent } from "./lib/view";
import { useEditor } from "./store/editor";
import { useSettings } from "./store/settings";
import { runAnalysis, startSimulation, stopSimulation, useSim } from "./store/sim";

// Geliştirme modunda durumu tarayıcı konsolundan incelemek için.
if (import.meta.env.DEV) {
  Object.assign(window, {
    __devrem: {
      editor: useEditor,
      sim: useSim,
      settings: useSettings,
      startSimulation,
      stopSimulation,
      runAnalysis,
      fitToContent,
      writePng,
      writeSpice,
      writeSvg,
    },
  });
}

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <ErrorBoundary>
      <App />
    </ErrorBoundary>
  </StrictMode>,
);
