import { create } from "zustand";
import { createJSONStorage, persist } from "zustand/middleware";
import i18n, { detectLanguage, type Language } from "../i18n";

export type PanelTab = "scope" | "ac" | "dcSweep";

interface SettingsState {
  language: Language;
  setLanguage: (language: Language) => void;
  panelOpen: boolean;
  panelTab: PanelTab;
  /** Osiloskop penceresi (s); 0 = otomatik. */
  scopeWindow: number;
}

/** Kullanıcı tercihleri; tarayıcı depolamasında saklanır. */
export const useSettings = create<SettingsState>()(
  persist(
    (set) => ({
      language: detectLanguage(),
      setLanguage: (language) => {
        void i18n.changeLanguage(language);
        set({ language });
      },
      panelOpen: true,
      panelTab: "scope",
      scopeWindow: 0,
    }),
    {
      name: "devrem.settings",
      storage: createJSONStorage(() => localStorage),
      onRehydrateStorage: () => (state) => {
        if (state) void i18n.changeLanguage(state.language);
      },
    },
  ),
);
