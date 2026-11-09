import i18n from "../i18n";
import { isAppError } from "./ipc";

/** Rust'tan gelen hatayı kullanıcının dilinde açıklar. */
export function describeError(error: unknown): string {
  if (!isAppError(error)) return String(error);
  const key = `errors.${error.code}`;
  const params: Record<string, unknown> = { ...error };
  if (typeof error.prop === "string") params.prop = i18n.t(`props.${error.prop}`, error.prop);
  return i18n.exists(key)
    ? String(i18n.t(key as never, params))
    : String(error.message ?? error.code);
}
