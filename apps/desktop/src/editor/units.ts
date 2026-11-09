/**
 * SI önekli değerler. Ayrıştırma kuralları Rust tarafındaki
 * `devrem_core::units` ile aynıdır: `4k7`, `4,7k`, `100n`, `2.2µ`, `1M` (mega),
 * `1m` (mili), `1meg`, `1e-3`, `10kΩ`, `5V`, `50Hz`.
 */

const UNITS = ["ohm", "Hz", "Ω", "°", "F", "H", "V", "A", "s", "W"];

const PREFIXES: Record<string, number> = {
  f: 1e-15,
  p: 1e-12,
  n: 1e-9,
  u: 1e-6,
  µ: 1e-6,
  μ: 1e-6,
  m: 1e-3,
  k: 1e3,
  K: 1e3,
  M: 1e6,
  G: 1e9,
  T: 1e12,
};

const CORE = /^([+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?)(meg|[fpnuµμmkKMGT])?(\d*)$/;

function parseCore(s: string): number | null {
  const m = CORE.exec(s);
  if (!m) return null;
  const [, num = "", prefix, tail = ""] = m;
  const mult = prefix === undefined ? 1 : prefix === "meg" ? 1e6 : (PREFIXES[prefix] ?? 1);
  if (tail) {
    // 4k7 yazımı yalnızca önekten sonra ve ondalık nokta yokken geçerli
    if (!prefix || /[.eE]/.test(num)) return null;
    return Number(`${num}.${tail}`) * mult;
  }
  const v = Number(num) * mult;
  return Number.isFinite(v) ? v : null;
}

/** Değer metnini sayıya çevirir; geçersizse `null`. */
export function parseValue(text: string): number | null {
  const s = text.trim().replace(/,/g, ".").replace(/−/g, "-");
  if (!s) return null;
  const direct = parseCore(s);
  if (direct !== null) return direct;
  for (const unit of UNITS) {
    if (s.endsWith(unit)) {
      const v = parseCore(s.slice(0, -unit.length).trimEnd());
      if (v !== null) return v;
    }
  }
  return null;
}

const FORMAT_PREFIXES: [number, string][] = [
  [1e12, "T"],
  [1e9, "G"],
  [1e6, "M"],
  [1e3, "k"],
  [1, ""],
  [1e-3, "m"],
  [1e-6, "µ"],
  [1e-9, "n"],
  [1e-12, "p"],
  [1e-15, "f"],
];

/** Sayıyı SI önekiyle okunabilir biçime çevirir: `4700, "Ω"` → `"4.7 kΩ"`. */
export function formatValue(value: number, unit = "", digits = 3): string {
  if (!Number.isFinite(value)) return `— ${unit}`.trim();
  if (value === 0) return `0 ${unit}`.trim();
  const abs = Math.abs(value);
  for (let i = 0; i < FORMAT_PREFIXES.length; i++) {
    const [scale, prefix] = FORMAT_PREFIXES[i]!;
    if (abs >= scale * 0.9995 || i === FORMAT_PREFIXES.length - 1) {
      const mantissa = Number((value / scale).toPrecision(digits));
      if (Math.abs(mantissa) >= 1000 && i > 0) {
        const [upScale, upPrefix] = FORMAT_PREFIXES[i - 1]!;
        return `${Number((value / upScale).toPrecision(digits))} ${upPrefix}${unit}`;
      }
      return `${mantissa} ${prefix}${unit}`;
    }
  }
  return `${value} ${unit}`;
}
