/** Şemayı ve sonuçları dışa aktarma: SPICE, PNG, SVG, CSV. */
import { save } from "@tauri-apps/plugin-dialog";
import { Context as SvgContext } from "svgcanvas";
import i18n from "../i18n";
import { componentBounds } from "../editor/library";
import { drawScene, readTheme, type SceneInput } from "../editor/render";
import type { CircuitDocument } from "../editor/types";
import { GRID, useEditor } from "../store/editor";
import { showNotice } from "../store/notice";
import { describeError } from "./errors";
import { exportSpice, isTauri, writeBinaryFile, writeTextFile } from "./ipc";

/** Dışa aktarılan görüntüde bir ızgara biriminin piksel karşılığı. */
const EXPORT_SCALE = 24;
const MARGIN = 2;

async function pickPath(extension: string, label: string): Promise<string | null> {
  const { doc } = useEditor.getState();
  const path = await save({
    filters: [{ name: label, extensions: [extension] }],
    defaultPath: `${doc.meta.name}.${extension}`,
  });
  if (!path) return null;
  return path.toLowerCase().endsWith(`.${extension}`) ? path : `${path}.${extension}`;
}

async function run(task: () => Promise<string | null>) {
  if (!isTauri()) return;
  try {
    const path = await task();
    if (path) showNotice("info", i18n.t("notices.exported", { path }));
  } catch (e) {
    showNotice("error", describeError(e));
  }
}

/** Devrenin kapladığı alan (ızgara birimi). */
function documentBounds(doc: CircuitDocument): [number, number, number, number] | null {
  const xs: number[] = [];
  const ys: number[] = [];
  for (const c of doc.components) {
    const [x0, y0, x1, y1] = componentBounds(c);
    xs.push(x0, x1);
    ys.push(y0 - 1.5, y1); // üstteki etiketler için pay
  }
  for (const w of doc.wires) {
    xs.push(w.a[0], w.b[0]);
    ys.push(w.a[1], w.b[1]);
  }
  if (!xs.length) return null;
  return [
    Math.min(...xs) - MARGIN,
    Math.min(...ys) - MARGIN,
    Math.max(...xs) + MARGIN,
    Math.max(...ys) + MARGIN,
  ];
}

/** Devreyi verilen bağlama (tuval veya SVG) çizer; boyutları döner. */
function renderDocument(
  ctx: CanvasRenderingContext2D,
  doc: CircuitDocument,
  dpr: number,
): { width: number; height: number } {
  const bounds = documentBounds(doc) ?? [0, 0, 10, 10];
  const [x0, y0, x1, y1] = bounds;
  const width = Math.ceil((x1 - x0) * EXPORT_SCALE);
  const height = Math.ceil((y1 - y0) * EXPORT_SCALE);
  const scene: SceneInput = {
    width,
    height,
    dpr,
    theme: readTheme(),
    doc,
    view: { x: -x0 * EXPORT_SCALE, y: -y0 * EXPORT_SCALE, zoom: EXPORT_SCALE / GRID },
    selection: { components: [], wires: [] },
    tool: { type: "select" },
    wireDraft: null,
    cursor: [0, 0],
    hover: { anchor: null, componentId: null, wireId: null },
    box: null,
    sim: { topology: null, frame: null, elementIndex: new Map(), pressed: new Set() },
    animation: { topology: null, segmentPhase: [], elementPhase: [] },
    tooltip: null,
    exportMode: true,
  };
  drawScene(ctx, scene);
  return { width, height };
}

/** Görüntü boyutunu ölçmek için devreyi boş bir bağlama çizmeden hesaplar. */
function imageSize(doc: CircuitDocument) {
  const [x0, y0, x1, y1] = documentBounds(doc) ?? [0, 0, 10, 10];
  return {
    width: Math.ceil((x1 - x0) * EXPORT_SCALE),
    height: Math.ceil((y1 - y0) * EXPORT_SCALE),
  };
}

/** Belgeyi verilen yola SPICE netlist olarak yazar. */
export async function writeSpice(path: string, doc = useEditor.getState().doc) {
  await writeTextFile(path, await exportSpice(doc));
}

/** Belgeyi verilen yola PNG (2× çözünürlük) olarak yazar. */
export async function writePng(path: string, doc = useEditor.getState().doc) {
  const dpr = 2;
  const { width, height } = imageSize(doc);
  const canvas = document.createElement("canvas");
  canvas.width = width * dpr;
  canvas.height = height * dpr;
  renderDocument(canvas.getContext("2d")!, doc, dpr);
  const base64 = canvas.toDataURL("image/png").split(",")[1] ?? "";
  await writeBinaryFile(path, base64);
}

/** Belgeyi verilen yola SVG olarak yazar. */
export async function writeSvg(path: string, doc = useEditor.getState().doc) {
  const { width, height } = imageSize(doc);
  const svg = new SvgContext(width, height);
  renderDocument(svg as unknown as CanvasRenderingContext2D, doc, 1);
  await writeTextFile(path, svg.getSerializedSvg(true));
}

function exportWith(extension: string, label: string, write: (path: string) => Promise<void>) {
  return run(async () => {
    const path = await pickPath(extension, label);
    if (!path) return null;
    await write(path);
    return path;
  });
}

export const exportSpiceFile = () => exportWith("cir", "SPICE netlist", (p) => writeSpice(p));
export const exportPng = () => exportWith("png", "PNG", (p) => writePng(p));
export const exportSvg = () => exportWith("svg", "SVG", (p) => writeSvg(p));

/** Sütunlardan CSV metni üretir (ondalık ayırıcı nokta, ayraç virgül). */
export function toCsv(headers: string[], columns: number[][]): string {
  const escape = (s: string) => (/[",\n]/.test(s) ? `"${s.replace(/"/g, '""')}"` : s);
  const rows = [headers.map(escape).join(",")];
  const n = Math.max(0, ...columns.map((c) => c.length));
  for (let i = 0; i < n; i++) rows.push(columns.map((c) => String(c[i] ?? "")).join(","));
  return `${rows.join("\n")}\n`;
}

export function exportCsv(name: string, headers: string[], columns: number[][]) {
  return run(async () => {
    const { doc } = useEditor.getState();
    const path = await save({
      filters: [{ name: "CSV", extensions: ["csv"] }],
      defaultPath: `${doc.meta.name} - ${name}.csv`,
    });
    if (!path) return null;
    const file = path.toLowerCase().endsWith(".csv") ? path : `${path}.csv`;
    await writeTextFile(file, toCsv(headers, columns));
    return file;
  });
}
