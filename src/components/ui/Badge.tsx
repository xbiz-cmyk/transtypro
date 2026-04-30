import type { ReactNode } from "react";

type BadgeVariant = "default" | "success" | "warning" | "danger" | "muted";

interface BadgeProps {
  variant?: BadgeVariant;
  children: ReactNode;
  className?: string;
}

const variantClasses: Record<BadgeVariant, string> = {
  default: "bg-(--color-brand-500)/20 text-(--color-brand-300)",
  success: "bg-(--color-status-success)/20 text-(--color-status-success)",
  warning: "bg-(--color-status-warning)/20 text-(--color-status-warning)",
  danger: "bg-(--color-status-error)/20 text-(--color-status-error)",
  muted: "bg-(--color-surface-overlay) text-(--color-text-muted)",
};

export default function Badge({
  variant = "default",
  children,
  className = "",
}: BadgeProps) {
  return (
    <span
      className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${variantClasses[variant]} ${className}`}
    >
      {children}
    </span>
  );
}
