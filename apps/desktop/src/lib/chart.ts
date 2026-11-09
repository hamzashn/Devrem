import type uPlot from "uplot";
import { formatValue } from "../editor/units";

/** Kanal renkleri (koyu temada ayırt edilebilir). */
export const CHANNEL_COLORS = ["#22d3ee", "#facc15", "#f472b6", "#4ade80", "#a78bfa", "#fb923c"];

const AXIS_STROKE = "#8b95a7";
const GRID_STROKE = "#1f2735";
const FONT = "11px Inter Variable, Inter, sans-serif";

/** Koyu temaya uygun eksen; değerler SI önekiyle yazılır. */
export function axis(unit: string, extra: Partial<uPlot.Axis> = {}): uPlot.Axis {
  return {
    stroke: AXIS_STROKE,
    font: FONT,
    labelFont: FONT,
    grid: { stroke: GRID_STROKE, width: 1 },
    ticks: { stroke: GRID_STROKE, width: 1 },
    size: 56,
    values: (_u, splits) => splits.map((v) => (v == null ? "" : formatValue(v, unit, 3))),
    ...extra,
  };
}
