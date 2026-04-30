interface ErrorMessageProps {
  message: string;
  className?: string;
}

export default function ErrorMessage({
  message,
  className = "",
}: ErrorMessageProps) {
  return (
    <div
      className={`flex items-start gap-3 bg-(--color-status-error)/10 border border-(--color-status-error)/30 rounded-(--radius-card) p-4 ${className}`}
    >
      <span className="text-base mt-0.5">⚠️</span>
      <p className="text-sm text-(--color-status-error) flex-1">{message}</p>
    </div>
  );
}
