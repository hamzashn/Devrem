/** Arka plan için hareketli akım noktalı devre çizimi. */
export function CircuitMotif({ className = "" }: { className?: string }) {
  const wire = "#2a3346";
  const path = "M20 60 H180 V160 H340 V60 H500 M180 160 V260 H420 V160";
  return (
    <svg viewBox="0 0 520 300" className={className} aria-hidden>
      <path d={path} fill="none" stroke={wire} strokeWidth="3" />
      <path d={path} fill="none" stroke="#facc15" strokeWidth="6" className="current-flow" />
      {/* Direnç, kondansatör, LED */}
      <rect
        x="240"
        y="148"
        width="50"
        height="24"
        rx="3"
        fill="#0b0e14"
        stroke="#b8c2d4"
        strokeWidth="3"
      />
      <g stroke="#b8c2d4" strokeWidth="4">
        <line x1="335" y1="40" x2="335" y2="80" />
        <line x1="347" y1="40" x2="347" y2="80" />
      </g>
      <circle cx="420" cy="210" r="16" fill="#22d3ee" opacity="0.35" />
      <circle cx="180" cy="160" r="5" fill="#b8c2d4" />
      <circle cx="420" cy="160" r="5" fill="#b8c2d4" />
    </svg>
  );
}
