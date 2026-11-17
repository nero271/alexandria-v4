import type { CSSProperties } from "react";

/** Logo: o número 3 em algarismos romanos (III), três "I" com serifas. */
const CENTERS = [20, 60, 100];
// Atraso (em segundos) de cada "I": o do meio nasce primeiro, depois os das pontas.
const DELAY = [0.6, 0.35, 0.6];

export function Logo({ className }: { className?: string }) {
  return (
    <svg className={`logo${className ? ` ${className}` : ""}`} viewBox="0 0 120 100" role="img" aria-label="ALEXANDRIA" fill="currentColor">
      {CENTERS.map((cx, i) => (
        <g key={cx} style={{ "--d": DELAY[i] } as CSSProperties}>
          <rect className="stem" x={cx - 5} y={12} width={10} height={76} />
          <rect className="serif" x={cx - 15} y={6} width={30} height={6} />
          <rect className="serif" x={cx - 15} y={88} width={30} height={6} />
        </g>
      ))}
    </svg>
  );
}
