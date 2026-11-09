/**
 * Belge üzerinde saf (yan etkisiz) düzenleme işlemleri.
 * Her işlem yeni bir belge döner; geri al / yinele bu sayede basittir.
 */
import { LIBRARY, pinPositions, pointKey, samePoint } from "./library";
import type { CircuitComponent, CircuitDocument, ComponentKind, Point, Wire } from "./types";

export interface Selection {
  components: string[];
  wires: string[];
}

export const EMPTY_SELECTION: Selection = { components: [], wires: [] };

/** `R` önekli bileşenler için sıradaki boş isim: R1, R2 ... */
export function nextId(ids: Iterable<string>, prefix: string): string {
  const re = new RegExp(`^${prefix}(\\d+)$`);
  let max = 0;
  for (const id of ids) {
    const m = re.exec(id);
    if (m) max = Math.max(max, Number(m[1]));
  }
  return `${prefix}${max + 1}`;
}

export function addComponent(
  doc: CircuitDocument,
  kind: ComponentKind,
  pos: Point,
  rot: number,
): [CircuitDocument, string] {
  const def = LIBRARY[kind];
  const id = nextId(
    doc.components.map((c) => c.id),
    def.prefix,
  );
  const component: CircuitComponent = { id, kind, pos, rot, props: { ...def.props } };
  return [{ ...doc, components: [...doc.components, component] }, id];
}

/** Ardışık noktalar arasına teller ekler (sıfır uzunluklu parçalar atlanır). */
export function addWirePath(doc: CircuitDocument, points: Point[]): CircuitDocument {
  const ids = doc.wires.map((w) => w.id);
  const wires: Wire[] = [];
  for (let i = 1; i < points.length; i++) {
    const a = points[i - 1]!;
    const b = points[i]!;
    if (samePoint(a, b)) continue;
    const id = nextId(ids, "W");
    ids.push(id);
    wires.push({ id, a, b });
  }
  return wires.length ? { ...doc, wires: [...doc.wires, ...wires] } : doc;
}

/**
 * Bileşenleri ve telleri günceller; eski pin / tel ucu konumlarına bağlı olan
 * seçili olmayan tellerin uçları yeni konumlara taşınır (lastik bant).
 */
function applyWithFollowingWires(
  doc: CircuitDocument,
  selection: Selection,
  updateComponent: (c: CircuitComponent) => CircuitComponent,
  updateWire: (w: Wire) => Wire,
): CircuitDocument {
  const moved = new Map<string, Point>();
  const selComps = new Set(selection.components);
  const selWires = new Set(selection.wires);

  const components = doc.components.map((c) => {
    if (!selComps.has(c.id)) return c;
    const next = updateComponent(c);
    const before = pinPositions(c);
    const after = pinPositions(next);
    before.forEach((p, i) => moved.set(pointKey(p), after[i]!));
    return next;
  });

  const wires = doc.wires.map((w) => {
    if (!selWires.has(w.id)) return w;
    const next = updateWire(w);
    moved.set(pointKey(w.a), next.a);
    moved.set(pointKey(w.b), next.b);
    return next;
  });

  const followed = wires
    .map((w) =>
      selWires.has(w.id)
        ? w
        : {
            ...w,
            a: moved.get(pointKey(w.a)) ?? w.a,
            b: moved.get(pointKey(w.b)) ?? w.b,
          },
    )
    .filter((w) => !samePoint(w.a, w.b));

  return { ...doc, components, wires: followed };
}

export function moveSelection(
  doc: CircuitDocument,
  selection: Selection,
  [dx, dy]: Point,
): CircuitDocument {
  if (dx === 0 && dy === 0) return doc;
  const shift = (p: Point): Point => [p[0] + dx, p[1] + dy];
  return applyWithFollowingWires(
    doc,
    selection,
    (c) => ({ ...c, pos: shift(c.pos) }),
    (w) => ({ ...w, a: shift(w.a), b: shift(w.b) }),
  );
}

/** Seçili bileşenleri kendi merkezleri etrafında 90° döndürür. */
export function rotateSelection(doc: CircuitDocument, selection: Selection): CircuitDocument {
  if (selection.components.length === 0) return doc;
  return applyWithFollowingWires(
    doc,
    { components: selection.components, wires: [] },
    (c) => ({ ...c, rot: (c.rot + 1) % 4 }),
    (w) => w,
  );
}

export function deleteSelection(doc: CircuitDocument, selection: Selection): CircuitDocument {
  const comps = new Set(selection.components);
  const wires = new Set(selection.wires);
  return {
    ...doc,
    components: doc.components.filter((c) => !comps.has(c.id)),
    wires: doc.wires.filter((w) => !wires.has(w.id)),
  };
}

export function updateComponent(
  doc: CircuitDocument,
  id: string,
  update: (c: CircuitComponent) => CircuitComponent,
): CircuitDocument {
  return {
    ...doc,
    components: doc.components.map((c) => (c.id === id ? update(c) : c)),
  };
}
