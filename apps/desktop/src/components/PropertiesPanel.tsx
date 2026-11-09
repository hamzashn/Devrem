import { useState } from "react";
import { useTranslation } from "react-i18next";
import { PROP_META, LED_COLORS } from "../editor/library";
import type { CircuitComponent } from "../editor/types";
import { formatValue, parseValue } from "../editor/units";
import { deleteSelected, rotateSelected, setProp, useEditor } from "../store/editor";
import { flipToggle, setPotentiometer } from "../store/sim";

const SHORTCUTS = [
  ["R", "rotate"],
  ["W", "wire"],
  ["Del", "delete"],
  ["Esc", "cancel"],
  ["Ctrl+Z", "undo"],
  ["F5", "run"],
  ["Space", "pan"],
] as const;

export function PropertiesPanel() {
  const { t } = useTranslation();
  const selection = useEditor((s) => s.selection);
  const doc = useEditor((s) => s.doc);
  const count = selection.components.length + selection.wires.length;
  const component =
    selection.components.length === 1 && selection.wires.length === 0
      ? doc.components.find((c) => c.id === selection.components[0])
      : undefined;

  return (
    <aside className="flex w-64 shrink-0 flex-col border-l border-border bg-surface">
      <h2 className="px-3 pt-3 pb-2 text-[11px] font-semibold tracking-wider text-secondary uppercase">
        {t("properties.title")}
      </h2>
      <div className="flex-1 overflow-y-auto px-3 pb-3">
        {component ? (
          <ComponentProperties key={component.id} component={component} />
        ) : count > 0 ? (
          <div className="space-y-3">
            <p className="text-primary">
              {selection.wires.length === 1 && count === 1
                ? t("properties.wire", { id: selection.wires[0] })
                : t("properties.multiple", { count })}
            </p>
            <DeleteButton />
          </div>
        ) : (
          <div className="space-y-4">
            <p className="text-secondary">{t("properties.empty")}</p>
            <div>
              <h3 className="mb-2 text-[11px] font-semibold tracking-wider text-secondary uppercase">
                {t("properties.shortcuts")}
              </h3>
              <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1.5">
                {SHORTCUTS.map(([key, action]) => (
                  <div key={key} className="contents">
                    <dt>
                      <kbd className="rounded border border-border bg-base px-1.5 py-0.5 font-mono text-[10px] text-primary">
                        {key}
                      </kbd>
                    </dt>
                    <dd className="text-secondary">{t(`shortcuts.${action}`)}</dd>
                  </div>
                ))}
              </dl>
            </div>
          </div>
        )}
      </div>
    </aside>
  );
}

function ComponentProperties({ component }: { component: CircuitComponent }) {
  const { t } = useTranslation();
  const keys = Object.keys(component.props).filter((k) => PROP_META[k]);

  return (
    <div className="space-y-4">
      <div>
        <div className="text-[15px] font-semibold">{t(`components.${component.kind}`)}</div>
        <div className="font-mono text-[11px] text-secondary">{component.id}</div>
      </div>

      {keys.length > 0 && (
        <div className="space-y-3">
          {keys.map((key) => (
            <PropField key={key} component={component} prop={key} />
          ))}
        </div>
      )}

      <div className="flex gap-2">
        <button
          type="button"
          onClick={rotateSelected}
          className="flex-1 rounded-md border border-border px-2 py-1.5 text-secondary transition-colors hover:bg-elevated hover:text-primary"
        >
          ⟳ {t("properties.rotate")}
        </button>
        <DeleteButton />
      </div>
    </div>
  );
}

function DeleteButton() {
  const { t } = useTranslation();
  return (
    <button
      type="button"
      onClick={deleteSelected}
      className="flex-1 rounded-md border border-border px-2 py-1.5 text-secondary transition-colors hover:border-error/50 hover:bg-error/10 hover:text-error"
    >
      {t("properties.delete")}
    </button>
  );
}

function PropField({ component, prop }: { component: CircuitComponent; prop: string }) {
  const { t } = useTranslation();
  const meta = PROP_META[prop]!;
  const value = component.props[prop];
  const label = t(`props.${prop}`, prop);
  const labelClass = "mb-1 block text-[11px] font-medium text-secondary";

  if (meta.type === "bool") {
    const on = Boolean(value);
    return (
      <div>
        <span className={labelClass}>{label}</span>
        <button
          type="button"
          role="switch"
          aria-checked={on}
          onClick={() => flipToggle(component.id)}
          className={`flex w-full items-center justify-between rounded-md border px-2.5 py-1.5 transition-colors ${
            on ? "border-success/50 bg-success/10 text-success" : "border-border text-secondary"
          }`}
        >
          {prop === "high"
            ? t(on ? "levels.high" : "levels.low")
            : prop === "toB"
              ? t(on ? "props.atB" : "props.toA")
              : t(on ? "props.on" : "props.off")}
          <span
            className={`relative h-4 w-7 rounded-full transition-colors ${on ? "bg-success" : "bg-border"}`}
          >
            <span
              className={`absolute top-0.5 size-3 rounded-full bg-primary transition-all ${on ? "left-3.5" : "left-0.5"}`}
            />
          </span>
        </button>
      </div>
    );
  }

  if (meta.type === "slider") {
    const n = typeof value === "number" ? value : Number(value ?? 50);
    return (
      <label className="block">
        <span className={labelClass}>
          {label}
          <span className="float-right font-mono text-primary">
            {n}
            {meta.unit}
          </span>
        </span>
        <input
          type="range"
          min={meta.min}
          max={meta.max}
          step={1}
          value={n}
          onChange={(e) => setPotentiometer(component.id, Number(e.target.value))}
          className="w-full accent-[var(--accent)]"
        />
      </label>
    );
  }

  if (meta.type === "select") {
    return (
      <label className="block">
        <span className={labelClass}>{label}</span>
        <div className="flex items-center gap-2">
          <span
            className="size-3 shrink-0 rounded-full"
            style={{
              background: LED_COLORS[String(value)],
              boxShadow: `0 0 8px ${LED_COLORS[String(value)]}`,
            }}
          />
          <select
            value={String(value)}
            onChange={(e) => setProp(component.id, prop, e.target.value)}
            className="w-full rounded-md border border-border bg-base px-2 py-1.5 text-primary focus:border-accent focus:outline-none"
          >
            {meta.options.map((o) => (
              <option key={o} value={o}>
                {t(`colors.${o}`, o)}
              </option>
            ))}
          </select>
        </div>
      </label>
    );
  }

  // Değer dışarıdan değişirse (geri al vb.) alan yeniden oluşturulur.
  return (
    <ValueField
      key={String(value)}
      component={component}
      prop={prop}
      label={label}
      unit={meta.unit}
    />
  );
}

function ValueField({
  component,
  prop,
  label,
  unit,
}: {
  component: CircuitComponent;
  prop: string;
  label: string;
  unit: string;
}) {
  const { t } = useTranslation();
  const stored = String(component.props[prop] ?? "");
  const [text, setText] = useState(stored);

  const parsed = parseValue(text);
  const valid = parsed !== null;

  const apply = () => {
    if (valid && text.trim() !== stored) setProp(component.id, prop, text.trim());
    else if (!valid) setText(stored);
  };

  return (
    <label className="block">
      <span className="mb-1 block text-[11px] font-medium text-secondary">{label}</span>
      <div
        className={`flex items-center rounded-md border bg-base pr-2 focus-within:border-accent ${
          valid ? "border-border" : "border-error"
        }`}
      >
        <input
          value={text}
          onChange={(e) => setText(e.target.value)}
          onBlur={apply}
          onKeyDown={(e) => {
            if (e.key === "Enter") e.currentTarget.blur();
            if (e.key === "Escape") {
              setText(stored);
              e.currentTarget.blur();
            }
          }}
          spellCheck={false}
          className="min-w-0 flex-1 bg-transparent px-2.5 py-1.5 font-mono text-primary outline-none"
        />
        <span className="font-mono text-secondary">{unit}</span>
      </div>
      <span className={`mt-1 block text-[11px] ${valid ? "text-secondary" : "text-error"}`}>
        {valid ? `= ${formatValue(parsed, unit, 4)}` : t("properties.invalidValue")}
      </span>
    </label>
  );
}
