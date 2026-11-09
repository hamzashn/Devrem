import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { LANGUAGES } from "../i18n";
import { exportPng, exportSpiceFile, exportSvg } from "../lib/export";
import { newDocument, openDocument, saveDocument } from "../lib/files";
import { openGallery } from "../store/ui";
import { fitToContent, zoomBy } from "../lib/view";
import { deleteSelected, redo, rotateSelected, selectAll, undo, useEditor } from "../store/editor";
import { useSettings } from "../store/settings";
import {
  changeTimeScale,
  runDcOperatingPoint,
  stopSimulation,
  toggleRunning,
  useSim,
} from "../store/sim";
import { Logo } from "./Logo";

interface MenuItem {
  label: string;
  shortcut?: string;
  action: () => unknown;
  disabled?: boolean;
}

const TIME_SCALES = [1, 0.1, 0.01, 1e-3, 1e-4, 1e-5];

function timeScaleLabel(scale: number): string {
  return scale >= 1 ? "1×" : `1/${Math.round(1 / scale)}×`;
}

export function Toolbar() {
  const { t } = useTranslation();
  const language = useSettings((s) => s.language);
  const setLanguage = useSettings((s) => s.setLanguage);
  const mode = useSim((s) => s.mode);
  const canUndo = useEditor((s) => s.past.length > 0);
  const canRedo = useEditor((s) => s.future.length > 0);
  const timeScale = useEditor((s) => s.doc.simulation.timeScale);
  const running = mode === "running";

  const menus: { id: "file" | "edit" | "view" | "simulate"; items: (MenuItem | "separator")[] }[] =
    [
      {
        id: "file",
        items: [
          { label: t("menu.new"), shortcut: "Ctrl+N", action: newDocument },
          { label: t("menu.open"), shortcut: "Ctrl+O", action: openDocument },
          { label: t("menu.examples"), action: openGallery },
          "separator",
          { label: t("menu.save"), shortcut: "Ctrl+S", action: () => saveDocument() },
          { label: t("menu.saveAs"), shortcut: "Ctrl+Shift+S", action: () => saveDocument(true) },
          "separator",
          { label: t("menu.exportSpice"), action: exportSpiceFile },
          { label: t("menu.exportPng"), action: exportPng },
          { label: t("menu.exportSvg"), action: exportSvg },
        ],
      },
      {
        id: "edit",
        items: [
          { label: t("menu.undo"), shortcut: "Ctrl+Z", action: undo, disabled: !canUndo },
          { label: t("menu.redo"), shortcut: "Ctrl+Y", action: redo, disabled: !canRedo },
          "separator",
          { label: t("menu.rotate"), shortcut: "R", action: rotateSelected },
          { label: t("menu.delete"), shortcut: "Del", action: deleteSelected },
          { label: t("menu.selectAll"), shortcut: "Ctrl+A", action: selectAll },
        ],
      },
      {
        id: "view",
        items: [
          { label: t("menu.zoomIn"), shortcut: "Ctrl++", action: () => zoomBy(1.25) },
          { label: t("menu.zoomOut"), shortcut: "Ctrl+-", action: () => zoomBy(0.8) },
          { label: t("menu.fit"), shortcut: "Ctrl+0", action: fitToContent },
        ],
      },
      {
        id: "simulate",
        items: [
          {
            label: running ? t("menu.stop") : t("menu.run"),
            shortcut: "F5",
            action: toggleRunning,
          },
          { label: t("menu.dcOp"), action: runDcOperatingPoint },
          { label: t("menu.clearResults"), action: stopSimulation, disabled: mode === "idle" },
        ],
      },
    ];

  return (
    <header className="flex h-11 shrink-0 items-center gap-1 border-b border-border bg-surface px-3">
      <div className="mr-3 flex items-center gap-2">
        <Logo />
        <span className="text-[14px] font-semibold tracking-tight">Devrem</span>
      </div>

      <nav className="flex items-center">
        {menus.map((menu) => (
          <Menu key={menu.id} label={t(`toolbar.${menu.id}`)} items={menu.items} />
        ))}
      </nav>

      <div className="ml-auto flex items-center gap-2">
        <div
          role="group"
          aria-label={t("toolbar.language")}
          className="flex rounded-md border border-border p-0.5"
        >
          {LANGUAGES.map((lng) => (
            <button
              key={lng}
              type="button"
              onClick={() => setLanguage(lng)}
              aria-pressed={language === lng}
              className={`rounded px-2 py-0.5 font-mono text-[11px] uppercase transition-colors ${
                language === lng ? "bg-elevated text-accent" : "text-secondary hover:text-primary"
              }`}
            >
              {lng}
            </button>
          ))}
        </div>

        <label className="flex items-center gap-1.5 text-secondary" title={t("toolbar.speedHint")}>
          <span className="text-[11px]">{t("toolbar.speed")}</span>
          <select
            value={timeScale}
            onChange={(e) => changeTimeScale(Number(e.target.value))}
            className="rounded-md border border-border bg-base px-1.5 py-1 font-mono text-[11px] text-primary focus:border-accent focus:outline-none"
          >
            {TIME_SCALES.map((s) => (
              <option key={s} value={s}>
                {timeScaleLabel(s)}
              </option>
            ))}
          </select>
        </label>

        <button
          type="button"
          onClick={() => void toggleRunning()}
          title="F5"
          className={`flex items-center gap-1.5 rounded-md px-3 py-1 font-medium transition-all ${
            running
              ? "bg-error/15 text-error ring-1 ring-error/40 hover:bg-error/25"
              : "bg-accent text-[var(--bg-base)] shadow-[0_0_16px_var(--accent-glow)] hover:brightness-110"
          }`}
        >
          {running ? (
            <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden>
              <rect x="1" y="1" width="8" height="8" rx="1" fill="currentColor" />
            </svg>
          ) : (
            <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden>
              <path d="M1 0.5 L9 5 L1 9.5 Z" fill="currentColor" />
            </svg>
          )}
          {running ? t("toolbar.stop") : t("toolbar.run")}
        </button>
      </div>
    </header>
  );
}

function Menu({ label, items }: { label: string; items: (MenuItem | "separator")[] }) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const close = (e: PointerEvent) => {
      if (!ref.current?.contains(e.target as Node)) setOpen(false);
    };
    const esc = (e: KeyboardEvent) => e.key === "Escape" && setOpen(false);
    window.addEventListener("pointerdown", close);
    window.addEventListener("keydown", esc);
    return () => {
      window.removeEventListener("pointerdown", close);
      window.removeEventListener("keydown", esc);
    };
  }, [open]);

  return (
    <div ref={ref} className="relative">
      <button
        type="button"
        onClick={() => setOpen((o) => !o)}
        aria-expanded={open}
        className={`rounded-md px-2.5 py-1 transition-colors ${
          open ? "bg-elevated text-primary" : "text-secondary hover:bg-elevated hover:text-primary"
        }`}
      >
        {label}
      </button>
      {open && (
        <div
          role="menu"
          className="absolute top-full left-0 z-20 mt-1 min-w-56 rounded-lg border border-border bg-elevated p-1 shadow-2xl shadow-black/50"
        >
          {items.map((item, i) =>
            item === "separator" ? (
              <div key={i} className="my-1 h-px bg-border" />
            ) : (
              <button
                key={item.label}
                type="button"
                role="menuitem"
                disabled={item.disabled}
                onClick={() => {
                  setOpen(false);
                  void item.action();
                }}
                className="flex w-full items-center justify-between gap-6 rounded-md px-2.5 py-1.5 text-left text-primary transition-colors hover:bg-accent/10 hover:text-accent disabled:pointer-events-none disabled:opacity-40"
              >
                <span>{item.label}</span>
                {item.shortcut && (
                  <span className="font-mono text-[10px] text-secondary">{item.shortcut}</span>
                )}
              </button>
            ),
          )}
        </div>
      )}
    </div>
  );
}
