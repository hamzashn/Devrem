import { useState } from "react";
import { formatValue, parseValue } from "../editor/units";

interface SiInputProps {
  label: string;
  value: number;
  unit?: string;
  /** Yalnızca tam sayı kabul et. */
  integer?: boolean;
  onCommit: (value: number) => void;
}

/** SI önekli sayı girişi (ör. `1k`, `100n`); Enter veya odak kaybında uygulanır. */
export function SiInput(props: SiInputProps) {
  // Dışarıdan değer değişirse alan yeniden oluşturulur.
  return <SiInputInner key={props.value} {...props} />;
}

function SiInputInner({ label, value, unit = "", integer, onCommit }: SiInputProps) {
  const initial = integer ? String(value) : formatValue(value, "", 4).replace(" ", "");
  const [text, setText] = useState(initial);
  const parsed = parseValue(text);
  const valid = parsed !== null && (!integer || (Number.isInteger(parsed) && parsed > 0));

  const apply = () => {
    if (valid && parsed !== value) onCommit(parsed!);
    else if (!valid) setText(initial);
  };

  return (
    <label className="flex flex-col gap-1">
      <span className="text-[10px] font-medium tracking-wide text-secondary uppercase">
        {label}
      </span>
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
          }}
          spellCheck={false}
          className="w-20 min-w-0 flex-1 bg-transparent px-2 py-1 font-mono text-[12px] text-primary outline-none"
        />
        {unit && <span className="font-mono text-[11px] text-secondary">{unit}</span>}
      </div>
    </label>
  );
}
