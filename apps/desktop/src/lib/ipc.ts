import { Channel, invoke, isTauri } from "@tauri-apps/api/core";
import type { CircuitDocument, Point } from "../editor/types";

export { isTauri };

/** Rust tarafındaki `app_info` komutunun dönüş tipi. */
export interface AppInfo {
  appVersion: string;
  engineVersion: string;
  formatVersion: number;
}

/** Rust'tan gelen hata: `code` alanına göre çevrilir. */
export interface AppError {
  code: string;
  [key: string]: unknown;
}

export function isAppError(value: unknown): value is AppError {
  return typeof value === "object" && value !== null && "code" in value;
}

export interface Topology {
  nodeCount: number;
  elements: { id: string; nodes: number[] }[];
  segments: { wireId: string; a: Point; b: Point; node: number }[];
  /** Ölçüm aletleri (voltmetre, ampermetre, lojik prob). */
  probes: Probe[];
}

export type ProbeKind = "voltage" | "current";

export interface Probe {
  id: string;
  kind: ProbeKind;
}

export interface Frame {
  time: number;
  nodeVoltages: number[];
  /** `pinCurrents[e][k]`: bileşenin k. pininden bileşene giren akım. */
  pinCurrents: number[][];
  /** Her tel parçasında a'dan b'ye akan akım. */
  segmentCurrents: number[];
}

/** Osiloskop örnekleri: `values[p][i]`, `topology.probes[p]` aletinin `time[i]` anındaki okuması. */
export interface Samples {
  time: number[];
  values: number[][];
}

export type SimEvent =
  { event: "frame"; data: { frame: Frame; samples: Samples } } | { event: "error"; data: AppError };

export interface AcResult {
  frequencies: number[];
  reference: string;
  probes: { id: string; kind: ProbeKind; magnitudeDb: number[]; phaseDeg: number[] }[];
}

export interface DcSweepResult {
  source: string;
  voltage: boolean;
  values: number[];
  probes: { id: string; kind: ProbeKind; data: number[] }[];
}

/** Uygulama bilgilerini getirir; Tauri dışında (tarayıcıda) `null` döner. */
export async function getAppInfo(): Promise<AppInfo | null> {
  if (!isTauri()) return null;
  return invoke<AppInfo>("app_info");
}

export function openDocumentFile(path: string) {
  return invoke<CircuitDocument>("open_document", { path });
}

export function saveDocumentFile(path: string, document: CircuitDocument) {
  return invoke<void>("save_document", { path, document });
}

export function simStart(document: CircuitDocument, onEvent: (event: SimEvent) => void) {
  const channel = new Channel<SimEvent>();
  channel.onmessage = onEvent;
  return invoke<Topology>("sim_start", { document, onEvent: channel });
}

export function simStop() {
  return invoke<void>("sim_stop");
}

/** Anahtar / buton / lojik giriş için 0 veya 1; potansiyometre için 0–1 konum. */
export function simSetControl(id: string, value: number) {
  return invoke<void>("sim_set_control", { id, value });
}

export function exportSpice(document: CircuitDocument) {
  return invoke<string>("export_spice", { document });
}

export function writeTextFile(path: string, contents: string) {
  return invoke<void>("write_text_file", { path, contents });
}

export function writeBinaryFile(path: string, base64: string) {
  return invoke<void>("write_binary_file", { path, base64 });
}

export function simSetTimeScale(scale: number) {
  return invoke<void>("sim_set_time_scale", { scale });
}

export function simDc(document: CircuitDocument) {
  return invoke<{ topology: Topology; frame: Frame }>("sim_dc", { document });
}

export function simAc(document: CircuitDocument) {
  return invoke<AcResult>("sim_ac", { document });
}

export function simDcSweep(document: CircuitDocument) {
  return invoke<DcSweepResult>("sim_dc_sweep", { document });
}
