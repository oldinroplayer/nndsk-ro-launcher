type DotStatus = 'ok' | 'error' | 'neutral'

export function StatusDot({ status }: { status: DotStatus }) {
  const classes: Record<DotStatus, string> = {
    ok: 'bg-emerald-500 shadow-[0_0_6px_rgba(16,185,129,0.5)]',
    error: 'bg-red-500 shadow-[0_0_6px_rgba(239,68,68,0.5)]',
    neutral: 'bg-zinc-600',
  }

  return (
    <span className={`inline-block w-2 h-2 rounded-full shrink-0 ${classes[status]}`} aria-hidden />
  )
}

interface ToolRowProps {
  label: string
  dotStatus: DotStatus
  detail?: string | null
  warning?: string | null
  onAction?: () => void
  actionLabel?: string
  actionBusy?: boolean
  actionDisabled?: boolean
  onSecondary?: () => void
  secondaryLabel?: string
  secondaryBusy?: boolean
  secondaryDanger?: boolean
}

export function ToolRow({
  label,
  dotStatus,
  detail,
  warning,
  onAction,
  actionLabel,
  actionBusy,
  actionDisabled,
  onSecondary,
  secondaryLabel,
  secondaryBusy,
  secondaryDanger,
}: ToolRowProps) {
  const actionClass =
    'text-xs px-2.5 py-1 rounded-md border border-zinc-700/80 text-zinc-300 hover:border-amber-500/50 hover:text-amber-400 hover:bg-amber-500/5 transition-colors shrink-0 disabled:opacity-40 disabled:cursor-not-allowed disabled:hover:border-zinc-700 disabled:hover:text-zinc-300 disabled:hover:bg-transparent'

  const secondaryClass = secondaryDanger
    ? `${actionClass} hover:border-red-500/50 hover:text-red-400 hover:bg-red-500/5`
    : actionClass

  return (
    <div className="flex flex-col gap-1 py-2.5 border-b border-zinc-800/60 last:border-0">
      <div className="flex items-center gap-2.5 min-w-0">
        <StatusDot status={dotStatus} />
        <span className="text-sm text-zinc-200 shrink-0 w-20">{label}</span>
        {detail && (
          <span className="text-xs text-zinc-500 truncate flex-1 font-mono" title={detail}>
            {detail}
          </span>
        )}
        <div className="flex items-center gap-1.5 shrink-0">
          {onSecondary && secondaryLabel && (
            <button
              type="button"
              onClick={onSecondary}
              disabled={secondaryBusy}
              className={secondaryClass}
            >
              {secondaryBusy ? `${secondaryLabel}...` : secondaryLabel}
            </button>
          )}
          {onAction && actionLabel && (
            <button
              type="button"
              onClick={onAction}
              disabled={actionDisabled || actionBusy}
              className={actionClass}
            >
              {actionBusy ? `${actionLabel}...` : actionLabel}
            </button>
          )}
        </div>
      </div>
      {warning && (
        <p className="text-xs text-amber-400/90 pl-[18px] leading-relaxed">{warning}</p>
      )}
    </div>
  )
}
