import { ask, open, save } from "@tauri-apps/plugin-dialog";
import i18n from "../i18n";
import { emptyDocument } from "../editor/types";
import { loadDocument, markSaved, useEditor } from "../store/editor";
import { stopSimulation } from "../store/sim";
import { showNotice } from "../store/notice";
import { isTauri, openDocumentFile, saveDocumentFile } from "./ipc";
import { describeError } from "./errors";
import type { Example } from "./examples";
import { fitToContent } from "./view";
import { closeGallery } from "../store/ui";

const FILTERS = [{ name: "Devrem", extensions: ["dvrm"] }];

function baseName(path: string): string {
  const file = path.split(/[\\/]/).pop() ?? path;
  return file.replace(/\.dvrm$/i, "");
}

/** Kaydedilmemiş değişiklik varsa kullanıcıya sorar. */
async function confirmDiscard(): Promise<boolean> {
  if (!useEditor.getState().dirty) return true;
  if (!isTauri()) return window.confirm(i18n.t("dialogs.discard"));
  return ask(i18n.t("dialogs.discard"), { title: "Devrem", kind: "warning" });
}

export async function newDocument() {
  if (!(await confirmDiscard())) return;
  await stopSimulation();
  loadDocument(emptyDocument(i18n.t("app.untitled")), null);
}

/** Galeriden bir örneği yeni (kaydedilmemiş) belge olarak açar. */
export async function openExample(example: Example) {
  if (!(await confirmDiscard())) return;
  await stopSimulation();
  loadDocument(structuredClone(example.doc), null);
  closeGallery();
  requestAnimationFrame(() => fitToContent());
}

export async function openDocument() {
  if (!isTauri() || !(await confirmDiscard())) return;
  const path = await open({ filters: FILTERS, multiple: false, directory: false });
  if (!path) return;
  try {
    const doc = await openDocumentFile(path);
    await stopSimulation();
    loadDocument(doc, path);
  } catch (e) {
    showNotice("error", describeError(e));
  }
}

/** Belgeyi kaydeder; başarılıysa `true` döner. */
export async function saveDocument(saveAs = false): Promise<boolean> {
  if (!isTauri()) return false;
  const { doc, filePath } = useEditor.getState();
  let path = filePath;
  if (!path || saveAs) {
    path = await save({ filters: FILTERS, defaultPath: `${doc.meta.name}.dvrm` });
    if (!path) return false;
    if (!/\.dvrm$/i.test(path)) path += ".dvrm";
  }
  const name = baseName(path);
  try {
    await saveDocumentFile(path, { ...doc, meta: { ...doc.meta, name } });
    markSaved(path, name);
    showNotice("info", i18n.t("notices.saved", { name }));
    return true;
  } catch (e) {
    showNotice("error", describeError(e));
    return false;
  }
}
