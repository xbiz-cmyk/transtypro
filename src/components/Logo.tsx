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
      {/* Speech waveform — short / tall / medium bars, left to right */}
      <rect x="2.5"  y="10"   width="2.5" height="4"  rx="1.25" fill="var(--color-brand-300)" opacity="0.55" />
      <rect x="6.5"  y="4.5"  width="2.5" height="15" rx="1.25" fill="var(--color-brand-400)" />
      <rect x="10.5" y="7"    width="2.5" height="10" rx="1.25" fill="var(--color-brand-400)" opacity="0.75" />
      {/* Text cursor — stem + top/bottom serifs */}
      <rect x="16"    y="5"     width="1.5" height="14"  rx="0.75"  fill="var(--color-text-primary)" opacity="0.8" />
      <rect x="14.75" y="5"     width="4"   height="1.25" rx="0.625" fill="var(--color-text-primary)" opacity="0.8" />
      <rect x="14.75" y="17.75" width="4"   height="1.25" rx="0.625" fill="var(--color-text-primary)" opacity="0.8" />
    </svg>
  );
}
