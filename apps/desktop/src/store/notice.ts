import { create } from "zustand";

export interface Notice {
  kind: "info" | "error";
  text: string;
  id: number;
}

/** Durum çubuğunda kısa süre görünen bildirim. */
export const useNotice = create<{ notice: Notice | null }>()(() => ({ notice: null }));

let counter = 0;
let timer: ReturnType<typeof setTimeout> | undefined;

export function showNotice(kind: Notice["kind"], text: string) {
  const id = ++counter;
  useNotice.setState({ notice: { kind, text, id } });
  clearTimeout(timer);
  timer = setTimeout(
    () => {
      if (useNotice.getState().notice?.id === id) useNotice.setState({ notice: null });
    },
    kind === "error" ? 8000 : 3000,
  );
}
