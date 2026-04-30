import type { ButtonHTMLAttributes, ReactNode } from "react";

type Variant = "primary" | "secondary" | "ghost" | "danger";
type Size = "sm" | "md" | "lg";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant;
  size?: Size;
  children: ReactNode;
}

const variantClasses: Record<Variant, string> = {
  primary:
    "bg-(--color-brand-500) hover:bg-(--color-brand-400) text-(--color-text-primary) border-transparent",
  secondary:
    "bg-(--color-surface-overlay) hover:bg-(--color-surface-raised) text-(--color-text-primary) border-(--color-border-default)",
  ghost:
    "bg-transparent hover:bg-(--color-surface-raised) text-(--color-text-secondary) hover:text-(--color-text-primary) border-transparent",
  danger:
    "bg-(--color-status-error)/20 hover:bg-(--color-status-error)/30 text-(--color-status-error) border-(--color-status-error)/30",
};

const sizeClasses: Record<Size, string> = {
  sm: "px-3 py-1.5 text-xs",
  md: "px-4 py-2 text-sm",
  lg: "px-5 py-2.5 text-base",
};

export default function Button({
  variant = "secondary",
  size = "md",
  className = "",
  disabled,
  children,
  ...rest
}: ButtonProps) {
  return (
    <button
      className={`inline-flex items-center justify-center gap-2 rounded-(--radius-btn) border font-medium transition-colors duration-150 cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed ${variantClasses[variant]} ${sizeClasses[size]} ${className}`}
      disabled={disabled}
      {...rest}
    >
      {children}
    </button>
  );
}
