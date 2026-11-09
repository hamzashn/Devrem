import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import type uPlot from "uplot";
import { describeError } from "../lib/errors";
import { exportCsv } from "../lib/export";
import type { AppError, ProbeKind } from "../lib/ipc";
import { formatValue } from "../editor/units";
import { updateSimulationSettings, useEditor } from "../store/editor";
import { scope, subscribeScope } from "../store/scope";
import { useSettings, type PanelTab } from "../store/settings";
import { runAnalysis, useSim } from "../store/sim";
import { CHANNEL_COLORS, axis } from "../lib/chart";
import { Chart } from "./Chart";
import { SiInput } from "./SiInput";

const TABS: PanelTab[] = ["scope", "ac", "dcSweep"];
const unitOf = (kind: ProbeKind) => (kind === "current" ? "A" : "V");
const color = (i: number) => CHANNEL_COLORS[i % CHANNEL_COLORS.length]!;

export function AnalysisPanel() {
  const { t } = useTranslation();
  const open = useSettings((s) => s.panelOpen);
  const tab = useSettings((s) => s.panelTab);

  return (
    <section
      className={`flex shrink-0 flex-col border-t border-border bg-surface ${open ? "h-72" : ""}`}
    >
      <div className="flex h-9 shrink-0 items-center gap-1 px-2">
        {TABS.map((id) => (
          <button
            key={id}
            type="button"
            onClick={() => useSettings.setState({ panelTab: id, panelOpen: true })}
            className={`rounded-md px-2.5 py-1 text-[12px] transition-colors ${
              tab === id && open
                ? "bg-elevated text-accent"
                : "text-secondary hover:bg-elevated hover:text-primary"
            }`}
          >
            {t(`panel.${id}`)}
          </button>
        ))}
        {open && <CsvButton tab={tab} />}
        <button
          type="button"
          onClick={() => useSettings.setState({ panelOpen: !open })}
          title={open ? t("panel.hide") : t("panel.show")}
          aria-label={open ? t("panel.hide") : t("panel.show")}
          className={`${open ? "" : "ml-auto "}rounded-md px-2 py-1 text-secondary hover:bg-elevated hover:text-primary`}
        >
          <span
            className={`inline-block text-[10px] transition-transform ${open ? "" : "rotate-180"}`}
          >
            ▼
          </span>
        </button>
      </div>
      {open && (
        <div className="min-h-0 flex-1 px-3 pb-2">
          {tab === "scope" && <ScopePanel />}
          {tab === "ac" && <AcPanel />}
          {tab === "dcSweep" && <DcSweepPanel />}
        </div>
      )}
    </section>
  );
}

/** Etkin sekmenin verisini CSV olarak dışa aktarır. */
function CsvButton({ tab }: { tab: PanelTab }) {
  const { t } = useTranslation();
  const ac = useSim((s) => s.ac.result);
  const sweep = useSim((s) => s.dcSweep.result);
  const mode = useSim((s) => s.mode);

  const exportTab = () => {
    if (tab === "scope") {
      if (!scope.time.length) return;
      const unit = (k: ProbeKind) => unitOf(k);
      void exportCsv(
        t("panel.scope"),
        ["time_s", ...scope.probes.map((p) => `${p.id}_${unit(p.kind)}`)],
        [scope.time, ...scope.values],
      );
    } else if (tab === "ac" && ac) {
      void exportCsv(
        t("panel.ac"),
        ["frequency_Hz", ...ac.probes.flatMap((p) => [`${p.id}_dB`, `${p.id}_deg`])],
        [ac.frequencies, ...ac.probes.flatMap((p) => [p.magnitudeDb, p.phaseDeg])],
      );
    } else if (tab === "dcSweep" && sweep) {
      const xUnit = sweep.voltage ? "V" : "A";
      void exportCsv(
        t("panel.dcSweep"),
        [`${sweep.source}_${xUnit}`, ...sweep.probes.map((p) => `${p.id}_${unitOf(p.kind)}`)],
        [sweep.values, ...sweep.probes.map((p) => p.data)],
      );
    }
  };

  const available =
    (tab === "scope" && (mode === "running" || scope.time.length > 0)) ||
    (tab === "ac" && ac !== null) ||
    (tab === "dcSweep" && sweep !== null);

  return (
    <button
      type="button"
      onClick={exportTab}
      disabled={!available}
      title={t("panel.exportCsv")}
      className="ml-auto rounded-md border border-border px-2 py-0.5 font-mono text-[11px] text-secondary transition hover:border-accent/50 hover:text-accent disabled:pointer-events-none disabled:opacity-30"
    >
      CSV
    </button>
  );
}

function Message({ children, error }: { children: React.ReactNode; error?: boolean }) {
  return (
    <div
      className={`flex h-full items-center justify-center text-center text-[12px] ${
        error ? "text-error" : "text-secondary"
      }`}
    >
      {children}
    </div>
  );
}

// ---------------------------------------------------------------- Osiloskop

const SCOPE_WINDOWS = [0, 1e-3, 10e-3, 0.1, 1, 10];

function ScopePanel() {
  const { t } = useTranslation();
  const mode = useSim((s) => s.mode);
  const topology = useSim((s) => s.topology);
  const timeScale = useEditor((s) => s.doc.simulation.timeScale);
  const windowSetting = useSettings((s) => s.scopeWindow);
  const plotRef = useRef<uPlot | null>(null);
  const [probes, setProbes] = useState(scope.probes);

  const span = windowSetting > 0 ? windowSetting : 3 * timeScale;

  const options = useMemo<Omit<uPlot.Options, "width" | "height">>(() => {
    const hasCurrent = probes.some((p) => p.kind === "current");
    const hasVoltage = probes.some((p) => p.kind === "voltage");
    return {
      scales: { x: { time: false }, V: {}, A: {} },
      legend: { live: true },
      cursor: { drag: { x: false, y: false } },
      series: [
        { label: t("panel.time"), value: (_u, v) => (v == null ? "—" : formatValue(v, "s", 4)) },
        ...probes.map((p, i) => ({
          label: p.id,
          scale: unitOf(p.kind),
          stroke: color(i),
          width: 1.5,
          value: (_u: uPlot, v: number | null) =>
            v == null ? "—" : formatValue(v, unitOf(p.kind), 4),
        })),
      ],
      axes: [
        axis("s"),
        ...(hasVoltage || !hasCurrent ? [axis("V", { scale: "V" })] : []),
        ...(hasCurrent ? [axis("A", { scale: "A", side: 1, grid: { show: false } })] : []),
      ],
    };
  }, [probes, t]);

  const onReady = useCallback((plot: uPlot | null) => {
    plotRef.current = plot;
  }, []);

  useEffect(() => {
    const redraw = () => {
      if (scope.probes !== probes) {
        setProbes(scope.probes);
        return;
      }
      const plot = plotRef.current;
      if (!plot) return;
      const time = scope.time;
      const last = time.at(-1) ?? 0;
      let start = time.length;
      while (start > 0 && time[start - 1]! >= last - span) start--;
      const data = [time.slice(start), ...scope.values.map((v) => v.slice(start))];
      plot.setData(data as uPlot.AlignedData, false);
      plot.setScale("x", { min: Math.max(0, last - span), max: Math.max(last, span) });
    };
    redraw();
    return subscribeScope(redraw);
  }, [probes, span]);

  const windowSelect = (
    <label className="flex items-center gap-1.5 text-[11px] text-secondary">
      {t("panel.window")}
      <select
        value={windowSetting}
        onChange={(e) => useSettings.setState({ scopeWindow: Number(e.target.value) })}
        className="rounded-md border border-border bg-base px-1.5 py-0.5 font-mono text-[11px] text-primary focus:border-accent focus:outline-none"
      >
        {SCOPE_WINDOWS.map((w) => (
          <option key={w} value={w}>
            {w === 0
              ? `${t("panel.auto")} (${formatValue(3 * timeScale, "s")})`
              : formatValue(w, "s")}
          </option>
        ))}
      </select>
    </label>
  );

  if (mode === "running" && topology && topology.probes.length === 0) {
    return <Message>{t("panel.noProbes")}</Message>;
  }
  if (probes.length === 0) {
    return <Message>{mode === "running" ? t("panel.noProbes") : t("panel.scopeIdle")}</Message>;
  }
  return (
    <div className="flex h-full flex-col gap-1">
      <div className="flex justify-end">{windowSelect}</div>
      <div className="min-h-0 flex-1">
        <Chart options={options} onReady={onReady} />
      </div>
    </div>
  );
}

// ---------------------------------------------------------------- Ortak düzen

function AnalysisLayout({
  settings,
  hint,
  busy,
  error,
  hasResult,
  onRun,
  children,
}: {
  settings: React.ReactNode;
  hint: string;
  busy: boolean;
  error: AppError | null;
  hasResult: boolean;
  onRun: () => void;
  children: React.ReactNode;
}) {
  const { t } = useTranslation();
  return (
    <div className="flex h-full gap-4">
      <div className="flex w-52 shrink-0 flex-col gap-2 overflow-y-auto pt-1">
        {settings}
        <button
          type="button"
          onClick={onRun}
          disabled={busy}
          className="mt-1 rounded-md bg-accent px-3 py-1.5 font-medium text-[var(--bg-base)] shadow-[0_0_14px_var(--accent-glow)] transition hover:brightness-110 disabled:opacity-50"
        >
          {busy ? t("panel.busy") : t("panel.run")}
        </button>
        <p className="text-[11px] leading-snug text-secondary">{hint}</p>
      </div>
      <div className="min-w-0 flex-1">
        {error ? (
          <Message error>{describeError(error)}</Message>
        ) : hasResult ? (
          children
        ) : (
          <Message>{t("panel.empty")}</Message>
        )}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------- AC analizi

function AcPanel() {
  const { t } = useTranslation();
  const settings = useEditor((s) => s.doc.simulation.ac);
  const { result, error, busy } = useSim((s) => s.ac);

  const options = useMemo<Omit<uPlot.Options, "width" | "height">>(() => {
    const probes = result?.probes ?? [];
    return {
      scales: { x: { time: false, distr: 3 }, dB: {}, deg: {} },
      legend: { live: true },
      cursor: { drag: { x: true, y: false } },
      series: [
        {
          label: t("panel.frequency"),
          value: (_u, v) => (v == null ? "—" : formatValue(v, "Hz", 4)),
        },
        ...probes.flatMap((p, i) => [
          {
            label: `${p.id} |H|`,
            scale: "dB",
            stroke: color(i),
            width: 1.75,
            value: (_u: uPlot, v: number | null) => (v == null ? "—" : `${v.toFixed(2)} dB`),
          },
          {
            label: `${p.id} ∠`,
            scale: "deg",
            stroke: color(i),
            width: 1.25,
            dash: [5, 4],
            value: (_u: uPlot, v: number | null) => (v == null ? "—" : `${v.toFixed(1)}°`),
          },
        ]),
      ],
      axes: [
        axis("Hz"),
        axis("", {
          scale: "dB",
          label: t("panel.magnitude"),
          values: (_u, s) => s.map((v) => (v == null ? "" : `${v}`)),
        }),
        axis("", {
          scale: "deg",
          side: 1,
          label: t("panel.phase"),
          grid: { show: false },
          values: (_u, s) => s.map((v) => (v == null ? "" : `${v}°`)),
        }),
      ],
    };
  }, [result, t]);

  const data = useMemo(() => {
    if (!result) return [[]] as uPlot.AlignedData;
    return [
      result.frequencies,
      ...result.probes.flatMap((p) => [p.magnitudeDb, p.phaseDeg]),
    ] as uPlot.AlignedData;
  }, [result]);

  const update = (patch: Partial<typeof settings>) =>
    updateSimulationSettings((s) => ({ ...s, ac: { ...s.ac, ...patch } }));

  return (
    <AnalysisLayout
      settings={
        <>
          <div className="grid grid-cols-2 gap-2">
            <SiInput
              label={t("panel.start")}
              unit="Hz"
              value={settings.start}
              onCommit={(start) => update({ start })}
            />
            <SiInput
              label={t("panel.stop")}
              unit="Hz"
              value={settings.stop}
              onCommit={(stop) => update({ stop })}
            />
          </div>
          <SiInput
            label={t("panel.pointsPerDecade")}
            integer
            value={settings.pointsPerDecade}
            onCommit={(pointsPerDecade) => update({ pointsPerDecade })}
          />
        </>
      }
      hint={t("panel.acHint")}
      busy={busy}
      error={error}
      hasResult={!!result}
      onRun={() => void runAnalysis("ac")}
    >
      <Chart options={options} data={data} />
    </AnalysisLayout>
  );
}

// ---------------------------------------------------------------- DC tarama

function DcSweepPanel() {
  const { t } = useTranslation();
  const settings = useEditor((s) => s.doc.simulation.dcSweep);
  const components = useEditor((s) => s.doc.components);
  const { result, error, busy } = useSim((s) => s.dcSweep);

  const sources = components.filter((c) => c.kind === "dcVoltage" || c.kind === "dcCurrent");
  const selected = sources.find((c) => c.id === settings.source);
  const sourceUnit = selected?.kind === "dcCurrent" ? "A" : "V";

  const update = (patch: Partial<typeof settings>) =>
    updateSimulationSettings((s) => ({ ...s, dcSweep: { ...s.dcSweep, ...patch } }));

  const options = useMemo<Omit<uPlot.Options, "width" | "height">>(() => {
    const probes = result?.probes ?? [];
    const xUnit = result?.voltage === false ? "A" : "V";
    const hasCurrent = probes.some((p) => p.kind === "current");
    const hasVoltage = probes.some((p) => p.kind === "voltage");
    return {
      scales: { x: { time: false }, V: {}, A: {} },
      legend: { live: true },
      cursor: { drag: { x: true, y: false } },
      series: [
        {
          label: result?.source ?? "",
          value: (_u, v) => (v == null ? "—" : formatValue(v, xUnit, 4)),
        },
        ...probes.map((p, i) => ({
          label: p.id,
          scale: unitOf(p.kind),
          stroke: color(i),
          width: 1.75,
          value: (_u: uPlot, v: number | null) =>
            v == null ? "—" : formatValue(v, unitOf(p.kind), 4),
        })),
      ],
      axes: [
        axis(xUnit),
        ...(hasVoltage ? [axis("V", { scale: "V" })] : []),
        ...(hasCurrent
          ? [axis("A", { scale: "A", side: hasVoltage ? 1 : 3, grid: { show: !hasVoltage } })]
          : []),
      ],
    };
  }, [result]);

  const data = useMemo(() => {
    if (!result) return [[]] as uPlot.AlignedData;
    return [result.values, ...result.probes.map((p) => p.data)] as uPlot.AlignedData;
  }, [result]);

  return (
    <AnalysisLayout
      settings={
        <>
          <label className="flex flex-col gap-1">
            <span className="text-[10px] font-medium tracking-wide text-secondary uppercase">
              {t("panel.source")}
            </span>
            <select
              value={selected ? settings.source : ""}
              onChange={(e) => update({ source: e.target.value })}
              className="rounded-md border border-border bg-base px-2 py-1 text-[12px] text-primary focus:border-accent focus:outline-none"
            >
              <option value="">{t("panel.selectSource")}</option>
              {sources.map((c) => (
                <option key={c.id} value={c.id}>
                  {c.id} — {t(`components.${c.kind}`)}
                </option>
              ))}
            </select>
          </label>
          <div className="grid grid-cols-2 gap-2">
            <SiInput
              label={t("panel.start")}
              unit={sourceUnit}
              value={settings.start}
              onCommit={(start) => update({ start })}
            />
            <SiInput
              label={t("panel.stop")}
              unit={sourceUnit}
              value={settings.stop}
              onCommit={(stop) => update({ stop })}
            />
          </div>
          <SiInput
            label={t("panel.steps")}
            integer
            value={settings.steps}
            onCommit={(steps) => update({ steps })}
          />
        </>
      }
      hint={t("panel.sweepHint")}
      busy={busy}
      error={error}
      hasResult={!!result}
      onRun={() => void runAnalysis("dcSweep")}
    >
      <Chart options={options} data={data} />
    </AnalysisLayout>
  );
}
