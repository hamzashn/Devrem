import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import tr from "../../../../locales/tr/common.json";
import en from "../../../../locales/en/common.json";

export const LANGUAGES = ["tr", "en"] as const;
export type Language = (typeof LANGUAGES)[number];

export function detectLanguage(): Language {
  return navigator.language.toLowerCase().startsWith("tr") ? "tr" : "en";
}

void i18n.use(initReactI18next).init({
  resources: { tr: { common: tr }, en: { common: en } },
  lng: detectLanguage(),
  fallbackLng: "en",
  defaultNS: "common",
  interpolation: { escapeValue: false },
});

export default i18n;
