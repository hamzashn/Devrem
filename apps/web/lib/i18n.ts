import tr from "@/dictionaries/tr.json";
import en from "@/dictionaries/en.json";

export const LOCALES = ["tr", "en"] as const;
export type Locale = (typeof LOCALES)[number];
export type Dictionary = typeof tr;

const DICTIONARIES: Record<Locale, Dictionary> = { tr, en };

export function isLocale(value: string): value is Locale {
  return (LOCALES as readonly string[]).includes(value);
}

export function getDictionary(locale: string): Dictionary {
  return DICTIONARIES[isLocale(locale) ? locale : "tr"];
}

/** `{{name}}` yer tutucularını doldurur. */
export function fill(text: string, values: Record<string, string>): string {
  return text.replace(/\{\{(\w+)\}\}/g, (_, key: string) => values[key] ?? "");
}
