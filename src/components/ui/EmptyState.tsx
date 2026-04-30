import type { ReactNode } from "react";

interface EmptyStateProps {
  icon?: string;
  heading: string;
  subtext?: string;
  action?: ReactNode;
}

export default function EmptyState({
  icon = "📭",
  heading,
  subtext,
  action,
}: EmptyStateProps) {
  return (
    <div className="flex flex-col items-center justify-center py-16 px-8 text-center">
      <span className="text-4xl mb-4 opacity-60">{icon}</span>
      <h3 className="text-base font-medium text-(--color-text-primary) mb-1">
        {heading}
      </h3>
      {subtext && (
        <p className="text-sm text-(--color-text-muted) max-w-xs mb-4">
          {subtext}
        </p>
      )}
      {action && <div className="mt-2">{action}</div>}
    </div>
  );
}
