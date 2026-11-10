import { useEffect, useRef } from "react";
import { LIBRARY, componentBounds } from "../editor/library";
import { drawSymbol } from "../editor/symbols";
import type { ComponentKind } from "../editor/types";

const WIDTH = 40;
const HEIGHT = 28;
/** Önizlemede bir ızgara biriminin en büyük piksel karşılığı. */
const MAX_SCALE = 8;

/** Bileşen sembolünün küçük önizlemesi; sembol kutuya sığacak şekilde ölçeklenir. */
export function SymbolIcon({ kind, color }: { kind: ComponentKind; color: string }) {
  const ref = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = ref.current!;
    const dpr = window.devicePixelRatio || 1;
    canvas.width = WIDTH * dpr;
    canvas.height = HEIGHT * dpr;
    const ctx = canvas.getContext("2d")!;
    const vertical = LIBRARY[kind].pins.length === 2 && LIBRARY[kind].pins[0]![0] === 0;
    // Dikey kaynaklar yatay gösterilir (90° döndürülmüş sınırlar).
    const [x0, y0, x1, y1] = componentBounds({
      id: "",
      kind,
      pos: [0, 0],
      rot: vertical ? 3 : 0,
      props: {},
    });
    const scale = Math.min(MAX_SCALE, (WIDTH - 2) / (x1 - x0), (HEIGHT - 2) / (y1 - y0));
    ctx.setTransform(dpr * scale, 0, 0, dpr * scale, (dpr * WIDTH) / 2, (dpr * HEIGHT) / 2);
    ctx.translate(-(x0 + x1) / 2, -(y0 + y1) / 2);
    if (vertical) ctx.rotate(-Math.PI / 2);
    const leads = LIBRARY[kind].pins.map(() => color);
    drawSymbol(ctx, kind, LIBRARY[kind].props, { stroke: color, leads, glow: 0 });
  }, [kind, color]);

  return <canvas ref={ref} style={{ width: WIDTH, height: HEIGHT }} aria-hidden />;
}
