/** Izgara noktası `[x, y]` (ızgara birimi, y aşağı doğru artar). */
export type Point = [number, number];

export type ComponentKind =
  | "ground"
  | "resistor"
  | "capacitor"
  | "inductor"
  | "dcVoltage"
  | "dcCurrent"
  | "acVoltage"
  | "diode"
  | "led"
  | "switch"
  | "voltmeter"
  | "ammeter"
  | "npn"
  | "pnp"
  | "nmos"
  | "pmos"
  | "opAmp"
  | "and"
  | "or"
  | "not"
  | "nand"
  | "nor"
  | "xor"
  | "xnor"
  | "dFlipFlop"
  | "jkFlipFlop"
  | "tFlipFlop"
  | "logicInput"
  | "logicProbe"
  | "clock"
  | "potentiometer"
  | "zener"
  | "pushButton"
  | "spdtSwitch"
  | "lamp"
  | "pulseVoltage"
  | "timer555"
  | "sevenSegment"
  | "hexDecoder"
  | "transformer"
  | "optocoupler";

export type PropValue = string | number | boolean;

export interface CircuitComponent {
  id: string;
  kind: ComponentKind;
  pos: Point;
  /** 90° adımlarla saat yönünde dönüş (0–3). */
  rot: number;
  props: Record<string, PropValue>;
}

export interface Wire {
  id: string;
  a: Point;
  b: Point;
}

export interface CircuitDocument {
  formatVersion: number;
  meta: { name: string; createdWith: string };
  components: CircuitComponent[];
  wires: Wire[];
  simulation: SimulationSettings;
}

export interface AcSettings {
  start: number;
  stop: number;
  pointsPerDecade: number;
}

export interface DcSweepSettings {
  source: string;
  start: number;
  stop: number;
  steps: number;
}

export interface SimulationSettings {
  timeScale: number;
  ac: AcSettings;
  dcSweep: DcSweepSettings;
}

export const DEFAULT_SIMULATION: SimulationSettings = {
  timeScale: 1,
  ac: { start: 1, stop: 1e6, pointsPerDecade: 20 },
  dcSweep: { source: "", start: 0, stop: 5, steps: 100 },
};

export const FORMAT_VERSION = 1;

export function emptyDocument(name: string): CircuitDocument {
  return {
    formatVersion: FORMAT_VERSION,
    meta: { name, createdWith: "Devrem" },
    components: [],
    wires: [],
    simulation: structuredClone(DEFAULT_SIMULATION),
  };
}

/**
 * Eksik alanları varsayılanlarla tamamlar. Eski sürümlerle kaydedilmiş veya
 * doğrudan okunan (`examples/`) dosyalarda analiz ayarları bulunmayabilir.
 */
export function normalizeDocument(doc: CircuitDocument): CircuitDocument {
  const sim: Partial<SimulationSettings> = doc.simulation ?? {};
  return {
    ...doc,
    components: doc.components ?? [],
    wires: doc.wires ?? [],
    simulation: {
      timeScale: sim.timeScale ?? DEFAULT_SIMULATION.timeScale,
      ac: { ...DEFAULT_SIMULATION.ac, ...sim.ac },
      dcSweep: { ...DEFAULT_SIMULATION.dcSweep, ...sim.dcSweep },
    },
  };
}
