interface LogoProps {
  size?: number;
  className?: string;
  color?: "brand" | "white";
}

export default function Logo({ size = 22, className, color = "brand" }: LogoProps) {
  const isBrand = color === "brand";

  const bar1Fill  = isBrand ? "var(--color-brand-300)" : "white";
  const bar2Fill  = isBrand ? "var(--color-brand-500)" : "white";
  const bar3Fill  = isBrand ? "var(--color-brand-400)" : "white";
  const cursorFill = isBrand ? "var(--color-text-primary)" : "white";

  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      className={className}
      aria-hidden="true"
    >
      {/* Speech waveform bars */}
      <rect x="2"  y="10" width="3" height="4"  rx="1.5" fill={bar1Fill}  opacity={isBrand ? 1 : 0.55} />
      <rect x="7"  y="4"  width="3" height="16" rx="1.5" fill={bar2Fill} />
      <rect x="12" y="7"  width="3" height="10" rx="1.5" fill={bar3Fill}  opacity={isBrand ? 1 : 0.80} />
      {/* Text cursor I-beam */}
      <rect x="18.5"  y="4.5"  width="2"   height="15"  rx="1"    fill={cursorFill} opacity="0.75" />
      <rect x="16.75" y="4.5"  width="5.5" height="1.5"  rx="0.75" fill={cursorFill} opacity="0.75" />
      <rect x="16.75" y="18"   width="5.5" height="1.5"  rx="0.75" fill={cursorFill} opacity="0.75" />
    </svg>
  );
}
