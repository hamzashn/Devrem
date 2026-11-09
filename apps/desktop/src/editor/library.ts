import raw from "@devrem/library/components.json";
import type { CircuitComponent, ComponentKind, Point, PropValue } from "./types";

export type Category =
  | "passive"
  | "sources"
  | "semiconductors"
  | "ic"
  | "switches"
  | "logic"
  | "sequential"
  | "io"
  | "measure";

interface KindDef {
  prefix: string;
  category: Category;
  pins: Point[];
  props: Record<string, PropValue>;
}

/** Bileşen tanımları; Rust tarafıyla ortak `packages/library/components.json`. */
export const LIBRARY = raw as unknown as Record<ComponentKind, KindDef>;
export const KINDS = Object.keys(LIBRARY) as ComponentKind[];

/** Kenar çubuğundaki kategori sırası. */
export const CATEGORIES = [
  "passive",
  "sources",
  "semiconductors",
  "ic",
  "switches",
  "logic",
  "sequential",
  "io",
  "measure",
] as const;

export function kindsInCategory(category: string): ComponentKind[] {
  return KINDS.filter((k) => LIBRARY[k].category === category);
}

export type PropMeta =
  | { type: "slider"; min: number; max: number; unit: string }
  | { type: "value"; unit: string }
  | { type: "bool" }
  | { type: "select"; options: string[] };

export const PROP_META: Record<string, PropMeta> = {
  resistance: { type: "value", unit: "Ω" },
  capacitance: { type: "value", unit: "F" },
  inductance: { type: "value", unit: "H" },
  voltage: { type: "value", unit: "V" },
  current: { type: "value", unit: "A" },
  amplitude: { type: "value", unit: "V" },
  frequency: { type: "value", unit: "Hz" },
  offset: { type: "value", unit: "V" },
  phase: { type: "value", unit: "°" },
  closed: { type: "bool" },
  color: { type: "select", options: ["red", "green", "yellow", "blue", "white"] },
  beta: { type: "value", unit: "" },
  vth: { type: "value", unit: "V" },
  k: { type: "value", unit: "A/V²" },
  gain: { type: "value", unit: "" },
  vsat: { type: "value", unit: "V" },
  high: { type: "bool" },
  position: { type: "slider", min: 0, max: 100, unit: "%" },
  breakdown: { type: "value", unit: "V" },
  toB: { type: "bool" },
  power: { type: "value", unit: "W" },
  low: { type: "value", unit: "V" },
  duty: { type: "value", unit: "%" },
};

/** Dijital (lojik) bileşenler. */
export const LOGIC_KINDS = new Set<ComponentKind>([
  "and",
  "or",
  "not",
  "nand",
  "nor",
  "xor",
  "xnor",
  "dFlipFlop",
  "jkFlipFlop",
  "tFlipFlop",
  "logicInput",
  "logicProbe",
  "clock",
  "sevenSegment",
  "hexDecoder",
]);

/** Simülasyon sırasında tıklanınca açılıp kapanan bileşenler ve özellikleri. */
export const TOGGLE_PROP: Partial<Record<ComponentKind, string>> = {
  switch: "closed",
  logicInput: "high",
  spdtSwitch: "toB",
};

/** Lojik giriş eşiği (V); Rust tarafıyla aynı. */
export const LOGIC_THRESHOLD = 2.5;

export const LED_COLORS: Record<string, string> = {
  red: "#ef4444",
  green: "#22c55e",
  yellow: "#facc15",
  blue: "#3b82f6",
  white: "#f1f5f9",
};

// ------------------------------------------------------------ Geometri

export function rotate([x, y]: Point, rot: number): Point {
  switch (((rot % 4) + 4) % 4) {
    case 0:
      return [x, y];
    case 1:
      return [-y, x];
    case 2:
      return [-x, -y];
    default:
      return [y, -x];
  }
}

export const addPoints = (a: Point, b: Point): Point => [a[0] + b[0], a[1] + b[1]];
export const samePoint = (a: Point, b: Point) => a[0] === b[0] && a[1] === b[1];
export const pointKey = (p: Point) => `${p[0]},${p[1]}`;

export function pinPositions(c: Pick<CircuitComponent, "kind" | "pos" | "rot">): Point[] {
  return LIBRARY[c.kind].pins.map((p) => addPoints(c.pos, rotate(p, c.rot)));
}

/** `p`, `a`–`b` doğru parçasının üzerinde ve uçlardan farklı mı? (Rust ile aynı) */
export function strictlyInside(p: Point, a: Point, b: Point): boolean {
  if (samePoint(p, a) || samePoint(p, b)) return false;
  const cross = (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]);
  return (
    cross === 0 &&
    p[0] >= Math.min(a[0], b[0]) &&
    p[0] <= Math.max(a[0], b[0]) &&
    p[1] >= Math.min(a[1], b[1]) &&
    p[1] <= Math.max(a[1], b[1])
  );
}

export function distanceToSegment(x: number, y: number, a: Point, b: Point): number {
  const dx = b[0] - a[0];
  const dy = b[1] - a[1];
  const len2 = dx * dx + dy * dy;
  const t = len2 === 0 ? 0 : Math.max(0, Math.min(1, ((x - a[0]) * dx + (y - a[1]) * dy) / len2));
  return Math.hypot(x - (a[0] + t * dx), y - (a[1] + t * dy));
}

/** Döndürülmemiş haldeki tıklama alanı `[x0, y0, x1, y1]`. */
function localBounds(kind: ComponentKind): [number, number, number, number] {
  switch (kind) {
    case "ground":
      return [-1, -0.3, 1, 1.5];
    case "dcVoltage":
    case "dcCurrent":
    case "acVoltage":
    case "pulseVoltage":
      return [-1.2, -2, 1.2, 2];
    case "potentiometer":
      return [-2, -2, 2, 0.6];
    case "pushButton":
      return [-2, -1.3, 2, 0.4];
    case "spdtSwitch":
      return [-2, -1.3, 2, 1.3];
    case "lamp":
      return [-2, -1, 2, 1];
    case "timer555":
    case "hexDecoder":
      return [-3, -4, 3, 4];
    case "sevenSegment":
      return [-3, -4, 2, 4];
    case "voltmeter":
    case "ammeter":
      return [-2, -1.3, 2, 1.3];
    case "switch":
      return [-2, -1.1, 2, 0.6];
    case "led":
      return [-2, -1.3, 2, 0.8];
    case "npn":
    case "pnp":
    case "nmos":
    case "pmos":
      return [-2, -2, 1.3, 2];
    case "opAmp":
    case "and":
    case "or":
    case "nand":
    case "nor":
    case "xor":
    case "xnor":
      return [-3, -1.6, 3, 1.6];
    case "not":
      return [-3, -1, 3, 1];
    case "dFlipFlop":
    case "tFlipFlop":
    case "jkFlipFlop":
      return [-3, -2, 3, 2];
    case "logicInput":
    case "clock":
      return [-1.4, -1, 2, 1];
    case "logicProbe":
      return [-2, -1, 1.2, 1];
    default:
      return [-2, -0.9, 2, 0.9];
  }
}

export function componentBounds(c: CircuitComponent): [number, number, number, number] {
  const [x0, y0, x1, y1] = localBounds(c.kind);
  const corners = (
    [
      [x0, y0],
      [x1, y0],
      [x1, y1],
      [x0, y1],
    ] as Point[]
  ).map((p) => rotate(p, c.rot));
  const xs = corners.map((p) => p[0] + c.pos[0]);
  const ys = corners.map((p) => p[1] + c.pos[1]);
  return [Math.min(...xs), Math.min(...ys), Math.max(...xs), Math.max(...ys)];
}
