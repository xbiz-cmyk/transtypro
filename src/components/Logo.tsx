interface LogoProps {
  size?: number;
  className?: string;
}

export default function Logo({ size = 22, className }: LogoProps) {
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
      {/* Speech waveform — three bars of increasing height */}
      <rect x="1"   y="9"  width="2.5" height="6"  rx="1.25" fill="var(--color-brand-400)" />
      <rect x="5"   y="6"  width="2.5" height="12" rx="1.25" fill="var(--color-brand-400)" />
      <rect x="9"   y="4"  width="2.5" height="16" rx="1.25" fill="var(--color-brand-400)" />
      {/* Text cursor — vertical bar with serifs */}
      <rect x="14.5" y="5"    width="2"   height="14"  rx="1"    fill="var(--color-text-primary)" opacity="0.7" />
      <rect x="13"   y="5"    width="5"   height="1.5" rx="0.75" fill="var(--color-text-primary)" opacity="0.7" />
      <rect x="13"   y="17.5" width="5"   height="1.5" rx="0.75" fill="var(--color-text-primary)" opacity="0.7" />
    </svg>
  );
}
