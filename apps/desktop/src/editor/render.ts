/** Şema alanının çizimi (Canvas 2D). */
import type { Frame, Topology } from "../lib/ipc";
import { GRID, type Tool, type View } from "../store/editor";
import {
  LIBRARY,
  LOGIC_THRESHOLD,
  PROP_META,
  addPoints,
  componentBounds,
  pinPositions,
  pointKey,
  rotate,
  strictlyInside,
} from "./library";
import type { Selection } from "./ops";
import { drawSymbol, leadPaths, withAlpha } from "./symbols";
import type { CircuitComponent, CircuitDocument, Point } from "./types";
import { formatValue, parseValue } from "./units";

export interface Theme {
  canvas: string;
  grid: string;
  wire: string;
  symbol: string;
  symbolHover: string;
  accent: string;
  current: string;
  vHigh: string;
  vLow: string;
  text: string;
  textDim: string;
  error: string;
  surface: string;
  border: string;
  logicHigh: string;
}

export function readTheme(): Theme {
  const css = getComputedStyle(document.documentElement);
  const v = (name: string) => css.getPropertyValue(name).trim();
  return {
    canvas: v("--bg-canvas"),
    grid: "#2a3346",
    wire: v("--wire"),
    symbol: "#b8c2d4",
    symbolHover: v("--text-primary"),
    accent: v("--accent"),
    current: v("--current"),
    vHigh: v("--voltage-high"),
    vLow: v("--voltage-low"),
    text: v("--text-primary"),
    textDim: v("--text-secondary"),
    error: v("--error"),
    surface: v("--bg-elevated"),
    border: v("--border"),
    logicHigh: v("--success"),
  };
}

/** Akım animasyonunun durumu (karelere göre ilerler). */
export interface Animation {
  topology: Topology | null;
  segmentPhase: number[];
  /** `elementPhase[e][k]`: bileşenin k. bacağındaki faz (iki uçlularda tek değer). */
  elementPhase: number[][];
}

export interface SimView {
  topology: Topology | null;
  frame: Frame | null;
  elementIndex: Map<string, number>;
  /** Basılı tutulan butonlar. */
  pressed: Set<string>;
}

export interface SceneInput {
  width: number;
  height: number;
  dpr: number;
  theme: Theme;
  doc: CircuitDocument;
  view: View;
  selection: Selection;
  tool: Tool;
  wireDraft: Point[] | null;
  cursor: Point;
  hover: { anchor: Point | null; componentId: string | null; wireId: string | null };
  box: [[number, number], [number, number]] | null;
  sim: SimView;
  animation: Animation;
  tooltip: { x: number; y: number; text: string } | null;
  /** Dışa aktarma: ızgara ve boştaki pin işaretleri çizilmez. */
  exportMode?: boolean;
}

// ---------------------------------------------------------------- Yardımcılar

function hexToRgb(hex: string): [number, number, number] {
  const n = parseInt(hex.slice(1, 7), 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}

function mix(a: string, b: string, t: number): string {
  const [r0, g0, b0] = hexToRgb(a);
  const [r1, g1, b1] = hexToRgb(b);
  const f = (x: number, y: number) => Math.round(x + (y - x) * t);
  return `rgb(${f(r0, r1)},${f(g0, g1)},${f(b0, b1)})`;
}

function voltageColor(v: number, vmax: number, theme: Theme): string {
  const t = Math.max(-1, Math.min(1, v / vmax));
  return t >= 0 ? mix(theme.wire, theme.vHigh, t) : mix(theme.wire, theme.vLow, -t);
}

/** Akım (A) → nokta hızı (ızgara birimi / s), logaritmik ölçek. */
export function dotSpeed(current: number): number {
  const abs = Math.abs(current);
  if (abs < 1e-7) return 0;
  return Math.sign(current) * Math.max(0.3, Math.log10(abs / 1e-7) * 0.9);
}

interface Connectivity {
  junctions: Point[];
  openPins: Point[];
}

const connectivityCache = new WeakMap<CircuitDocument, Connectivity>();

/** Birleşim noktaları (≥3 bağlantı) ve boşta kalan pinler. */
function connectivity(doc: CircuitDocument): Connectivity {
  const cached = connectivityCache.get(doc);
  if (cached) return cached;
  const count = new Map<string, number>();
  const points = new Map<string, Point>();
  const bump = (p: Point, n = 1) => {
    const k = pointKey(p);
    count.set(k, (count.get(k) ?? 0) + n);
    points.set(k, p);
  };
  const pins = doc.components.flatMap((c) => pinPositions(c));
  pins.forEach((p) => bump(p));
  for (const w of doc.wires) {
    bump(w.a);
    bump(w.b);
  }
  const anchors = [...points.values()];
  for (const w of doc.wires) {
    for (const p of anchors) if (strictlyInside(p, w.a, w.b)) bump(p, 2);
  }
  const result: Connectivity = {
    junctions: [...points.entries()].filter(([k]) => (count.get(k) ?? 0) >= 3).map(([, p]) => p),
    openPins: pins.filter((p) => count.get(pointKey(p)) === 1),
  };
  connectivityCache.set(doc, result);
  return result;
}

/** Bileşenin etiketinde gösterilen değer. */
function valueLabel(c: CircuitComponent): { text: string; ok: boolean } | null {
  const fmt = (key: string) => {
    const raw = c.props[key];
    const meta = PROP_META[key];
    const unit = meta?.type === "value" ? meta.unit : "";
    const v = typeof raw === "number" ? raw : parseValue(String(raw ?? ""));
    return v === null ? null : formatValue(v, unit);
  };
  const single = (key: string) => {
    const text = fmt(key);
    return text ? { text, ok: true } : { text: String(c.props[key]), ok: false };
  };
  switch (c.kind) {
    case "resistor":
      return single("resistance");
    case "capacitor":
      return single("capacitance");
    case "inductor":
      return single("inductance");
    case "dcVoltage":
      return single("voltage");
    case "dcCurrent":
      return single("current");
    case "acVoltage": {
      const a = fmt("amplitude");
      const f = fmt("frequency");
      return a && f ? { text: `${a} · ${f}`, ok: true } : { text: "?", ok: false };
    }
    case "npn":
    case "pnp": {
      const b = fmt("beta");
      return b ? { text: `β ${b}`, ok: true } : { text: "?", ok: false };
    }
    case "nmos":
    case "pmos": {
      const vth = fmt("vth");
      return vth ? { text: `Vth ${vth}`, ok: true } : { text: "?", ok: false };
    }
    case "clock":
      return single("frequency");
    case "potentiometer": {
      const r = fmt("resistance");
      return r ? { text: `${r} · ${c.props.position ?? 50}%`, ok: true } : { text: "?", ok: false };
    }
    case "zener":
      return single("breakdown");
    case "lamp": {
      const v = fmt("voltage");
      const p = fmt("power");
      return v && p ? { text: `${v} · ${p}`, ok: true } : { text: "?", ok: false };
    }
    case "pulseVoltage": {
      const lo = fmt("low");
      const hi = fmt("high");
      const f = fmt("frequency");
      return lo && hi && f ? { text: `${lo}…${hi} · ${f}`, ok: true } : { text: "?", ok: false };
    }
    default:
      return null;
  }
}

// ---------------------------------------------------------------- Çizim

export function drawScene(ctx: CanvasRenderingContext2D, s: SceneInput) {
  const { theme, view, dpr, doc, sim } = s;
  const scale = GRID * view.zoom;
  const toScreen = (x: number, y: number): [number, number] => [
    x * scale + view.x,
    y * scale + view.y,
  ];
  const worldTransform = () =>
    ctx.setTransform(dpr * scale, 0, 0, dpr * scale, dpr * view.x, dpr * view.y);
  const screenTransform = () => ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

  // Arka plan
  screenTransform();
  ctx.fillStyle = theme.canvas;
  ctx.fillRect(0, 0, s.width, s.height);

  // Izgara noktaları
  const step = s.exportMode ? 0 : scale >= 7 ? 1 : scale >= 2 ? 5 : 0;
  if (step) {
    ctx.fillStyle = theme.grid;
    const spacing = scale * step;
    const ox = ((view.x % spacing) + spacing) % spacing;
    const oy = ((view.y % spacing) + spacing) % spacing;
    const size = Math.max(1, Math.min(2, scale / 10));
    for (let x = ox; x < s.width; x += spacing)
      for (let y = oy; y < s.height; y += spacing)
        ctx.fillRect(x - size / 2, y - size / 2, size, size);
  }

  // Simülasyon verileri
  const frame = sim.frame;
  const topology = sim.topology;
  const live = frame !== null && topology !== null;
  let vmax = 1;
  const wireNode = new Map<string, number>();
  if (live) {
    vmax = Math.max(0.1, ...frame.nodeVoltages.map(Math.abs));
    for (const seg of topology.segments) wireNode.set(seg.wireId, seg.node);
  }
  const nodeColor = (node: number | undefined) =>
    live && node !== undefined ? voltageColor(frame.nodeVoltages[node] ?? 0, vmax, theme) : null;
  const pinNode = (c: CircuitComponent, pin: number): number | undefined => {
    if (c.kind === "ground") return 0;
    const i = sim.elementIndex.get(c.id);
    return i === undefined ? undefined : topology?.elements[i]?.nodes[pin];
  };

  const selComps = new Set(s.selection.components);
  const selWires = new Set(s.selection.wires);

  worldTransform();
  ctx.lineCap = "round";
  ctx.lineJoin = "round";

  // Teller
  for (const w of doc.wires) {
    const selected = selWires.has(w.id);
    const hovered = s.hover.wireId === w.id;
    ctx.strokeStyle = selected
      ? theme.accent
      : (nodeColor(wireNode.get(w.id)) ?? (hovered ? theme.symbol : theme.wire));
    ctx.lineWidth = selected || hovered ? 0.2 : 0.15;
    if (selected) {
      ctx.shadowColor = theme.accent;
      ctx.shadowBlur = 8 * dpr;
    }
    ctx.beginPath();
    ctx.moveTo(w.a[0], w.a[1]);
    ctx.lineTo(w.b[0], w.b[1]);
    ctx.stroke();
    ctx.shadowBlur = 0;
  }

  // Bileşenler
  for (const c of doc.components) {
    const selected = selComps.has(c.id);
    const hovered = s.hover.componentId === c.id;
    const stroke = selected ? theme.accent : hovered ? theme.symbolHover : theme.symbol;
    const leads = LIBRARY[c.kind].pins.map(
      (_, i) => nodeColor(pinNode(c, i)) ?? (selected ? theme.accent : theme.wire),
    );
    let glow = 0;
    let level: boolean | null = null;
    if (live && c.kind === "led") {
      const i = sim.elementIndex.get(c.id);
      const current = i === undefined ? 0 : (frame.pinCurrents[i]?.[0] ?? 0);
      glow = Math.max(0, Math.min(1, current / 0.015));
    }
    if (c.kind === "logicInput") level = Boolean(c.props.high);
    if (c.kind === "pushButton") level = sim.pressed.has(c.id);
    let segments: boolean[] | undefined;
    if (live && c.kind === "sevenSegment") {
      segments = LIBRARY[c.kind].pins.map((_, k) => {
        const node = pinNode(c, k);
        return node !== undefined && (frame.nodeVoltages[node] ?? 0) > LOGIC_THRESHOLD;
      });
    }
    if (live && c.kind === "lamp") {
      const i = sim.elementIndex.get(c.id);
      const current = Math.abs(i === undefined ? 0 : (frame.pinCurrents[i]?.[0] ?? 0));
      const rated =
        (parseValue(String(c.props.power)) ?? 1) / (parseValue(String(c.props.voltage)) ?? 12);
      glow = Math.max(0, Math.min(1, (current / rated) ** 2));
    }
    if (live && c.kind === "logicProbe") {
      const node = pinNode(c, 0);
      level = node === undefined ? null : (frame.nodeVoltages[node] ?? 0) > LOGIC_THRESHOLD;
    }
    if (live && c.kind === "clock") {
      const node = pinNode(c, 0);
      level = node === undefined ? null : (frame.nodeVoltages[node] ?? 0) > LOGIC_THRESHOLD;
    }
    ctx.save();
    ctx.translate(c.pos[0], c.pos[1]);
    ctx.rotate((c.rot * Math.PI) / 2);
    if (selected) {
      ctx.shadowColor = theme.accent;
      ctx.shadowBlur = 10 * dpr;
    }
    drawSymbol(ctx, c.kind, c.props, {
      stroke,
      leads,
      glow,
      level,
      high: theme.logicHigh,
      segments,
    });
    ctx.restore();
  }

  // Birleşim noktaları ve boştaki pinler
  const conn = connectivity(doc);
  for (const p of conn.junctions) {
    ctx.fillStyle = theme.symbol;
    ctx.beginPath();
    ctx.arc(p[0], p[1], 0.22, 0, Math.PI * 2);
    ctx.fill();
  }
  ctx.lineWidth = 0.08;
  ctx.strokeStyle = withAlpha(theme.error, 0.8);
  for (const p of s.exportMode ? [] : conn.openPins) {
    ctx.beginPath();
    ctx.arc(p[0], p[1], 0.2, 0, Math.PI * 2);
    ctx.stroke();
  }

  // Akım noktaları
  if (live) drawCurrentDots(ctx, s, theme);

  // Üzerine gelinen bağlantı noktası
  if (s.hover.anchor) {
    ctx.strokeStyle = theme.accent;
    ctx.lineWidth = 0.1;
    ctx.beginPath();
    ctx.arc(s.hover.anchor[0], s.hover.anchor[1], 0.35, 0, Math.PI * 2);
    ctx.stroke();
  }

  // Çizilmekte olan tel
  if (s.wireDraft && s.wireDraft.length) {
    const pts = [...s.wireDraft, ...routeTo(s.wireDraft.at(-1)!, s.cursor)];
    ctx.strokeStyle = theme.accent;
    ctx.lineWidth = 0.15;
    ctx.setLineDash([0.4, 0.3]);
    ctx.beginPath();
    pts.forEach((p, i) => (i === 0 ? ctx.moveTo(p[0], p[1]) : ctx.lineTo(p[0], p[1])));
    ctx.stroke();
    ctx.setLineDash([]);
  }

  // Yerleştirme önizlemesi
  if (s.tool.type === "place") {
    ctx.save();
    ctx.globalAlpha = 0.55;
    ctx.translate(s.cursor[0], s.cursor[1]);
    ctx.rotate((s.tool.rot * Math.PI) / 2);
    drawSymbol(ctx, s.tool.kind, LIBRARY[s.tool.kind].props, {
      stroke: theme.accent,
      leads: LIBRARY[s.tool.kind].pins.map(() => theme.accent),
    });
    ctx.restore();
  }

  // ------------------------------------------------ Ekran koordinatında metin
  screenTransform();
  const fontSize = Math.max(10, Math.min(13, 11 * view.zoom));
  if (scale >= 8) {
    for (const c of doc.components) {
      if (c.kind === "ground") continue;
      drawLabel(ctx, c, s, toScreen, fontSize);
    }
  }

  // Seçim kutusu
  if (s.box) {
    const [a, b] = s.box;
    const [ax, ay] = toScreen(a[0], a[1]);
    const [bx, by] = toScreen(b[0], b[1]);
    ctx.fillStyle = withAlpha(theme.accent, 0.08);
    ctx.strokeStyle = withAlpha(theme.accent, 0.7);
    ctx.lineWidth = 1;
    ctx.fillRect(Math.min(ax, bx), Math.min(ay, by), Math.abs(bx - ax), Math.abs(by - ay));
    ctx.strokeRect(
      Math.min(ax, bx) + 0.5,
      Math.min(ay, by) + 0.5,
      Math.abs(bx - ax),
      Math.abs(by - ay),
    );
  }

  // İpucu (gerilim)
  if (s.tooltip) drawTooltip(ctx, s.tooltip, theme);
}

function drawLabel(
  ctx: CanvasRenderingContext2D,
  c: CircuitComponent,
  s: SceneInput,
  toScreen: (x: number, y: number) => [number, number],
  fontSize: number,
) {
  const { theme, sim } = s;
  const pins = pinPositions(c);
  const isMeter = c.kind === "voltmeter" || c.kind === "ammeter";
  if (pins.length === 1) {
    // Tek pinli bileşenler (lojik giriş/prob, saat): etiket pinin karşı tarafında
    const [x0, y0, x1, y1] = componentBounds(c);
    const [dx, dy] = rotate(LIBRARY[c.kind].pins[0]!, c.rot);
    const value = valueLabel(c);
    const text = value ? `${c.id}  ${value.text}` : c.id;
    ctx.font = `500 ${fontSize}px Inter Variable, Inter, sans-serif`;
    ctx.fillStyle = theme.textDim;
    if (dx !== 0) {
      const [ax, ay] = toScreen(dx > 0 ? x0 - 0.35 : x1 + 0.35, c.pos[1]);
      ctx.textAlign = dx > 0 ? "right" : "left";
      ctx.textBaseline = "middle";
      ctx.fillText(text, ax, ay);
    } else {
      const [ax, ay] = toScreen(c.pos[0], dy > 0 ? y0 - 0.3 : y1 + 0.3);
      ctx.textAlign = "center";
      ctx.textBaseline = dy > 0 ? "bottom" : "top";
      ctx.fillText(text, ax, ay);
    }
    return;
  }
  if (pins.length !== 2) {
    // Çok uçlu bileşenler: etiket gövdenin üstünde
    const [x0, y0, x1] = componentBounds(c);
    const value = valueLabel(c);
    const [ax, ay] = toScreen((x0 + x1) / 2, y0 - 0.35);
    ctx.font = `500 ${fontSize}px Inter Variable, Inter, sans-serif`;
    ctx.textBaseline = "bottom";
    ctx.textAlign = "left";
    const idText = value ? `${c.id}  ` : c.id;
    const idWidth = ctx.measureText(idText).width;
    const valueWidth = value ? ctx.measureText(value.text).width : 0;
    const start = ax - (idWidth + valueWidth) / 2;
    ctx.fillStyle = theme.textDim;
    ctx.fillText(idText, start, ay);
    if (value) {
      ctx.fillStyle = value.ok ? theme.text : theme.error;
      ctx.fillText(value.text, start + idWidth, ay);
    }
    return;
  }
  const [p0, p1] = [pins[0]!, pins[1]!];
  const horizontal = Math.abs(p1[0] - p0[0]) >= Math.abs(p1[1] - p0[1]);

  let value = valueLabel(c);
  let valueColor = theme.text;
  if (isMeter) {
    const i = sim.elementIndex.get(c.id);
    const el = i === undefined ? undefined : sim.topology?.elements[i];
    if (sim.frame && el) {
      const reading =
        c.kind === "voltmeter"
          ? (sim.frame.nodeVoltages[el.nodes[0]!] ?? 0) -
            (sim.frame.nodeVoltages[el.nodes[1]!] ?? 0)
          : (sim.frame.pinCurrents[i!]?.[0] ?? 0);
      value = { text: formatValue(reading, c.kind === "voltmeter" ? "V" : "A"), ok: true };
      valueColor = theme.accent;
    }
  }

  // Ölçü aletinin harfi her zaman dik durur.
  if (isMeter) {
    const [cx, cy] = toScreen(c.pos[0], c.pos[1]);
    ctx.font = `600 ${fontSize * 1.25 * Math.max(1, s.view.zoom * 0.8)}px Inter Variable, Inter, sans-serif`;
    ctx.fillStyle = theme.symbol;
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.fillText(c.kind === "voltmeter" ? "V" : "A", cx, cy + 1);
  }

  const gap = isMeter ? 1.9 : 1.5;
  const [ax, ay] = horizontal
    ? toScreen(c.pos[0], c.pos[1] - gap)
    : toScreen(c.pos[0] + gap, c.pos[1]);

  ctx.font = `500 ${fontSize}px Inter Variable, Inter, sans-serif`;
  ctx.textBaseline = horizontal ? "bottom" : "middle";
  ctx.textAlign = horizontal ? "center" : "left";

  if (horizontal) {
    const idText = value ? `${c.id}  ` : c.id;
    const idWidth = ctx.measureText(idText).width;
    const valueWidth = value ? ctx.measureText(value.text).width : 0;
    const start = ax - (idWidth + valueWidth) / 2;
    ctx.textAlign = "left";
    ctx.fillStyle = theme.textDim;
    ctx.fillText(idText, start, ay);
    if (value) {
      ctx.fillStyle = value.ok ? valueColor : theme.error;
      ctx.fillText(value.text, start + idWidth, ay);
    }
  } else {
    const lineGap = fontSize * 0.65;
    ctx.fillStyle = theme.textDim;
    ctx.fillText(c.id, ax, value ? ay - lineGap : ay);
    if (value) {
      ctx.fillStyle = value.ok ? valueColor : theme.error;
      ctx.fillText(value.text, ax, ay + lineGap);
    }
  }
}

function drawCurrentDots(ctx: CanvasRenderingContext2D, s: SceneInput, theme: Theme) {
  const { animation, sim } = s;
  const topology = sim.topology;
  if (!topology || animation.topology !== topology) return;
  ctx.fillStyle = theme.current;
  const dotsAlong = (a: Point, b: Point, phase: number) => {
    const len = Math.hypot(b[0] - a[0], b[1] - a[1]);
    if (len === 0) return;
    const ux = (b[0] - a[0]) / len;
    const uy = (b[1] - a[1]) / len;
    for (let d = phase; d < len; d += 1) {
      ctx.beginPath();
      ctx.arc(a[0] + ux * d, a[1] + uy * d, 0.13, 0, Math.PI * 2);
      ctx.fill();
    }
  };
  const frame = sim.frame!;
  topology.segments.forEach((seg, i) => {
    if (dotSpeed(frame.segmentCurrents[i] ?? 0) !== 0)
      dotsAlong(seg.a, seg.b, animation.segmentPhase[i] ?? 0);
  });
  // Bileşenlerden geçen akım: iki uçlularda pin 0 → pin 1, diğerlerinde
  // her bacak boyunca pinden gövdeye.
  for (const c of s.doc.components) {
    const i = sim.elementIndex.get(c.id);
    if (i === undefined || c.kind === "voltmeter") continue;
    const currents = frame.pinCurrents[i] ?? [];
    const phases = animation.elementPhase[i] ?? [];
    const pins = pinPositions(c);
    if (pins.length === 2) {
      if (dotSpeed(currents[0] ?? 0) !== 0) dotsAlong(pins[0]!, pins[1]!, phases[0] ?? 0);
      continue;
    }
    leadPaths(c.kind).forEach((path, k) => {
      if (dotSpeed(currents[k] ?? 0) === 0) return;
      const world = path.map((p) => addPoints(c.pos, rotate(p, c.rot)));
      let offset = phases[k] ?? 0;
      for (let j = 1; j < world.length; j++) {
        const [a, b] = [world[j - 1]!, world[j]!];
        dotsAlong(a, b, offset);
        const len = Math.hypot(b[0] - a[0], b[1] - a[1]);
        offset = (((offset - len) % 1) + 1) % 1;
      }
    });
  }
}

/** Animasyon fazlarını `dt` saniye ilerletir. */
export function advanceAnimation(animation: Animation, sim: SimView, dt: number) {
  const { topology, frame } = sim;
  if (!topology || !frame) return;
  if (animation.topology !== topology) {
    animation.topology = topology;
    animation.segmentPhase = topology.segments.map(() => 0);
    animation.elementPhase = topology.elements.map((e) => e.nodes.map(() => 0));
  }
  const advance = (phases: number[], currents: number[]) => {
    for (let i = 0; i < phases.length; i++) {
      const p = (phases[i] ?? 0) + dotSpeed(currents[i] ?? 0) * dt;
      phases[i] = ((p % 1) + 1) % 1;
    }
  };
  advance(animation.segmentPhase, frame.segmentCurrents);
  animation.elementPhase.forEach((phases, i) => {
    const currents = frame.pinCurrents[i] ?? [];
    // İki uçlularda tek akım (pin 0) kullanılır.
    advance(phases, phases.length === 2 ? [currents[0] ?? 0, currents[0] ?? 0] : currents);
  });
}

function drawTooltip(
  ctx: CanvasRenderingContext2D,
  tip: { x: number; y: number; text: string },
  theme: Theme,
) {
  ctx.font = "500 12px JetBrains Mono Variable, JetBrains Mono, monospace";
  const w = ctx.measureText(tip.text).width + 14;
  const x = tip.x + 14;
  const y = tip.y + 16;
  ctx.fillStyle = theme.surface;
  ctx.strokeStyle = theme.border;
  ctx.lineWidth = 1;
  ctx.beginPath();
  ctx.roundRect(x, y, w, 22, 5);
  ctx.fill();
  ctx.stroke();
  ctx.fillStyle = theme.text;
  ctx.textAlign = "left";
  ctx.textBaseline = "middle";
  ctx.fillText(tip.text, x + 7, y + 11.5);
}

/** Son noktadan hedefe önce yatay sonra dikey giden tel yolu. */
export function routeTo(from: Point, to: Point): Point[] {
  if (from[0] === to[0] || from[1] === to[1]) return [to];
  return [[to[0], from[1]], to];
}
