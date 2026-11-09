import { isTyping } from "./keyboard";
import {
  cancel,
  deleteSelected,
  redo,
  rotateSelected,
  selectAll,
  setTool,
  undo,
  useEditor,
} from "../store/editor";
import { toggleRunning } from "../store/sim";
import { newDocument, openDocument, saveDocument } from "./files";
import { fitToContent, zoomBy } from "./view";

/** Genel klavye kısayolları. Kurulumu kaldıran bir fonksiyon döner. */
export function installShortcuts(): () => void {
  const onKeyDown = (e: KeyboardEvent) => {
    if (isTyping(e)) return;
    const ctrl = e.ctrlKey || e.metaKey;
    const key = e.key.toLowerCase();
    const run = (fn: () => unknown) => {
      e.preventDefault();
      void fn();
    };

    if (ctrl) {
      if (key === "z" && e.shiftKey) return run(redo);
      if (key === "z") return run(undo);
      if (key === "y") return run(redo);
      if (key === "a") return run(selectAll);
      if (key === "s") return run(() => saveDocument(e.shiftKey));
      if (key === "o") return run(openDocument);
      if (key === "n") return run(newDocument);
      if (key === "=" || key === "+") return run(() => zoomBy(1.25));
      if (key === "-") return run(() => zoomBy(0.8));
      if (key === "0") return run(fitToContent);
      return;
    }

    if (key === "escape") return run(cancel);
    if (key === "delete" || key === "backspace") return run(deleteSelected);
    if (key === "f5") return run(toggleRunning);
    if (key === "r") {
      const { tool } = useEditor.getState();
      if (tool.type === "place") return run(() => setTool({ ...tool, rot: (tool.rot + 1) % 4 }));
      return run(rotateSelected);
    }
  };
  window.addEventListener("keydown", onKeyDown);
  return () => window.removeEventListener("keydown", onKeyDown);
}
