import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { CATEGORIES, kindsInCategory } from "../editor/library";
import type { ComponentKind } from "../editor/types";
import { setTool, useEditor } from "../store/editor";
import { DRAG_MIME } from "./Canvas";
import { SymbolIcon } from "./SymbolIcon";

export function Sidebar() {
  const { t } = useTranslation();
  const [query, setQuery] = useState("");
  const [collapsed, setCollapsed] = useState<Set<string>>(new Set());
  const tool = useEditor((s) => s.tool);
  const activeKind = tool.type === "place" ? tool.kind : null;

  const groups = useMemo(() => {
    const q = query.trim().toLocaleLowerCase();
    return CATEGORIES.map((category) => ({
      category,
      kinds: kindsInCategory(category).filter(
        (k) => !q || t(`components.${k}`).toLocaleLowerCase().includes(q),
      ),
    })).filter((g) => !q || g.kinds.length > 0);
  }, [query, t]);

  const toggle = (category: string) =>
    setCollapsed((prev) => {
      const next = new Set(prev);
      if (next.has(category)) next.delete(category);
      else next.add(category);
      return next;
    });

  const pick = (kind: ComponentKind) =>
    setTool(activeKind === kind ? { type: "select" } : { type: "place", kind, rot: 0 });

  return (
    <aside className="flex w-60 shrink-0 flex-col border-r border-border bg-surface">
      <div className="px-3 pt-3 pb-2">
        <h2 className="mb-2 text-[11px] font-semibold tracking-wider text-secondary uppercase">
          {t("sidebar.title")}
        </h2>
        <input
          type="search"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder={t("sidebar.search")}
          className="w-full rounded-md border border-border bg-base px-2.5 py-1.5 text-primary placeholder:text-secondary/60 focus:border-accent focus:outline-none"
        />
      </div>
      <div className="flex-1 overflow-y-auto px-1.5 pb-3">
        {groups.map(({ category, kinds }) => {
          const open = !collapsed.has(category) || query.trim() !== "";
          const empty = kinds.length === 0;
          return (
            <section key={category} className="mb-0.5">
              <button
                type="button"
                onClick={() => !empty && toggle(category)}
                className={`flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left transition-colors ${
                  empty ? "cursor-default text-secondary/50" : "text-secondary hover:text-primary"
                }`}
              >
                <span
                  className={`text-[9px] text-wire transition-transform ${open && !empty ? "rotate-90" : ""}`}
                >
                  ▶
                </span>
                <span className="flex-1 font-medium">{t(`sidebar.categories.${category}`)}</span>
                {empty && (
                  <span className="rounded border border-border px-1.5 text-[10px] text-secondary/60">
                    {t("sidebar.soon")}
                  </span>
                )}
              </button>
              {open && !empty && (
                <ul className="mt-0.5 mb-1 grid gap-0.5">
                  {kinds.map((kind) => (
                    <li key={kind}>
                      <button
                        type="button"
                        draggable
                        onDragStart={(e) => {
                          e.dataTransfer.setData(DRAG_MIME, kind);
                          e.dataTransfer.effectAllowed = "copy";
                        }}
                        onClick={() => pick(kind)}
                        aria-pressed={activeKind === kind}
                        className={`flex w-full items-center gap-2.5 rounded-md py-1 pr-2 pl-3 text-left transition-colors ${
                          activeKind === kind
                            ? "bg-accent/10 text-accent ring-1 ring-accent/40"
                            : "text-primary/90 hover:bg-elevated"
                        }`}
                      >
                        <SymbolIcon
                          kind={kind}
                          color={activeKind === kind ? "#22d3ee" : "#b8c2d4"}
                        />
                        <span>{t(`components.${kind}`)}</span>
                      </button>
                    </li>
                  ))}
                </ul>
              )}
            </section>
          );
        })}
      </div>
    </aside>
  );
}
