type DotStatus = 'ok' | 'warning' | 'error' | 'neutral'

const dotClasses: Record<DotStatus, string> = {
  ok: 'bg-emerald-500 shadow-[0_0_6px_rgba(16,185,129,0.5)]',
  warning: 'bg-amber-500 shadow-[0_0_6px_rgba(245,158,11,0.5)]',
  error: 'bg-red-500 shadow-[0_0_6px_rgba(239,68,68,0.5)]',
  neutral: 'bg-zinc-600',
}

export function StatusDot({
  status,
  pulse = false,
}: {
  status: DotStatus
  pulse?: boolean
}) {
  return (
    <span
      className={`inline-block w-2 h-2 rounded-full shrink-0 ${dotClasses[status]} ${
        pulse ? 'animate-pulse-dot' : ''
      }`}
      aria-hidden
    />
  )
}

export type { DotStatus }
