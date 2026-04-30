import type { InputHTMLAttributes } from "react";

interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  helperText?: string;
  error?: string;
  id: string;
}

export default function Input({
  label,
  helperText,
  error,
  id,
  className = "",
  ...rest
}: InputProps) {
  return (
    <div className="flex flex-col gap-1">
      {label && (
        <label
          htmlFor={id}
          className="text-sm font-medium text-(--color-text-secondary)"
        >
          {label}
        </label>
      )}
      <input
        id={id}
        className={`bg-(--color-surface-base) border rounded-(--radius-btn) px-3 py-2 text-sm text-(--color-text-primary) placeholder:text-(--color-text-muted) focus:outline-none focus:ring-2 focus:ring-(--color-brand-500)/50 transition-colors ${
          error
            ? "border-(--color-status-error)"
            : "border-(--color-border-default) focus:border-(--color-brand-500)"
        } ${className}`}
        {...rest}
      />
      {error && (
        <p className="text-xs text-(--color-status-error)">{error}</p>
      )}
      {helperText && !error && (
        <p className="text-xs text-(--color-text-muted)">{helperText}</p>
      )}
    </div>
  );
}
