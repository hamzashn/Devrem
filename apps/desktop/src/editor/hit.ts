/** Fare ile seçim için isabet testleri (dünya koordinatlarında). */
import {
  componentBounds,
  distanceToSegment,
  pinPositions,
  samePoint,
  strictlyInside,
} from "./library";
import type { CircuitComponent, CircuitDocument, Point, Wire } from "./types";

/** İmlece en yakın pin veya tel ucu. */
export function hitAnchor(doc: CircuitDocument, x: number, y: number, radius = 0.45): Point | null {
  let best: Point | null = null;
  let bestDist = radius;
  const consider = (p: Point) => {
    const d = Math.hypot(p[0] - x, p[1] - y);
    if (d <= bestDist) {
      best = p;
      bestDist = d;
    }
  };
  for (const c of doc.components) pinPositions(c).forEach(consider);
  for (const w of doc.wires) {
    consider(w.a);
    consider(w.b);
  }
  return best;
}

export function hitComponent(doc: CircuitDocument, x: number, y: number): CircuitComponent | null {
  for (let i = doc.components.length - 1; i >= 0; i--) {
    const c = doc.components[i]!;
    const [x0, y0, x1, y1] = componentBounds(c);
    if (x >= x0 && x <= x1 && y >= y0 && y <= y1) return c;
  }
  return null;
}

export function hitWire(doc: CircuitDocument, x: number, y: number, tolerance = 0.35): Wire | null {
  let best: Wire | null = null;
  let bestDist = tolerance;
  for (const w of doc.wires) {
    const d = distanceToSegment(x, y, w.a, w.b);
    if (d <= bestDist) {
      best = w;
      bestDist = d;
    }
  }
  return best;
}

/** Nokta bir pine, tel ucuna veya telin üzerine denk geliyor mu? */
export function isConnectionPoint(doc: CircuitDocument, p: Point): boolean {
  return (
    doc.components.some((c) => pinPositions(c).some((q) => samePoint(p, q))) ||
    doc.wires.some((w) => samePoint(p, w.a) || samePoint(p, w.b) || strictlyInside(p, w.a, w.b))
  );
}

/** Dikdörtgen içinde kalan bileşen ve teller. */
export function itemsInRect(doc: CircuitDocument, a: Point, b: Point) {
  const [x0, x1] = [Math.min(a[0], b[0]), Math.max(a[0], b[0])];
  const [y0, y1] = [Math.min(a[1], b[1]), Math.max(a[1], b[1])];
  const inside = (p: Point) => p[0] >= x0 && p[0] <= x1 && p[1] >= y0 && p[1] <= y1;
  return {
    components: doc.components.filter((c) => inside(c.pos)).map((c) => c.id),
    wires: doc.wires.filter((w) => inside(w.a) && inside(w.b)).map((w) => w.id),
  };
}
