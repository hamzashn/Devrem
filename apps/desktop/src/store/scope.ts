/**
 * Osiloskop tamponu. Her karede gelen örnekler buraya eklenir; grafik
 * React'i yeniden çizdirmeden doğrudan dinleyicilerle güncellenir.
 */
import type { Probe, Samples } from "../lib/ipc";

/** Tamponda tutulan en fazla örnek. */
const MAX_POINTS = 20_000;

export interface ScopeBuffer {
  probes: Probe[];
  time: number[];
  values: number[][];
}

export const scope: ScopeBuffer = { probes: [], time: [], values: [] };

const listeners = new Set<() => void>();

export function subscribeScope(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

function notify() {
  for (const l of listeners) l();
}

/** Tamponu yeni bir ölçüm listesiyle sıfırlar. */
export function resetScope(probes: Probe[]) {
  scope.probes = probes;
  scope.time = [];
  scope.values = probes.map(() => []);
  notify();
}

export function appendScope(samples: Samples) {
  if (!samples.time.length || samples.values.length !== scope.values.length) return;
  scope.time.push(...samples.time);
  samples.values.forEach((v, i) => scope.values[i]!.push(...v));
  const excess = scope.time.length - MAX_POINTS;
  if (excess > 0) {
    scope.time.splice(0, excess);
    for (const v of scope.values) v.splice(0, excess);
  }
  notify();
}
