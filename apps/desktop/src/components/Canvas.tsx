import { useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { hitAnchor, hitComponent, hitWire, isConnectionPoint, itemsInRect } from "../editor/hit";
import { TOGGLE_PROP, pinPositions, samePoint } from "../editor/library";
import { moveSelection, type Selection } from "../editor/ops";
import {
  advanceAnimation,
  drawScene,
  readTheme,
  routeTo,
  type Animation,
  type SceneInput,
} from "../editor/render";
import type { CircuitDocument, ComponentKind, Point } from "../editor/types";
import { formatValue } from "../editor/units";
import {
  GRID,
  addWirePath,
  commit,
  placeComponent,
  replace,
  setCursor,
  setSelection,
  setTool,
  setView,
  setWireDraft,
  useEditor,
} from "../store/editor";
import { flipToggle, pressButton, useSim } from "../store/sim";
import { isTyping } from "../lib/keyboard";
import { openGallery } from "../store/ui";
import { Logo } from "./Logo";

export const DRAG_MIME = "application/x-devrem-kind";

type Drag =
  | { type: "pan"; startX: number; startY: number; viewX: number; viewY: number }
  | { type: "move"; start: Point; startDoc: CircuitDocument; selection: Selection; moved: boolean }
  | { type: "box"; start: [number, number]; current: [number, number]; additive: boolean }
  | { type: "wire"; start: Point };

/** Fare konumunu dünya koordinatına çevirir. */
function toWorld(canvas: HTMLCanvasElement, clientX: number, clientY: number): [number, number] {
  const rect = canvas.getBoundingClientRect();
  const { view } = useEditor.getState();
  const scale = GRID * view.zoom;
  return [(clientX - rect.left - view.x) / scale, (clientY - rect.top - view.y) / scale];
}

const snap = ([x, y]: [number, number]): Point => [Math.round(x), Math.round(y)];

export function Canvas() {
  const { t } = useTranslation();
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const isEmpty = useEditor((s) => s.doc.components.length === 0 && s.doc.wires.length === 0);

  useEffect(() => {
    const canvas = canvasRef.current!;
    const ctx = canvas.getContext("2d")!;
    const theme = readTheme();
    const animation: Animation = { topology: null, segmentPhase: [], elementPhase: [] };

    let size = { width: 0, height: 0, dpr: 1 };
    let dirty = true;
    let drag: Drag | null = null;
    let pointer: [number, number] = [0, 0];
    let screenPointer: [number, number] = [0, 0];
    let spaceHeld = false;
    let heldButton: string | null = null;
    let hover: SceneInput["hover"] = { anchor: null, componentId: null, wireId: null };

    const invalidate = () => (dirty = true);

    // Boyut değişimi
    const resize = () => {
      const rect = canvas.getBoundingClientRect();
      const dpr = window.devicePixelRatio || 1;
      size = { width: rect.width, height: rect.height, dpr };
      canvas.width = Math.round(rect.width * dpr);
      canvas.height = Math.round(rect.height * dpr);
      invalidate();
    };
    const observer = new ResizeObserver(resize);
    observer.observe(canvas);
    resize();

    // Başlangıçta orijini ortala
    const initialView = useEditor.getState().view;
    if (initialView.x === 0 && initialView.y === 0) {
      setView({ x: Math.round(size.width / 2), y: Math.round(size.height / 2), zoom: 1.25 });
    }

    const unsubEditor = useEditor.subscribe(invalidate);
    const unsubSim = useSim.subscribe(invalidate);

    // İmleç altındaki öğeler
    const updateHover = () => {
      const { doc, wireDraft, tool } = useEditor.getState();
      const [x, y] = pointer;
      const placing = tool.type === "place";
      const anchor = placing ? null : hitAnchor(doc, x, y);
      const component = placing || anchor ? null : hitComponent(doc, x, y);
      const wire = placing || anchor || component || wireDraft ? null : hitWire(doc, x, y);
      hover = { anchor, componentId: component?.id ?? null, wireId: wire?.id ?? null };
      const running = useSim.getState().mode === "running";
      canvas.style.cursor =
        drag?.type === "pan"
          ? "grabbing"
          : anchor || wireDraft
            ? "crosshair"
            : component &&
                (TOGGLE_PROP[component.kind] || component.kind === "pushButton") &&
                running
              ? "pointer"
              : component || wire
                ? "move"
                : "default";
    };

    // Simülasyon sırasında imlecin altındaki noktanın gerilimi
    const tooltip = (): SceneInput["tooltip"] => {
      const { frame, topology, elementIndex } = useSim.getState();
      if (!frame || !topology || drag) return null;
      const { doc } = useEditor.getState();
      let node: number | undefined;
      if (hover.wireId) {
        node = topology.segments.find((s) => s.wireId === hover.wireId)?.node;
      } else if (hover.anchor) {
        const anchor = hover.anchor;
        for (const c of doc.components) {
          const pin = pinPositions(c).findIndex((p) => samePoint(p, anchor));
          if (pin < 0) continue;
          if (c.kind === "ground") node = 0;
          else {
            const i = elementIndex.get(c.id);
            node = i === undefined ? undefined : topology.elements[i]?.nodes[pin];
          }
          break;
        }
        if (node === undefined) {
          const w = doc.wires.find((w) => samePoint(w.a, anchor) || samePoint(w.b, anchor));
          if (w) node = topology.segments.find((s) => s.wireId === w.id)?.node;
        }
      }
      if (node === undefined) return null;
      const v = frame.nodeVoltages[node] ?? 0;
      return { x: screenPointer[0], y: screenPointer[1], text: formatValue(v, "V", 4) };
    };

    // Çizim döngüsü
    let last = performance.now();
    let raf = 0;
    const loop = (now: number) => {
      const dt = Math.min(0.1, (now - last) / 1000);
      last = now;
      const editor = useEditor.getState();
      const sim = useSim.getState();
      const animating = sim.frame !== null && sim.topology !== null;
      if (animating) advanceAnimation(animation, sim, dt);
      if (dirty || animating) {
        dirty = false;
        drawScene(ctx, {
          ...size,
          theme,
          doc: editor.doc,
          view: editor.view,
          selection: editor.selection,
          tool: editor.tool,
          wireDraft: editor.wireDraft,
          cursor: editor.cursor,
          hover,
          box: drag?.type === "box" ? [drag.start, drag.current] : null,
          sim,
          animation,
          tooltip: tooltip(),
        });
      }
      raf = requestAnimationFrame(loop);
    };
    raf = requestAnimationFrame(loop);

    // ------------------------------------------------------------ Tel çizimi

    /** Taslağa bir nokta ekler; bağlantı noktasına ulaşınca teli bitirir. */
    const addDraftPoint = (p: Point, forceFinish: boolean) => {
      const { wireDraft, doc } = useEditor.getState();
      if (!wireDraft) return;
      const lastPoint = wireDraft.at(-1)!;
      const points = samePoint(lastPoint, p) ? wireDraft : [...wireDraft, ...routeTo(lastPoint, p)];
      const finish =
        forceFinish ||
        (points.length > 1 && !samePoint(p, wireDraft[0]!) && isConnectionPoint(doc, p));
      if (finish) {
        addWirePath(points);
        setWireDraft(null);
      } else {
        setWireDraft(points);
      }
    };

    // ------------------------------------------------------------ Fare

    const onPointerDown = (e: PointerEvent) => {
      canvas.setPointerCapture(e.pointerId);
      pointer = toWorld(canvas, e.clientX, e.clientY);
      const p = snap(pointer);
      const state = useEditor.getState();

      if (e.button === 1 || (e.button === 0 && spaceHeld)) {
        drag = {
          type: "pan",
          startX: e.clientX,
          startY: e.clientY,
          viewX: state.view.x,
          viewY: state.view.y,
        };
        updateHover();
        return;
      }
      if (e.button === 2) {
        if (state.wireDraft) setWireDraft(null);
        else if (state.tool.type !== "select") setTool({ type: "select" });
        return;
      }
      if (e.button !== 0) return;

      if (state.tool.type === "place") {
        placeComponent(state.tool.kind, p, state.tool.rot);
        return;
      }
      if (state.wireDraft) return; // tıklama pointerup'ta işlenir

      const anchor = hitAnchor(state.doc, pointer[0], pointer[1]);
      if (anchor) {
        setWireDraft([anchor]);
        drag = { type: "wire", start: anchor };
        return;
      }

      const component = hitComponent(state.doc, pointer[0], pointer[1]);
      if (component?.kind === "pushButton" && useSim.getState().mode === "running") {
        heldButton = component.id;
        pressButton(component.id, true);
        return;
      }
      if (component && TOGGLE_PROP[component.kind] && useSim.getState().mode !== "idle") {
        flipToggle(component.id);
        return;
      }
      const wire = component ? null : hitWire(state.doc, pointer[0], pointer[1]);
      if (component || wire) {
        const sel = state.selection;
        const isSelected = component
          ? sel.components.includes(component.id)
          : sel.wires.includes(wire!.id);
        let selection = sel;
        if (!isSelected) {
          const item: Selection = component
            ? { components: [component.id], wires: [] }
            : { components: [], wires: [wire!.id] };
          selection = e.shiftKey
            ? {
                components: [...sel.components, ...item.components],
                wires: [...sel.wires, ...item.wires],
              }
            : item;
          setSelection(selection);
        }
        drag = { type: "move", start: p, startDoc: state.doc, selection, moved: false };
      } else {
        if (!e.shiftKey) setSelection({ components: [], wires: [] });
        drag = { type: "box", start: pointer, current: pointer, additive: e.shiftKey };
      }
    };

    const onPointerMove = (e: PointerEvent) => {
      pointer = toWorld(canvas, e.clientX, e.clientY);
      const rect = canvas.getBoundingClientRect();
      screenPointer = [e.clientX - rect.left, e.clientY - rect.top];
      setCursor(snap(pointer));

      if (drag?.type === "pan") {
        setView({
          ...useEditor.getState().view,
          x: drag.viewX + e.clientX - drag.startX,
          y: drag.viewY + e.clientY - drag.startY,
        });
      } else if (drag?.type === "move") {
        const p = snap(pointer);
        const delta: Point = [p[0] - drag.start[0], p[1] - drag.start[1]];
        if (delta[0] !== 0 || delta[1] !== 0 || drag.moved) {
          drag.moved = true;
          replace(moveSelection(drag.startDoc, drag.selection, delta));
        }
      } else if (drag?.type === "box") {
        drag.current = pointer;
      }
      updateHover();
      invalidate();
    };

    const onPointerUp = (e: PointerEvent) => {
      if (canvas.hasPointerCapture(e.pointerId)) canvas.releasePointerCapture(e.pointerId);
      if (heldButton) {
        pressButton(heldButton, false);
        heldButton = null;
        return;
      }
      const current = drag;
      drag = null;
      pointer = toWorld(canvas, e.clientX, e.clientY);
      const p = snap(pointer);

      if (current?.type === "move" && current.moved) {
        commit(useEditor.getState().doc, current.startDoc);
      } else if (current?.type === "box") {
        const { doc, selection } = useEditor.getState();
        const items = itemsInRect(doc, current.start, current.current);
        setSelection(
          current.additive
            ? {
                components: [...new Set([...selection.components, ...items.components])],
                wires: [...new Set([...selection.wires, ...items.wires])],
              }
            : items,
        );
      } else if (current?.type === "wire") {
        // Pinden sürükleyip bırakıldıysa tel bırakılan noktada biter.
        if (!samePoint(current.start, p))
          addDraftPoint(p, isConnectionPoint(useEditor.getState().doc, p));
      } else if (e.button === 0 && useEditor.getState().wireDraft && current === null) {
        addDraftPoint(p, false);
      }
      updateHover();
      invalidate();
    };

    const onDoubleClick = () => {
      const { wireDraft, doc } = useEditor.getState();
      if (wireDraft) {
        if (wireDraft.length > 1) addWirePath(wireDraft);
        setWireDraft(null);
        return;
      }
      const component = hitComponent(doc, pointer[0], pointer[1]);
      if (component && TOGGLE_PROP[component.kind] && useSim.getState().mode === "idle")
        flipToggle(component.id);
    };

    const onWheel = (e: WheelEvent) => {
      e.preventDefault();
      const { view } = useEditor.getState();
      const rect = canvas.getBoundingClientRect();
      const mx = e.clientX - rect.left;
      const my = e.clientY - rect.top;
      const zoom = Math.max(0.25, Math.min(5, view.zoom * Math.exp(-e.deltaY * 0.0015)));
      const k = zoom / view.zoom;
      setView({ zoom, x: mx - (mx - view.x) * k, y: my - (my - view.y) * k });
    };

    const onContextMenu = (e: MouseEvent) => e.preventDefault();
    const onLeave = () => {
      hover = { anchor: null, componentId: null, wireId: null };
      invalidate();
    };

    const onKeyDown = (e: KeyboardEvent) => {
      if (e.code === "Space" && !isTyping(e)) {
        spaceHeld = true;
        e.preventDefault();
      }
      if (e.key.toLowerCase() === "w" && !e.ctrlKey && !e.metaKey && !isTyping(e)) {
        const { wireDraft, cursor } = useEditor.getState();
        if (!wireDraft) setWireDraft([cursor]);
      }
    };
    const onKeyUp = (e: KeyboardEvent) => {
      if (e.code === "Space") spaceHeld = false;
    };

    // Kenar çubuğundan sürükle-bırak
    const onDragOver = (e: DragEvent) => {
      if (e.dataTransfer?.types.includes(DRAG_MIME)) {
        e.preventDefault();
        e.dataTransfer.dropEffect = "copy";
      }
    };
    const onDrop = (e: DragEvent) => {
      const kind = e.dataTransfer?.getData(DRAG_MIME) as ComponentKind | undefined;
      if (!kind) return;
      e.preventDefault();
      placeComponent(kind, snap(toWorld(canvas, e.clientX, e.clientY)));
      setTool({ type: "select" });
      canvas.focus();
    };

    canvas.addEventListener("pointerdown", onPointerDown);
    canvas.addEventListener("pointermove", onPointerMove);
    canvas.addEventListener("pointerup", onPointerUp);
    canvas.addEventListener("dblclick", onDoubleClick);
    canvas.addEventListener("wheel", onWheel, { passive: false });
    canvas.addEventListener("contextmenu", onContextMenu);
    canvas.addEventListener("pointerleave", onLeave);
    canvas.addEventListener("dragover", onDragOver);
    canvas.addEventListener("drop", onDrop);
    window.addEventListener("keydown", onKeyDown);
    window.addEventListener("keyup", onKeyUp);

    return () => {
      cancelAnimationFrame(raf);
      observer.disconnect();
      unsubEditor();
      unsubSim();
      canvas.removeEventListener("pointerdown", onPointerDown);
      canvas.removeEventListener("pointermove", onPointerMove);
      canvas.removeEventListener("pointerup", onPointerUp);
      canvas.removeEventListener("dblclick", onDoubleClick);
      canvas.removeEventListener("wheel", onWheel);
      canvas.removeEventListener("contextmenu", onContextMenu);
      canvas.removeEventListener("pointerleave", onLeave);
      canvas.removeEventListener("dragover", onDragOver);
      canvas.removeEventListener("drop", onDrop);
      window.removeEventListener("keydown", onKeyDown);
      window.removeEventListener("keyup", onKeyUp);
    };
  }, []);

  return (
    <main className="relative min-w-0 flex-1 overflow-hidden bg-canvas">
      <canvas ref={canvasRef} tabIndex={0} className="block h-full w-full outline-none" />
      {isEmpty && (
        <div className="pointer-events-none absolute inset-0 flex items-center justify-center">
          <div className="flex max-w-sm flex-col items-center gap-3 text-center">
            <div className="rounded-2xl p-3 shadow-[0_0_48px_var(--accent-glow)]">
              <Logo size={56} />
            </div>
            <h1 className="text-lg font-semibold">{t("canvas.emptyTitle")}</h1>
            <p className="leading-relaxed text-secondary">{t("canvas.emptyHint")}</p>
            <button
              type="button"
              onClick={openGallery}
              className="pointer-events-auto mt-1 rounded-md border border-accent/40 px-3 py-1.5 text-accent transition hover:bg-accent/10"
            >
              {t("canvas.browseExamples")}
            </button>
          </div>
        </div>
      )}
    </main>
  );
}
