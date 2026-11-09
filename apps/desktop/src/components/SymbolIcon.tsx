import { useEffect, useRef } from "react";
import { LIBRARY } from "../editor/library";
import { drawSymbol } from "../editor/symbols";
import type { ComponentKind } from "../editor/types";

const WIDTH = 40;
const HEIGHT = 28;

/** Bileşen sembolünün küçük önizlemesi. */
export function SymbolIcon({ kind, color }: { kind: ComponentKind; color: string }) {
  const ref = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = ref.current!;
    const dpr = window.devicePixelRatio || 1;
    canvas.width = WIDTH * dpr;
    canvas.height = HEIGHT * dpr;
    const ctx = canvas.getContext("2d")!;
    const vertical = LIBRARY[kind].pins.length === 2 && LIBRARY[kind].pins[0]![0] === 0;
    const scale = kind === "ground" ? 9 : 8;
    ctx.setTransform(dpr * scale, 0, 0, dpr * scale, (dpr * WIDTH) / 2, (dpr * HEIGHT) / 2);
    if (kind === "ground") ctx.translate(0, -0.6);
    // Dikey kaynakları yatay göster
    if (vertical) ctx.rotate(-Math.PI / 2);
    drawSymbol(ctx, kind, LIBRARY[kind].props, { stroke: color, leads: [color, color], glow: 0 });
  }, [kind, color]);

  return <canvas ref={ref} style={{ width: WIDTH, height: HEIGHT }} aria-hidden />;
}
