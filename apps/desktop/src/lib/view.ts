import { componentBounds } from "../editor/library";
import { GRID, setView, useEditor } from "../store/editor";

function canvasSize(): { width: number; height: number } {
  const canvas = document.querySelector("main canvas");
  const rect = canvas?.getBoundingClientRect();
  return { width: rect?.width ?? 800, height: rect?.height ?? 600 };
}

/** Görünümü alanın ortasına göre yakınlaştırır. */
export function zoomBy(factor: number) {
  const { view } = useEditor.getState();
  const { width, height } = canvasSize();
  const zoom = Math.max(0.25, Math.min(5, view.zoom * factor));
  const k = zoom / view.zoom;
  const cx = width / 2;
  const cy = height / 2;
  setView({ zoom, x: cx - (cx - view.x) * k, y: cy - (cy - view.y) * k });
}

/** Tüm devreyi ekrana sığdırır. */
export function fitToContent() {
  const { doc } = useEditor.getState();
  const { width, height } = canvasSize();
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
  if (!xs.length) {
    setView({ x: width / 2, y: height / 2, zoom: 1.25 });
    return;
  }
  const [minX, maxX, minY, maxY] = [
    Math.min(...xs),
    Math.max(...xs),
    Math.min(...ys),
    Math.max(...ys),
  ];
  // Yatayda daha geniş pay: tek pinli bileşenlerin etiketleri yanda durur.
  const [marginX, marginY] = [7, 4];
  const zoom = Math.max(
    0.25,
    Math.min(
      2.5,
      width / ((maxX - minX + marginX * 2) * GRID),
      height / ((maxY - minY + marginY * 2) * GRID),
    ),
  );
  const scale = GRID * zoom;
  setView({
    zoom,
    x: width / 2 - ((minX + maxX) / 2) * scale,
    y: height / 2 - ((minY + maxY) / 2) * scale,
  });
}
