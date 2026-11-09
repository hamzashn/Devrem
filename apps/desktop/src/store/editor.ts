import { create } from "zustand";
import i18n from "../i18n";
import * as ops from "../editor/ops";
import { TOGGLE_PROP } from "../editor/library";
import { EMPTY_SELECTION, type Selection } from "../editor/ops";
import {
  emptyDocument,
  normalizeDocument,
  type CircuitDocument,
  type ComponentKind,
  type Point,
  type PropValue,
  type SimulationSettings,
} from "../editor/types";

export type Tool = { type: "select" } | { type: "place"; kind: ComponentKind; rot: number };

/** Ekran = dünya × GRID × zoom + (x, y) */
export interface View {
  x: number;
  y: number;
  zoom: number;
}

export const GRID = 16;
const HISTORY_LIMIT = 200;

interface EditorState {
  doc: CircuitDocument;
  filePath: string | null;
  dirty: boolean;
  /** Devre yapısını etkileyen her değişiklikte artar (simülasyonu yeniden başlatır). */
  version: number;
  past: CircuitDocument[];
  future: CircuitDocument[];
  selection: Selection;
  tool: Tool;
  view: View;
  /** Çizilmekte olan telin köşe noktaları. */
  wireDraft: Point[] | null;
  /** İmlecin ızgaradaki konumu. */
  cursor: Point;
}

export const useEditor = create<EditorState>()(() => ({
  doc: emptyDocument(i18n.t("app.untitled")),
  filePath: null,
  dirty: false,
  version: 0,
  past: [],
  future: [],
  selection: EMPTY_SELECTION,
  tool: { type: "select" },
  view: { x: 0, y: 0, zoom: 1 },
  wireDraft: null,
  cursor: [0, 0],
}));

const get = useEditor.getState;
const set = useEditor.setState;

// ---------------------------------------------------------------- Geçmiş

/** Belgeyi değiştirir ve önceki halini geçmişe ekler. */
export function commit(next: CircuitDocument, previous = get().doc) {
  if (next === previous && next === get().doc) return;
  set((s) => ({
    doc: next,
    past: [...s.past, previous].slice(-HISTORY_LIMIT),
    future: [],
    dirty: true,
    version: s.version + 1,
  }));
}

/** Geçmişe eklemeden değiştirir (sürükleme sırasında). */
export function replace(next: CircuitDocument) {
  set((s) => ({ doc: next, dirty: true, version: s.version + 1 }));
}

function pruneSelection(doc: CircuitDocument, sel: Selection): Selection {
  const comps = new Set(doc.components.map((c) => c.id));
  const wires = new Set(doc.wires.map((w) => w.id));
  return {
    components: sel.components.filter((id) => comps.has(id)),
    wires: sel.wires.filter((id) => wires.has(id)),
  };
}

export function undo() {
  const { past, doc, future, selection } = get();
  const previous = past.at(-1);
  if (!previous) return;
  set((s) => ({
    doc: previous,
    past: past.slice(0, -1),
    future: [doc, ...future],
    dirty: true,
    version: s.version + 1,
    selection: pruneSelection(previous, selection),
    wireDraft: null,
  }));
}

export function redo() {
  const { past, doc, future, selection } = get();
  const next = future[0];
  if (!next) return;
  set((s) => ({
    doc: next,
    past: [...past, doc],
    future: future.slice(1),
    dirty: true,
    version: s.version + 1,
    selection: pruneSelection(next, selection),
    wireDraft: null,
  }));
}

// --------------------------------------------------------------- İşlemler

export function placeComponent(kind: ComponentKind, pos: Point, rot = 0) {
  const [next, id] = ops.addComponent(get().doc, kind, pos, rot);
  commit(next);
  set({ selection: { components: [id], wires: [] } });
}

export function addWirePath(points: Point[]) {
  commit(ops.addWirePath(get().doc, points));
}

export function deleteSelected() {
  const { doc, selection } = get();
  if (!selection.components.length && !selection.wires.length) return;
  commit(ops.deleteSelection(doc, selection));
  set({ selection: EMPTY_SELECTION });
}

export function rotateSelected() {
  const { doc, selection } = get();
  commit(ops.rotateSelection(doc, selection));
}

export function setProp(id: string, key: string, value: PropValue) {
  commit(
    ops.updateComponent(get().doc, id, (c) => ({ ...c, props: { ...c.props, [key]: value } })),
  );
}

/**
 * Anahtarı veya lojik girişi değiştirir. Devre yapısı değişmediği için
 * simülasyon yeniden başlatılmaz (kondansatör yükleri korunur) ve geri alma
 * geçmişine eklenmez. Yeni durumu döner; bileşen değiştirilemiyorsa `null`.
 */
export function toggleComponent(id: string): boolean | null {
  const component = get().doc.components.find((c) => c.id === id);
  const prop = component && TOGGLE_PROP[component.kind];
  if (!component || !prop) return null;
  const on = !component.props[prop];
  set((s) => ({
    doc: ops.updateComponent(s.doc, id, (c) => ({ ...c, props: { ...c.props, [prop]: on } })),
    dirty: true,
  }));
  return on;
}

/**
 * Simülasyonu yeniden başlatmadan değiştirilebilen bir özelliği günceller
 * (ör. potansiyometre konumu). Geri alma geçmişine eklenmez.
 */
export function setLiveProp(id: string, key: string, value: PropValue) {
  set((s) => ({
    doc: ops.updateComponent(s.doc, id, (c) => ({ ...c, props: { ...c.props, [key]: value } })),
    dirty: true,
  }));
}

/** Analiz ayarlarını değiştirir (devre yapısını etkilemez). */
export function updateSimulationSettings(update: (s: SimulationSettings) => SimulationSettings) {
  set((s) => ({ doc: { ...s.doc, simulation: update(s.doc.simulation) }, dirty: true }));
}

export function setTimeScale(timeScale: number) {
  set((s) => ({ doc: { ...s.doc, simulation: { ...s.doc.simulation, timeScale } }, dirty: true }));
}

export function selectAll() {
  const { doc } = get();
  set({
    selection: {
      components: doc.components.map((c) => c.id),
      wires: doc.wires.map((w) => w.id),
    },
  });
}

export function setSelection(selection: Selection) {
  set({ selection });
}

export function setTool(tool: Tool) {
  set({ tool, wireDraft: null });
}

export function setView(view: View) {
  set({ view });
}

export function setWireDraft(wireDraft: Point[] | null) {
  set({ wireDraft });
}

export function setCursor(cursor: Point) {
  const c = get().cursor;
  if (c[0] !== cursor[0] || c[1] !== cursor[1]) set({ cursor });
}

/** Esc: önce yarım kalan işlemi, sonra seçimi iptal eder. */
export function cancel() {
  const { wireDraft, tool, selection } = get();
  if (wireDraft) set({ wireDraft: null });
  else if (tool.type !== "select") set({ tool: { type: "select" } });
  else if (selection.components.length || selection.wires.length)
    set({ selection: EMPTY_SELECTION });
}

// --------------------------------------------------------------- Dosyalar

export function loadDocument(doc: CircuitDocument, filePath: string | null) {
  set((s) => ({
    doc: normalizeDocument(doc),
    filePath,
    dirty: false,
    version: s.version + 1,
    past: [],
    future: [],
    selection: EMPTY_SELECTION,
    tool: { type: "select" },
    wireDraft: null,
  }));
}

export function markSaved(filePath: string, name: string) {
  set((s) => ({ filePath, dirty: false, doc: { ...s.doc, meta: { ...s.doc.meta, name } } }));
}
