import { useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { EXAMPLES, type Example } from "../lib/examples";
import { openExample } from "../lib/files";
import { drawThumbnail } from "../lib/thumbnail";
import { closeGallery, useUi } from "../store/ui";

/** Örnek devre galerisi (modal). */
export function Gallery() {
  const { t } = useTranslation();
  const open = useUi((s) => s.galleryOpen);

  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.stopPropagation();
        closeGallery();
      }
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  }, [open]);

  if (!open) return null;
  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-8 backdrop-blur-sm"
      onPointerDown={(e) => e.target === e.currentTarget && closeGallery()}
    >
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby="gallery-title"
        className="flex max-h-full w-full max-w-5xl flex-col rounded-xl border border-border bg-surface shadow-2xl shadow-black/60"
      >
        <header className="flex items-center gap-3 border-b border-border px-5 py-3.5">
          <h2 id="gallery-title" className="text-[15px] font-semibold">
            {t("gallery.title")}
          </h2>
          <span className="text-secondary">{t("gallery.subtitle")}</span>
          <button
            type="button"
            onClick={closeGallery}
            aria-label={t("gallery.close")}
            className="ml-auto rounded-md px-2 py-1 text-secondary hover:bg-elevated hover:text-primary"
          >
            ✕
          </button>
        </header>
        <div className="grid grid-cols-[repeat(auto-fill,minmax(220px,1fr))] gap-3 overflow-y-auto p-5">
          {EXAMPLES.map((example) => (
            <ExampleCard key={example.id} example={example} />
          ))}
        </div>
      </div>
    </div>
  );
}

function ExampleCard({ example }: { example: Example }) {
  const { t, i18n } = useTranslation();
  const ref = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    if (ref.current) drawThumbnail(ref.current, example.doc);
  }, [example]);

  const key = `examples.${example.id}`;
  const title = i18n.exists(`${key}.title`) ? t(`${key}.title` as never) : example.doc.meta.name;
  const description = i18n.exists(`${key}.description`) ? t(`${key}.description` as never) : "";

  return (
    <button
      type="button"
      onClick={() => void openExample(example)}
      className="group flex flex-col overflow-hidden rounded-lg border border-border bg-base text-left transition hover:border-accent/60 hover:shadow-[0_0_24px_var(--accent-glow)]"
    >
      <canvas ref={ref} className="h-32 w-full bg-canvas" aria-hidden />
      <div className="flex flex-1 flex-col gap-1 border-t border-border px-3 py-2.5">
        <span className="font-medium text-primary group-hover:text-accent">{String(title)}</span>
        {description && (
          <span className="text-[11px] leading-snug text-secondary">{String(description)}</span>
        )}
      </div>
    </button>
  );
}
