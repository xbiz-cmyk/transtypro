import type { ReactNode } from "react";

interface CardProps {
  children: ReactNode;
  className?: string;
  id?: string;
}

export default function Card({ children, className = "", id }: CardProps) {
  return (
    <div
      id={id}
      className={`bg-(--color-surface-raised) border border-(--color-border-default) rounded-(--radius-card) p-5 ${className}`}
    >
      {children}
    </div>
  );
}

interface CardHeaderProps {
  children: ReactNode;
  className?: string;
}

export function CardHeader({ children, className = "" }: CardHeaderProps) {
  return (
    <h2
      className={`text-sm font-medium text-(--color-text-secondary) uppercase tracking-wider mb-4 ${className}`}
    >
      {children}
    </h2>
  );
}
