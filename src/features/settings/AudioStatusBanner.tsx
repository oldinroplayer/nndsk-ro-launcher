import { audioDriverLabel } from '../../shared/audio'
import { StatusDot } from '../../shared/ui/StatusDot'
import { useSettingsStore } from './settings.store'

export function AudioStatusBanner() {
  const audioStatus = useSettingsStore((s) => s.audioStatus)

  if (!audioStatus) return null

  const driverLabel = audioDriverLabel(audioStatus.audioDriver)
  const dotStatus = !audioStatus.audioOk
    ? 'error'
    : audioStatus.audioWarning
      ? 'warning'
      : 'ok'
  const ok = dotStatus === 'ok'

  return (
    <div
      className={`rounded-xl border px-3 py-2.5 shrink-0 ${
        ok
          ? 'border-emerald-500/20 bg-emerald-500/5'
          : 'border-amber-500/30 bg-amber-500/5'
      }`}
    >
      <div className="flex items-center gap-2">
        <StatusDot status={dotStatus} />
        <p
          className={`text-xs ${
            ok ? 'text-emerald-400/90' : 'font-medium text-amber-400'
          }`}
        >
          Audio ·{' '}
          <span className={ok ? 'font-medium text-emerald-300' : undefined}>
            {driverLabel}
            {!audioStatus.audioOk && ' (no disponible)'}
          </span>
        </p>
      </div>
      {audioStatus.audioWarning && (
        <p className="mt-1.5 text-xs text-zinc-400 leading-relaxed pl-4">
          {audioStatus.audioWarning}
        </p>
      )}
    </div>
  )
}
