/** Uygulamaya gömülü örnek devreler (`examples/*.dvrm`). */
import type { CircuitDocument } from "../editor/types";

const files = import.meta.glob("../../../../examples/*.dvrm", {
  query: "?raw",
  import: "default",
  eager: true,
}) as Record<string, string>;

export interface Example {
  /** Dosya adı (uzantısız), çeviri anahtarı olarak da kullanılır. */
  id: string;
  doc: CircuitDocument;
}

/** Galerideki sıralama; listede olmayanlar sona eklenir. */
const ORDER = [
  "led",
  "gerilim-bolucu",
  "rc-sarj",
  "ac-led",
  "isik-ayari",
  "transformator",
  "transistor-anahtar",
  "zener-regulator",
  "optokuplor",
  "evirici-yukselteci",
  "rc-alcak-geciren",
  "555-flasor",
  "sayici",
  "hex-sayac",
];

export const EXAMPLES: Example[] = Object.entries(files)
  .map(([path, text]) => ({
    id: path
      .split("/")
      .pop()!
      .replace(/\.dvrm$/, ""),
    doc: JSON.parse(text) as CircuitDocument,
  }))
  .sort((a, b) => {
    const ia = ORDER.indexOf(a.id);
    const ib = ORDER.indexOf(b.id);
    return (ia < 0 ? 999 : ia) - (ib < 0 ? 999 : ib) || a.id.localeCompare(b.id);
  });
