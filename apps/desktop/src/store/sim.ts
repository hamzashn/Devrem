import { create } from "zustand";
import {
  isAppError,
  isTauri,
  simAc,
  simDc,
  simDcSweep,
  simSetControl,
  simSetTimeScale,
  simStart,
  simStop,
  type AcResult,
  type AppError,
  type DcSweepResult,
  type Frame,
  type Topology,
} from "../lib/ipc";
import { setLiveProp, setTimeScale, toggleComponent, useEditor } from "./editor";
import { appendScope, resetScope } from "./scope";

/** `idle`: kapalı, `running`: gerçek zamanlı, `dc`: DC çalışma noktası. */
export type SimMode = "idle" | "running" | "dc";

export type AnalysisKind = "ac" | "dcSweep";

interface AnalysisState<T> {
  result: T | null;
  error: AppError | null;
  busy: boolean;
}

interface SimState {
  mode: SimMode;
  topology: Topology | null;
  frame: Frame | null;
  error: AppError | null;
  /** Bileşen kimliği → `topology.elements` indeksi. */
  elementIndex: Map<string, number>;
  ac: AnalysisState<AcResult>;
  dcSweep: AnalysisState<DcSweepResult>;
  /** Basılı tutulan butonlar. */
  pressed: Set<string>;
}

const idleAnalysis = { result: null, error: null, busy: false };

export const useSim = create<SimState>()(() => ({
  mode: "idle",
  topology: null,
  frame: null,
  error: null,
  elementIndex: new Map(),
  ac: idleAnalysis,
  dcSweep: idleAnalysis,
  pressed: new Set(),
}));

const set = useSim.setState;
let session = 0;

function toError(e: unknown): AppError {
  return isAppError(e) ? e : { code: "unknown", message: String(e) };
}

function setTopology(topology: Topology | null) {
  set({
    topology,
    elementIndex: new Map(topology?.elements.map((e, i) => [e.id, i]) ?? []),
  });
}

async function startSession() {
  const token = ++session;
  const doc = useEditor.getState().doc;
  try {
    const topology = await simStart(doc, (msg) => {
      if (token !== session) return;
      if (msg.event === "frame") {
        set({ frame: msg.data.frame });
        appendScope(msg.data.samples);
      } else {
        set({ error: msg.data, frame: null, topology: null });
      }
    });
    if (token !== session) return;
    setTopology(topology);
    resetScope(topology.probes);
    set({ frame: null, error: null });
  } catch (e) {
    if (token !== session) return;
    setTopology(null);
    set({ frame: null, error: toError(e) });
  }
}

async function computeDc() {
  const token = ++session;
  try {
    const result = await simDc(useEditor.getState().doc);
    if (token !== session) return;
    setTopology(result.topology);
    set({ frame: result.frame, error: null });
  } catch (e) {
    if (token !== session) return;
    setTopology(null);
    set({ frame: null, error: toError(e) });
  }
}

export async function startSimulation() {
  if (!isTauri()) {
    set({ error: { code: "browserMode" } });
    return;
  }
  set({ mode: "running", error: null });
  await startSession();
}

export async function stopSimulation() {
  session++;
  set({ mode: "idle", frame: null, error: null, pressed: new Set() });
  setTopology(null);
  if (isTauri()) await simStop();
}

export async function runDcOperatingPoint() {
  if (!isTauri()) {
    set({ error: { code: "browserMode" } });
    return;
  }
  if (useSim.getState().mode === "running") await simStop();
  set({ mode: "dc", error: null });
  await computeDc();
}

export function toggleRunning() {
  return useSim.getState().mode === "running" ? stopSimulation() : startSimulation();
}

/** Anahtarı / lojik girişi değiştirir; simülasyon çalışıyorsa durum korunur. */
export function flipToggle(id: string) {
  const on = toggleComponent(id);
  if (on === null) return;
  const { mode } = useSim.getState();
  if (mode === "running") void simSetControl(id, on ? 1 : 0);
  else if (mode === "dc") void computeDc();
}

/** Butona basar / bırakır (yalnızca simülasyon çalışırken etkilidir). */
export function pressButton(id: string, down: boolean) {
  const pressed = new Set(useSim.getState().pressed);
  if (down) pressed.add(id);
  else pressed.delete(id);
  set({ pressed });
  if (useSim.getState().mode === "running") void simSetControl(id, down ? 1 : 0);
}

/** Potansiyometre konumunu (0–100) simülasyonu yeniden başlatmadan değiştirir. */
export function setPotentiometer(id: string, percent: number) {
  setLiveProp(id, "position", percent);
  const { mode } = useSim.getState();
  if (mode === "running") void simSetControl(id, percent / 100);
  else if (mode === "dc") void computeDc();
}

export function changeTimeScale(scale: number) {
  setTimeScale(scale);
  if (useSim.getState().mode === "running") void simSetTimeScale(scale);
}

/** AC analizi veya DC taramayı çalıştırır. */
export async function runAnalysis(kind: AnalysisKind) {
  if (!isTauri()) {
    set({ [kind]: { result: null, error: { code: "browserMode" }, busy: false } });
    return;
  }
  set((s) => ({ [kind]: { ...s[kind], busy: true, error: null } }));
  const doc = useEditor.getState().doc;
  try {
    const result = kind === "ac" ? await simAc(doc) : await simDcSweep(doc);
    set({ [kind]: { result, error: null, busy: false } });
  } catch (e) {
    set((s) => ({ [kind]: { ...s[kind], error: toError(e), busy: false } }));
  }
}

// Devre değiştiğinde (kısa bir gecikmeyle) simülasyonu yenile.
let restartTimer: ReturnType<typeof setTimeout> | undefined;
useEditor.subscribe((state, prev) => {
  if (state.version === prev.version) return;
  const { mode } = useSim.getState();
  if (mode === "idle") return;
  clearTimeout(restartTimer);
  restartTimer = setTimeout(() => {
    const current = useSim.getState().mode;
    if (current === "running") void startSession();
    else if (current === "dc") void computeDc();
  }, 120);
});
