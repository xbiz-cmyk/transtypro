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
      {/* Speech waveform — short / tall / medium bars */}
      <rect x="2"   y="10"  width="3" height="4"  rx="1.5" fill="var(--color-brand-300)" />
      <rect x="7"   y="4"   width="3" height="16" rx="1.5" fill="var(--color-brand-500)" />
      <rect x="12"  y="7"   width="3" height="10" rx="1.5" fill="var(--color-brand-400)" />
      {/* Text cursor — I-beam with serifs */}
      <rect x="18.5" y="4.5"   width="2"   height="15"  rx="1"     fill="var(--color-text-primary)" opacity="0.75" />
      <rect x="16.75" y="4.5"  width="5.5" height="1.5"  rx="0.75" fill="var(--color-text-primary)" opacity="0.75" />
      <rect x="16.75" y="18"   width="5.5" height="1.5"  rx="0.75" fill="var(--color-text-primary)" opacity="0.75" />
    </svg>
  );
}
