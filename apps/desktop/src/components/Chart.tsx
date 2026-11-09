import { useEffect, useRef } from "react";
import uPlot from "uplot";
import "uplot/dist/uPlot.min.css";

interface ChartProps {
  /** Genişlik ve yükseklik hariç uPlot ayarları; değiştiğinde grafik yeniden kurulur. */
  options: Omit<uPlot.Options, "width" | "height">;
  /** Verilmezse grafik `onReady` ile dışarıdan güncellenir. */
  data?: uPlot.AlignedData;
  /** Grafiği dışarıdan güncellemek için (ör. osiloskop). */
  onReady?: (plot: uPlot | null) => void;
}

/** Kapsayıcısını dolduran uPlot grafiği. */
export function Chart({ options, data, onReady }: ChartProps) {
  const ref = useRef<HTMLDivElement>(null);
  const plotRef = useRef<uPlot | null>(null);

  useEffect(() => {
    const el = ref.current!;
    const size = () => ({
      width: Math.max(100, el.clientWidth),
      height: Math.max(80, el.clientHeight - 28),
    });
    const empty = options.series.map(() => []) as unknown as uPlot.AlignedData;
    const plot = new uPlot({ ...options, ...size() }, empty, el);
    plotRef.current = plot;
    onReady?.(plot);
    const observer = new ResizeObserver(() => plot.setSize(size()));
    observer.observe(el);
    return () => {
      observer.disconnect();
      onReady?.(null);
      plot.destroy();
      plotRef.current = null;
    };
  }, [options, onReady]);

  useEffect(() => {
    if (data) plotRef.current?.setData(data);
  }, [data, options]);

  return <div ref={ref} className="devrem-chart h-full w-full overflow-hidden" />;
}
