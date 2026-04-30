interface ToggleProps {
  id: string;
  label: string;
  description?: string;
  checked: boolean;
  onChange: (value: boolean) => void;
  disabled?: boolean;
}

export default function Toggle({
  id,
  label,
  description,
  checked,
  onChange,
  disabled = false,
}: ToggleProps) {
  return (
    <div className="flex items-start justify-between gap-4">
      <div>
        <label
          htmlFor={id}
          className={`text-sm font-medium text-(--color-text-primary) ${disabled ? "opacity-40" : "cursor-pointer"}`}
        >
          {label}
        </label>
        {description && (
          <p className="text-xs text-(--color-text-muted)">{description}</p>
        )}
      </div>
      <button
        id={id}
        role="switch"
        aria-checked={checked}
        disabled={disabled}
        onClick={() => !disabled && onChange(!checked)}
        className={`relative inline-flex h-6 w-11 shrink-0 rounded-full border-2 border-transparent transition-colors duration-200 focus:outline-none disabled:opacity-40 disabled:cursor-not-allowed ${
          disabled ? "" : "cursor-pointer"
        } ${checked ? "bg-(--color-brand-500)" : "bg-(--color-surface-overlay)"}`}
      >
        <span
          className={`pointer-events-none inline-block h-5 w-5 rounded-full bg-white shadow transform transition-transform duration-200 ${
            checked ? "translate-x-5" : "translate-x-0"
          }`}
        />
      </button>
    </div>
  );
}
