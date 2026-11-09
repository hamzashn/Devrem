/**
 * Bileşen sembolleri. Çizimler döndürülmemiş, ızgara biriminde yerel
 * koordinatlardadır; çağıran taraf konum ve dönüşü uygular.
 */
import { LED_COLORS } from "./library";
import type { ComponentKind, Point, PropValue } from "./types";

type Ctx = CanvasRenderingContext2D;

export interface SymbolStyle {
  /** Gövde rengi. */
  stroke: string;
  /** Her pinin bağlantı ucu rengi (simülasyonda gerilime göre). */
  leads: string[];
  /** LED parlaklığı (0–1). */
  glow?: number;
  /** Lojik seviye (prob ve lojik giriş için); bilinmiyorsa `null`. */
  level?: boolean | null;
  /** Lojik "1" rengi. */
  high?: string;
  /** 7 segment göstergede yanan segmentler (a … g). */
  segments?: boolean[];
}

interface SymbolDef {
  /** Her pin için pinden gövdeye uzanan bağlantı ucu (çoklu çizgi). */
  leads: Point[][];
  body: (ctx: Ctx, props: Record<string, PropValue>, style: SymbolStyle) => void;
}

const LINE = 0.125;
const FONT = "Inter Variable, Inter, sans-serif";

function line(ctx: Ctx, x0: number, y0: number, x1: number, y1: number) {
  ctx.beginPath();
  ctx.moveTo(x0, y0);
  ctx.lineTo(x1, y1);
  ctx.stroke();
}

/**
 * Dikdörtgenler yol olarak çizilir: `strokeRect`/`fillRect` SVG dışa
 * aktarımında (svgcanvas) dönüşüm altında çizgi kalınlığını iki kez ölçekliyor.
 */
function strokeBox(ctx: Ctx, x: number, y: number, w: number, h: number) {
  ctx.beginPath();
  ctx.rect(x, y, w, h);
  ctx.stroke();
}

function fillBox(ctx: Ctx, x: number, y: number, w: number, h: number) {
  ctx.beginPath();
  ctx.rect(x, y, w, h);
  ctx.fill();
}

function polyline(ctx: Ctx, pts: Point[]) {
  ctx.beginPath();
  pts.forEach(([x, y], i) => (i === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y)));
  ctx.stroke();
}

function plus(ctx: Ctx, x: number, y: number, s = 0.22) {
  line(ctx, x - s, y, x + s, y);
  line(ctx, x, y - s, x, y + s);
}

function minus(ctx: Ctx, x: number, y: number, s = 0.22) {
  line(ctx, x - s, y, x + s, y);
}

function arrowHead(ctx: Ctx, x0: number, y0: number, x1: number, y1: number, head = 0.3) {
  const a = Math.atan2(y1 - y0, x1 - x0);
  ctx.beginPath();
  ctx.moveTo(x1, y1);
  ctx.lineTo(x1 - head * Math.cos(a - 0.45), y1 - head * Math.sin(a - 0.45));
  ctx.lineTo(x1 - head * Math.cos(a + 0.45), y1 - head * Math.sin(a + 0.45));
  ctx.closePath();
  ctx.fill();
}

function arrow(ctx: Ctx, x0: number, y0: number, x1: number, y1: number, head = 0.25) {
  line(ctx, x0, y0, x1, y1);
  arrowHead(ctx, x0, y0, x1, y1, head);
}

function text(
  ctx: Ctx,
  s: string,
  x: number,
  y: number,
  size = 0.75,
  align: CanvasTextAlign = "center",
) {
  ctx.font = `600 ${size}px ${FONT}`;
  ctx.textAlign = align;
  ctx.textBaseline = "middle";
  ctx.fillText(s, x, y);
}

function circle(ctx: Ctx, x: number, y: number, r: number) {
  ctx.beginPath();
  ctx.arc(x, y, r, 0, Math.PI * 2);
  ctx.stroke();
}

const H = (inner: number): Point[][] => [
  [
    [-2, 0],
    [-inner, 0],
  ],
  [
    [2, 0],
    [inner, 0],
  ],
];

const V = (inner: number): Point[][] => [
  [
    [0, -2],
    [0, -inner],
  ],
  [
    [0, 2],
    [0, inner],
  ],
];

function diodeBody(ctx: Ctx, fill: string | null) {
  ctx.beginPath();
  ctx.moveTo(-0.5, -0.55);
  ctx.lineTo(-0.5, 0.55);
  ctx.lineTo(0.5, 0);
  ctx.closePath();
  if (fill) {
    ctx.fillStyle = fill;
    ctx.fill();
  }
  ctx.stroke();
  line(ctx, 0.5, -0.55, 0.5, 0.55);
}

function meter(ctx: Ctx) {
  circle(ctx, 0, 0, 1.1);
  plus(ctx, -1.55, -0.55, 0.18);
}

// ---------------------------------------------------------------- Transistörler

/** BJT: `top` kollektör (NPN) veya emetör (PNP) ucudur. */
function bjtLeads(pnp: boolean): Point[][] {
  const top: Point[] = [
    [1, -2],
    [1, -1.1],
    [-0.5, -0.35],
  ];
  const bottom: Point[] = [
    [1, 2],
    [1, 1.1],
    [-0.5, 0.35],
  ];
  const base: Point[] = [
    [-2, 0],
    [-0.5, 0],
  ];
  // Pin sırası: C, B, E
  return pnp ? [bottom, base, top] : [top, base, bottom];
}

function bjtBody(pnp: boolean) {
  return (ctx: Ctx) => {
    circle(ctx, 0.15, 0, 1.45);
    ctx.lineWidth = LINE * 2;
    line(ctx, -0.5, -0.8, -0.5, 0.8);
    // Emetör oku: NPN dışarı, PNP içeri
    if (pnp) arrowHead(ctx, 1, -1.1, -0.35, -0.42, 0.42);
    else arrowHead(ctx, -0.5, 0.35, 0.85, 1.03, 0.42);
  };
}

function mosLeads(pmos: boolean): Point[][] {
  const top: Point[] = [
    [1, -2],
    [1, -0.7],
    [-0.35, -0.7],
  ];
  const bottom: Point[] = [
    [1, 2],
    [1, 0.7],
    [-0.35, 0.7],
  ];
  const gate: Point[] = [
    [-2, 0],
    [-0.75, 0],
  ];
  // Pin sırası: D, G, S
  return pmos ? [bottom, gate, top] : [top, gate, bottom];
}

function mosBody(pmos: boolean) {
  return (ctx: Ctx) => {
    circle(ctx, 0.15, 0, 1.45);
    // Kapı plakası
    line(ctx, -0.75, -0.8, -0.75, 0.8);
    // Kanal (zenginleştirmeli: kesikli)
    ctx.lineWidth = LINE * 1.6;
    line(ctx, -0.35, -0.95, -0.35, -0.45);
    line(ctx, -0.35, -0.2, -0.35, 0.2);
    line(ctx, -0.35, 0.45, -0.35, 0.95);
    ctx.lineWidth = LINE;
    // Gövde kaynağa bağlı
    const sy = pmos ? -0.7 : 0.7;
    polyline(ctx, [
      [0.4, 0],
      [1, 0],
      [1, sy],
    ]);
    if (pmos) arrow(ctx, -0.35, 0, 0.45, 0, 0.3);
    else arrow(ctx, 0.4, 0, -0.3, 0, 0.3);
  };
}

// ---------------------------------------------------------------- Lojik kapılar

const GATE_LEADS = (inputEnd: number, outputStart: number): Point[][] => [
  [
    [-3, -1],
    [inputEnd, -1],
  ],
  [
    [-3, 1],
    [inputEnd, 1],
  ],
  [
    [3, 0],
    [outputStart, 0],
  ],
];

function bubble(ctx: Ctx, x: number) {
  circle(ctx, x, 0, 0.28);
}

function andShape(ctx: Ctx) {
  ctx.beginPath();
  ctx.moveTo(-1.5, -1.5);
  ctx.lineTo(0, -1.5);
  ctx.arc(0, 0, 1.5, -Math.PI / 2, Math.PI / 2);
  ctx.lineTo(-1.5, 1.5);
  ctx.closePath();
  ctx.stroke();
}

function orShape(ctx: Ctx, xor: boolean) {
  ctx.beginPath();
  ctx.moveTo(-1.5, -1.5);
  ctx.quadraticCurveTo(-0.8, 0, -1.5, 1.5);
  ctx.quadraticCurveTo(0.5, 1.5, 1.5, 0);
  ctx.quadraticCurveTo(0.5, -1.5, -1.5, -1.5);
  ctx.closePath();
  ctx.stroke();
  if (xor) {
    ctx.beginPath();
    ctx.moveTo(-1.9, -1.5);
    ctx.quadraticCurveTo(-1.2, 0, -1.9, 1.5);
    ctx.stroke();
  }
}

function flipFlop(inputs: [string, number][], clockY: number) {
  return (ctx: Ctx, _props: Record<string, PropValue>, style: SymbolStyle) => {
    ctx.fillStyle = "rgba(255,255,255,0.03)";
    fillBox(ctx, -2, -2, 4, 4);
    strokeBox(ctx, -2, -2, 4, 4);
    ctx.fillStyle = style.stroke;
    for (const [label, y] of inputs) text(ctx, label, -1.7, y, 0.6, "left");
    // Saat girişi üçgeni
    polyline(ctx, [
      [-2, clockY - 0.3],
      [-1.6, clockY],
      [-2, clockY + 0.3],
    ]);
    text(ctx, "Q", 1.7, -1, 0.6, "right");
    text(ctx, "Q", 1.7, 1, 0.6, "right");
    ctx.lineWidth = LINE * 0.6;
    line(ctx, 1.12, 0.62, 1.52, 0.62);
  };
}

function ffLeads(inputs: number[]): Point[][] {
  return [
    ...inputs.map((y): Point[] => [
      [-3, y],
      [-2, y],
    ]),
    [
      [3, -1],
      [2, -1],
    ],
    [
      [3, 1],
      [2, 1],
    ],
  ];
}

function levelBox(ctx: Ctx, style: SymbolStyle, label: string | null) {
  const on = style.level === true;
  ctx.fillStyle = on ? withAlpha(style.high ?? "#22c55e", 0.25) : "rgba(255,255,255,0.03)";
  ctx.beginPath();
  ctx.roundRect(-1.2, -0.8, 2, 1.6, 0.25);
  ctx.fill();
  ctx.stroke();
  if (label !== null) {
    ctx.fillStyle = on ? (style.high ?? "#22c55e") : style.stroke;
    text(ctx, label, -0.2, 0.03, 0.95);
  }
}

// ---------------------------------------------------------------- Faz 3

function contacts(ctx: Ctx, points: Point[]) {
  ctx.beginPath();
  for (const [x, y] of points) {
    ctx.moveTo(x + 0.14, y);
    ctx.arc(x, y, 0.14, 0, Math.PI * 2);
  }
  ctx.stroke();
}

/** Kutu biçimli entegreler için pin etiketleri. */
function pinLabels(ctx: Ctx, labels: [string, number, number, CanvasTextAlign][], size = 0.5) {
  ctx.save();
  ctx.globalAlpha = 0.75;
  for (const [label, x, y, align] of labels) text(ctx, label, x, y, size, align);
  ctx.restore();
}

function chipBox(ctx: Ctx, x0: number, y0: number, w: number, h: number) {
  ctx.fillStyle = "rgba(255,255,255,0.03)";
  fillBox(ctx, x0, y0, w, h);
  strokeBox(ctx, x0, y0, w, h);
}

/** 7 segmentin yerel koordinatları (a … g). */
const SEGMENTS: [Point, Point][] = (() => {
  const [x0, x1, y0, ym, y1] = [-0.65, 0.95, -1.9, 0, 1.9];
  return [
    [
      [x0, y0],
      [x1, y0],
    ],
    [
      [x1, y0],
      [x1, ym],
    ],
    [
      [x1, ym],
      [x1, y1],
    ],
    [
      [x0, y1],
      [x1, y1],
    ],
    [
      [x0, ym],
      [x0, y1],
    ],
    [
      [x0, y0],
      [x0, ym],
    ],
    [
      [x0, ym],
      [x1, ym],
    ],
  ];
})();

const SEVEN_SEG_LEADS: Point[][] = [-3, -2, -1, 0, 1, 2, 3].map((y) => [
  [-3, y],
  [-2, y],
]);

const HEX_DECODER_LEADS: Point[][] = [
  ...[-3, -1, 1, 3].map((y): Point[] => [
    [-3, y],
    [-2, y],
  ]),
  ...[-3, -2, -1, 0, 1, 2, 3].map((y): Point[] => [
    [3, y],
    [2, y],
  ]),
];

// ---------------------------------------------------------------- Tanımlar

const SYMBOLS: Record<ComponentKind, SymbolDef> = {
  potentiometer: {
    leads: [
      [
        [-2, 0],
        [-1.2, 0],
      ],
      [
        [2, 0],
        [1.2, 0],
      ],
      [
        [0, -2],
        [0, -0.75],
      ],
    ],
    body: (ctx) => {
      strokeBox(ctx, -1.2, -0.4, 2.4, 0.8);
      arrowHead(ctx, 0, -1.2, 0, -0.42, 0.35);
    },
  },
  zener: {
    leads: H(0.5),
    body: (ctx) => {
      diodeBody(ctx, null);
      polyline(ctx, [
        [0.25, -0.75],
        [0.5, -0.55],
        [0.5, 0.55],
        [0.75, 0.75],
      ]);
    },
  },
  pushButton: {
    leads: H(1),
    body: (ctx, _props, style) => {
      contacts(ctx, [
        [-1, 0],
        [1, 0],
      ]);
      const y = style.level ? -0.18 : -0.6;
      line(ctx, -1.15, y, 1.15, y);
      line(ctx, 0, y, 0, -1.15);
      line(ctx, -0.45, -1.15, 0.45, -1.15);
    },
  },
  spdtSwitch: {
    leads: [
      [
        [-2, 0],
        [-1, 0],
      ],
      [
        [2, -1],
        [1, -1],
      ],
      [
        [2, 1],
        [1, 1],
      ],
    ],
    body: (ctx, props) => {
      contacts(ctx, [
        [-1, 0],
        [1, -1],
        [1, 1],
      ]);
      line(ctx, -1, 0, 0.88, props.toB ? 0.88 : -0.88);
    },
  },
  lamp: {
    leads: H(0.9),
    body: (ctx, _props, style) => {
      const glow = style.glow ?? 0;
      if (glow > 0.02) {
        const g = ctx.createRadialGradient(0, 0, 0.2, 0, 0, 2.6);
        g.addColorStop(0, withAlpha("#fde68a", 0.7 * glow));
        g.addColorStop(1, withAlpha("#fde68a", 0));
        ctx.fillStyle = g;
        fillBox(ctx, -2.6, -2.6, 5.2, 5.2);
        ctx.fillStyle = withAlpha("#fde68a", 0.25 + 0.6 * glow);
        ctx.beginPath();
        ctx.arc(0, 0, 0.9, 0, Math.PI * 2);
        ctx.fill();
      }
      circle(ctx, 0, 0, 0.9);
      const d = 0.9 * Math.SQRT1_2;
      line(ctx, -d, -d, d, d);
      line(ctx, -d, d, d, -d);
    },
  },
  pulseVoltage: {
    leads: V(1),
    body: (ctx) => {
      circle(ctx, 0, 0, 1);
      polyline(ctx, [
        [-0.6, 0.35],
        [-0.3, 0.35],
        [-0.3, -0.35],
        [0.15, -0.35],
        [0.15, 0.35],
        [0.6, 0.35],
      ]);
      ctx.lineWidth = LINE * 0.8;
      plus(ctx, 0.85, -1.4);
    },
  },
  timer555: {
    leads: [
      [
        [1, 4],
        [1, 3],
      ],
      [
        [-3, 1],
        [-2, 1],
      ],
      [
        [3, -1],
        [2, -1],
      ],
      [
        [-1, -4],
        [-1, -3],
      ],
      [
        [-1, 4],
        [-1, 3],
      ],
      [
        [-3, -1],
        [-2, -1],
      ],
      [
        [3, 1],
        [2, 1],
      ],
      [
        [1, -4],
        [1, -3],
      ],
    ],
    body: (ctx, _props, style) => {
      chipBox(ctx, -2, -3, 4, 6);
      ctx.fillStyle = style.stroke;
      text(ctx, "555", 0, 0.05, 0.9);
      pinLabels(ctx, [
        ["THR", -1.8, -1, "left"],
        ["TRIG", -1.8, 1, "left"],
        ["OUT", 1.8, -1, "right"],
        ["DIS", 1.8, 1, "right"],
        ["RST", -1, -2.5, "center"],
        ["VCC", 1, -2.5, "center"],
        ["CV", -1, 2.5, "center"],
        ["GND", 1, 2.5, "center"],
      ]);
    },
  },
  sevenSegment: {
    leads: SEVEN_SEG_LEADS,
    body: (ctx, props, style) => {
      ctx.fillStyle = "#07090d";
      fillBox(ctx, -2, -4, 4, 8);
      strokeBox(ctx, -2, -4, 4, 8);
      const color = LED_COLORS[String(props.color)] ?? LED_COLORS.red!;
      ctx.lineCap = "round";
      SEGMENTS.forEach(([a, b], i) => {
        const on = style.segments?.[i] === true;
        const shrink = 0.18;
        const dx = Math.sign(b[0] - a[0]) * shrink;
        const dy = Math.sign(b[1] - a[1]) * shrink;
        ctx.strokeStyle = on ? color : "rgba(255,255,255,0.07)";
        ctx.lineWidth = 0.32;
        if (on) {
          ctx.shadowColor = color;
          ctx.shadowBlur = 12;
        }
        line(ctx, a[0] + dx, a[1] + dy, b[0] - dx, b[1] - dy);
        ctx.shadowBlur = 0;
      });
      ctx.fillStyle = style.stroke;
      pinLabels(
        ctx,
        ["a", "b", "c", "d", "e", "f", "g"].map(
          (l, i): [string, number, number, CanvasTextAlign] => [l, -1.75, i - 3, "left"],
        ),
        0.45,
      );
    },
  },
  hexDecoder: {
    leads: HEX_DECODER_LEADS,
    body: (ctx, _props, style) => {
      chipBox(ctx, -2, -4, 4, 8);
      ctx.fillStyle = style.stroke;
      text(ctx, "HEX", 0, -0.35, 0.6);
      text(ctx, "7SEG", 0, 0.45, 0.5);
      pinLabels(ctx, [
        ...["A", "B", "C", "D"].map((l, i): [string, number, number, CanvasTextAlign] => [
          l,
          -1.8,
          i * 2 - 3,
          "left",
        ]),
        ...["a", "b", "c", "d", "e", "f", "g"].map(
          (l, i): [string, number, number, CanvasTextAlign] => [l, 1.8, i - 3, "right"],
        ),
      ]);
    },
  },

  ground: {
    leads: [
      [
        [0, 0],
        [0, 0.6],
      ],
    ],
    body: (ctx) => {
      line(ctx, -0.8, 0.6, 0.8, 0.6);
      line(ctx, -0.5, 0.95, 0.5, 0.95);
      line(ctx, -0.2, 1.3, 0.2, 1.3);
    },
  },
  resistor: {
    leads: H(1.2),
    body: (ctx) => strokeBox(ctx, -1.2, -0.4, 2.4, 0.8),
  },
  capacitor: {
    leads: H(0.25),
    body: (ctx) => {
      ctx.lineWidth = LINE * 1.4;
      line(ctx, -0.25, -0.8, -0.25, 0.8);
      line(ctx, 0.25, -0.8, 0.25, 0.8);
    },
  },
  inductor: {
    leads: H(1.2),
    body: (ctx) => {
      ctx.beginPath();
      for (let i = 0; i < 4; i++) ctx.arc(-0.9 + 0.6 * i, 0, 0.3, Math.PI, 0);
      ctx.stroke();
    },
  },
  dcVoltage: {
    leads: V(0.3),
    body: (ctx) => {
      line(ctx, -0.9, -0.3, 0.9, -0.3);
      ctx.lineWidth = LINE * 2.2;
      line(ctx, -0.5, 0.3, 0.5, 0.3);
      ctx.lineWidth = LINE * 0.8;
      plus(ctx, 0.85, -0.9);
    },
  },
  dcCurrent: {
    leads: V(1),
    body: (ctx) => {
      circle(ctx, 0, 0, 1);
      arrow(ctx, 0, 0.6, 0, -0.6);
    },
  },
  acVoltage: {
    leads: V(1),
    body: (ctx) => {
      circle(ctx, 0, 0, 1);
      ctx.beginPath();
      for (let i = 0; i <= 24; i++) {
        const t = i / 24;
        const x = -0.6 + 1.2 * t;
        const y = -0.4 * Math.sin(Math.PI * 2 * t);
        if (i === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
      }
      ctx.stroke();
      ctx.lineWidth = LINE * 0.8;
      plus(ctx, 0.85, -1.4);
    },
  },
  diode: {
    leads: H(0.5),
    body: (ctx) => diodeBody(ctx, null),
  },
  led: {
    leads: H(0.5),
    body: (ctx, props, style) => {
      const color = LED_COLORS[String(props.color)] ?? LED_COLORS.red!;
      const glow = style.glow ?? 0;
      if (glow > 0.02) {
        const g = ctx.createRadialGradient(0, 0, 0.1, 0, 0, 2.2);
        g.addColorStop(0, withAlpha(color, 0.55 * glow));
        g.addColorStop(1, withAlpha(color, 0));
        ctx.fillStyle = g;
        fillBox(ctx, -2.2, -2.2, 4.4, 4.4);
      }
      diodeBody(ctx, glow > 0.02 ? withAlpha(color, 0.35 + 0.65 * glow) : withAlpha(color, 0.15));
      ctx.fillStyle = style.stroke;
      ctx.lineWidth = LINE * 0.8;
      arrow(ctx, 0.05, -0.75, 0.5, -1.2, 0.2);
      arrow(ctx, 0.45, -0.6, 0.9, -1.05, 0.2);
    },
  },
  switch: {
    leads: H(1),
    body: (ctx, props) => {
      ctx.beginPath();
      ctx.arc(-1, 0, 0.14, 0, Math.PI * 2);
      ctx.arc(1, 0, 0.14, 0, Math.PI * 2);
      ctx.stroke();
      if (props.closed) line(ctx, -1, 0, 1, 0);
      else line(ctx, -1, 0, 0.85, -0.9);
    },
  },
  voltmeter: { leads: H(1.1), body: meter },
  ammeter: { leads: H(1.1), body: meter },

  npn: { leads: bjtLeads(false), body: bjtBody(false) },
  pnp: { leads: bjtLeads(true), body: bjtBody(true) },
  nmos: { leads: mosLeads(false), body: mosBody(false) },
  pmos: { leads: mosLeads(true), body: mosBody(true) },
  opAmp: {
    leads: GATE_LEADS(-2, 2),
    body: (ctx) => {
      ctx.beginPath();
      ctx.moveTo(-2, -2);
      ctx.lineTo(-2, 2);
      ctx.lineTo(2, 0);
      ctx.closePath();
      ctx.stroke();
      ctx.lineWidth = LINE * 0.9;
      minus(ctx, -1.45, -1);
      plus(ctx, -1.45, 1);
    },
  },

  and: { leads: GATE_LEADS(-1.5, 1.5), body: andShape },
  nand: {
    leads: GATE_LEADS(-1.5, 2.08),
    body: (ctx) => {
      andShape(ctx);
      bubble(ctx, 1.8);
    },
  },
  or: { leads: GATE_LEADS(-1.3, 1.5), body: (ctx) => orShape(ctx, false) },
  nor: {
    leads: GATE_LEADS(-1.3, 2.08),
    body: (ctx) => {
      orShape(ctx, false);
      bubble(ctx, 1.8);
    },
  },
  xor: { leads: GATE_LEADS(-1.68, 1.5), body: (ctx) => orShape(ctx, true) },
  xnor: {
    leads: GATE_LEADS(-1.68, 2.08),
    body: (ctx) => {
      orShape(ctx, true);
      bubble(ctx, 1.8);
    },
  },
  not: {
    leads: [
      [
        [-3, 0],
        [-1.2, 0],
      ],
      [
        [3, 0],
        [1.58, 0],
      ],
    ],
    body: (ctx) => {
      ctx.beginPath();
      ctx.moveTo(-1.2, -1);
      ctx.lineTo(-1.2, 1);
      ctx.lineTo(1, 0);
      ctx.closePath();
      ctx.stroke();
      bubble(ctx, 1.3);
    },
  },

  dFlipFlop: {
    leads: ffLeads([-1, 1]),
    body: flipFlop([["D", -1]], 1),
  },
  tFlipFlop: {
    leads: ffLeads([-1, 1]),
    body: flipFlop([["T", -1]], 1),
  },
  jkFlipFlop: {
    leads: ffLeads([-1, 0, 1]),
    body: flipFlop(
      [
        ["J", -1],
        ["K", 1],
      ],
      0,
    ),
  },

  logicInput: {
    leads: [
      [
        [2, 0],
        [0.8, 0],
      ],
    ],
    body: (ctx, props, style) => levelBox(ctx, style, props.high ? "1" : "0"),
  },
  clock: {
    leads: [
      [
        [2, 0],
        [0.8, 0],
      ],
    ],
    body: (ctx, _props, style) => {
      levelBox(ctx, style, null);
      polyline(ctx, [
        [-0.9, 0.35],
        [-0.6, 0.35],
        [-0.6, -0.35],
        [-0.2, -0.35],
        [-0.2, 0.35],
        [0.2, 0.35],
        [0.2, -0.35],
        [0.5, -0.35],
      ]);
    },
  },
  logicProbe: {
    leads: [
      [
        [-2, 0],
        [-0.6, 0],
      ],
    ],
    body: (ctx, _props, style) => {
      const on = style.level === true;
      const color = style.high ?? "#22c55e";
      if (on) {
        const g = ctx.createRadialGradient(0.2, 0, 0.2, 0.2, 0, 1.8);
        g.addColorStop(0, withAlpha(color, 0.45));
        g.addColorStop(1, withAlpha(color, 0));
        ctx.fillStyle = g;
        fillBox(ctx, -1.6, -1.8, 3.6, 3.6);
      }
      ctx.fillStyle = on ? withAlpha(color, 0.85) : "rgba(255,255,255,0.04)";
      ctx.beginPath();
      ctx.arc(0.2, 0, 0.8, 0, Math.PI * 2);
      ctx.fill();
      ctx.stroke();
      if (style.level !== undefined && style.level !== null) {
        ctx.fillStyle = on ? "#0b0e14" : style.stroke;
        text(ctx, on ? "1" : "0", 0.2, 0.03, 0.9);
      }
    },
  },
};

/** Sembolü çizer (bağlam bileşen merkezine taşınmış ve döndürülmüş olmalı). */
export function drawSymbol(
  ctx: Ctx,
  kind: ComponentKind,
  props: Record<string, PropValue>,
  style: SymbolStyle,
) {
  const def = SYMBOLS[kind];
  ctx.lineCap = "round";
  ctx.lineJoin = "round";
  def.leads.forEach((pts, i) => {
    ctx.strokeStyle = style.leads[i] ?? style.stroke;
    ctx.lineWidth = LINE;
    polyline(ctx, pts);
  });
  ctx.save();
  ctx.strokeStyle = style.stroke;
  ctx.fillStyle = style.stroke;
  ctx.lineWidth = LINE;
  def.body(ctx, props, style);
  ctx.restore();
}

/** Bileşenin pinlerinden gövdeye uzanan uçlar (yerel koordinat). */
export function leadPaths(kind: ComponentKind): Point[][] {
  return SYMBOLS[kind].leads;
}

/** `#rrggbb` rengine saydamlık ekler. */
export function withAlpha(hex: string, alpha: number): string {
  const a = Math.round(Math.max(0, Math.min(1, alpha)) * 255)
    .toString(16)
    .padStart(2, "0");
  return `${hex.slice(0, 7)}${a}`;
}
