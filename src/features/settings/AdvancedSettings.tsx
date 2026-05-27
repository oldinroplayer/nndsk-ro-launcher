import { useEffect, useState } from 'react'
import { audioDriverLabel } from '../../shared/audio'
import { StatusDot, type DotStatus } from '../../shared/ui/StatusDot'
import { PrefixResetButton } from './PrefixResetButton'
import { useSettingsStore } from './settings.store'

function StatusLine({
  dotStatus,
  label,
  hint,
}: {
  dotStatus: DotStatus
  label: string
  hint?: string | null
}) {
  return (
    <div className="min-w-0" title={hint ?? undefined}>
      <div className="flex items-center gap-2 min-w-0">
        <StatusDot status={dotStatus} />
        <p className="text-[11px] text-zinc-400 truncate">{label}</p>
      </div>
      {hint && (
        <p className="text-[10px] text-zinc-500 leading-snug pl-4 truncate">{hint}</p>
      )}
    </div>
  )
}

export function AdvancedSettings() {
  const audioStatus = useSettingsStore((s) => s.audioStatus)
  const autopotInputStatus = useSettingsStore((s) => s.autopotInputStatus)
  const [open, setOpen] = useState(false)

  const audioDot: DotStatus =
    audioStatus && !audioStatus.audioOk
      ? 'error'
      : audioStatus?.audioWarning
        ? 'warning'
        : 'ok'

  const autopotWarn =
    autopotInputStatus != null && !autopotInputStatus.autopotInputOk

  const hasIssue = audioDot !== 'ok' || autopotWarn

  useEffect(() => {
    if (hasIssue) setOpen(true)
  }, [hasIssue])

  if (!audioStatus) return null

  const audioLabel = `Audio · ${audioDriverLabel(audioStatus.audioDriver)}${
    !audioStatus.audioOk ? ' (no disponible)' : ''
  }`

  return (
    <div
      className={`rounded-xl border shrink-0 overflow-hidden ${
        hasIssue && !open
          ? 'border-amber-500/25 bg-amber-500/5'
          : 'border-zinc-800/80 bg-zinc-900/40'
      }`}
    >
      <button
        type="button"
        onClick={() => setOpen((v) => !v)}
        className="w-full flex items-center justify-between gap-2 px-3 py-2 text-left hover:bg-zinc-800/30 transition-colors"
      >
        <span className="text-[10px] font-semibold text-zinc-500 uppercase tracking-[0.14em]">
          Avanzado
        </span>
        <span className="flex items-center gap-2">
          {hasIssue && !open && (
            <span className="text-[9px] px-1.5 py-0.5 rounded bg-amber-500/15 text-amber-400 border border-amber-500/25">
              !
            </span>
          )}
          <span className="text-[10px] text-zinc-600">{open ? '▾' : '▸'}</span>
        </span>
      </button>

      {open && (
        <div className="px-3 pb-3 space-y-2 border-t border-zinc-800/80 pt-2">
          <div className={`rounded-lg px-2 py-1.5 space-y-1 ${hasIssue ? 'bg-amber-500/5' : ''}`}>
            <StatusLine
              dotStatus={audioDot}
              label={audioLabel}
              hint={audioStatus.audioWarning}
            />
            {autopotWarn && (
              <StatusLine
                dotStatus="warning"
                label="AutoPot · input no disponible"
                hint={autopotInputStatus.autopotInputWarning}
              />
            )}
          </div>
          <PrefixResetButton />
        </div>
      )}
    </div>
  )
}
