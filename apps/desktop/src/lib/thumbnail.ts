import { componentBounds } from "../editor/library";
import { drawScene, readTheme, type Theme } from "../editor/render";
import type { CircuitDocument } from "../editor/types";
import { GRID } from "../store/editor";

let theme: Theme | null = null;

/** Devreyi tuvale sığacak şekilde küçük bir önizleme olarak çizer. */
export function drawThumbnail(canvas: HTMLCanvasElement, doc: CircuitDocument) {
  const dpr = window.devicePixelRatio || 1;
  const width = canvas.clientWidth;
  const height = canvas.clientHeight;
  canvas.width = Math.round(width * dpr);
  canvas.height = Math.round(height * dpr);
  theme ??= readTheme();

  const xs: number[] = [];
  const ys: number[] = [];
  for (const c of doc.components) {
    const [x0, y0, x1, y1] = componentBounds(c);
    xs.push(x0, x1);
    ys.push(y0, y1);
  }
  for (const w of doc.wires) {
    xs.push(w.a[0], w.b[0]);
    ys.push(w.a[1], w.b[1]);
  }
  const [x0, x1, y0, y1] = xs.length
    ? [Math.min(...xs) - 1.5, Math.max(...xs) + 1.5, Math.min(...ys) - 3, Math.max(...ys) + 1.5]
    : [-5, 5, -5, 5];
  const scale = Math.min(width / (x1 - x0), height / (y1 - y0));
  const zoom = scale / GRID;

  drawScene(canvas.getContext("2d")!, {
    width,
    height,
    dpr,
    theme,
    doc,
    view: {
      zoom,
      x: width / 2 - ((x0 + x1) / 2) * scale,
      y: height / 2 - ((y0 + y1) / 2) * scale,
    },
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
  });
}
