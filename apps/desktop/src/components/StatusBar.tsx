import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { describeError } from "../lib/errors";
import { getAppInfo, type AppInfo } from "../lib/ipc";
import { formatValue } from "../editor/units";
import { useEditor } from "../store/editor";
import { useNotice } from "../store/notice";
import { useSim } from "../store/sim";
import { installUpdate, useUpdate } from "../lib/updater";

export function StatusBar() {
  const { t } = useTranslation();
  const [info, setInfo] = useState<AppInfo | null>(null);
  const mode = useSim((s) => s.mode);
  const error = useSim((s) => s.error);
  const time = useSim((s) => s.frame?.time);
  const notice = useNotice((s) => s.notice);
  const zoom = useEditor((s) => s.view.zoom);
  const cursor = useEditor((s) => s.cursor);
  const update = useUpdate((s) => s.update);
  const installing = useUpdate((s) => s.installing);

  useEffect(() => {
    getAppInfo()
      .then(setInfo)
      .catch((err: unknown) => console.error("app_info başarısız:", err));
  }, []);

  let dot = "bg-success shadow-[0_0_6px_var(--success)]";
  let text = t("status.ready");
  if (error) {
    dot = "bg-error shadow-[0_0_6px_var(--error)]";
    text = describeError(error);
  } else if (mode === "running") {
    dot = "bg-accent shadow-[0_0_6px_var(--accent)] animate-pulse";
    text = `${t("status.running")} · t = ${formatValue(time ?? 0, "s", 4)}`;
  } else if (mode === "dc") {
    dot = "bg-accent shadow-[0_0_6px_var(--accent)]";
    text = t("status.dc");
  }

  return (
    <footer className="flex h-7 shrink-0 items-center gap-4 border-t border-border bg-surface px-3 text-[11px] text-secondary">
      <span className={`flex min-w-0 items-center gap-1.5 ${error ? "text-error" : ""}`}>
        <span className={`size-1.5 shrink-0 rounded-full ${dot}`} />
        <span className="truncate">{text}</span>
      </span>
      {notice && (
        <span className={`truncate ${notice.kind === "error" ? "text-error" : "text-accent"}`}>
          {notice.text}
        </span>
      )}
      {update && (
        <button
          type="button"
          onClick={() => void installUpdate()}
          disabled={installing}
          className="shrink-0 rounded border border-accent/50 px-2 py-0.5 text-accent hover:bg-accent/10 disabled:opacity-60"
        >
          {installing
            ? t("update.installing")
            : `${t("update.available", { version: update.version })} — ${t("update.install")}`}
        </button>
      )}
      <span className="ml-auto flex shrink-0 items-center gap-4 font-mono">
        <span>
          {cursor[0]}, {cursor[1]}
        </span>
        <span>{Math.round(zoom * 100)}%</span>
        {info ? (
          <span>
            {t("status.engine")} v{info.engineVersion}
          </span>
        ) : (
          <span>{t("status.browser")}</span>
        )}
      </span>
    </footer>
  );
}
