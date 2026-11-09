/** Uygulama penceresi çerçevesi içinde ekran görüntüsü. */
export function Screenshot({
  src,
  alt,
  priority = false,
  className = "",
}: {
  src: string;
  alt: string;
  priority?: boolean;
  className?: string;
}) {
  return (
    <figure
      className={`overflow-hidden rounded-xl border border-border bg-surface shadow-[0_30px_80px_-20px_rgba(0,0,0,0.8),0_0_60px_-30px_var(--accent)] ${className}`}
    >
      <div className="flex h-7 items-center gap-1.5 border-b border-border px-3">
        <span className="size-2.5 rounded-full bg-[#2a3142]" />
        <span className="size-2.5 rounded-full bg-[#2a3142]" />
        <span className="size-2.5 rounded-full bg-[#2a3142]" />
      </div>
      <img
        src={src}
        alt={alt}
        width={1280}
        height={800}
        loading={priority ? "eager" : "lazy"}
        className="block h-auto w-full"
      />
    </figure>
  );
}
