import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Toolbar } from "./components/Toolbar";
import { Sidebar } from "./components/Sidebar";
import { Canvas } from "./components/Canvas";
import { AnalysisPanel } from "./components/AnalysisPanel";
import { PropertiesPanel } from "./components/PropertiesPanel";
import { StatusBar } from "./components/StatusBar";
import { Gallery } from "./components/Gallery";
import { isTauri } from "./lib/ipc";
import { installShortcuts } from "./lib/shortcuts";
import { checkForUpdates } from "./lib/updater";
import { useEditor } from "./store/editor";

export default function App() {
  const { i18n } = useTranslation();
  const name = useEditor((s) => s.doc.meta.name);
  const dirty = useEditor((s) => s.dirty);

  useEffect(() => {
    document.documentElement.lang = i18n.language;
  }, [i18n.language]);

  useEffect(installShortcuts, []);
  useEffect(() => void checkForUpdates(), []);

  useEffect(() => {
    const title = `${dirty ? "● " : ""}${name} — Devrem`;
    document.title = title;
    if (isTauri()) void getCurrentWindow().setTitle(title);
  }, [name, dirty]);

  return (
    <div className="flex h-full flex-col bg-base text-primary">
      <Toolbar />
      <div className="flex min-h-0 flex-1">
        <Sidebar />
        <div className="flex min-w-0 flex-1 flex-col">
          <Canvas />
          <AnalysisPanel />
        </div>
        <PropertiesPanel />
      </div>
      <StatusBar />
      <Gallery />
    </div>
  );
}
