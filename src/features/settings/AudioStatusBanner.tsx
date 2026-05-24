import { useSettingsStore } from './settings.store'

export function AudioStatusBanner() {
  const audioStatus = useSettingsStore((s) => s.audioStatus)

  if (!audioStatus) return null

  const driverLabel =
    audioStatus.audioDriver === 'pulse'
      ? 'PulseAudio'
      : audioStatus.audioDriver === 'alsa'
        ? 'ALSA'
        : 'sin driver'

  if (audioStatus.audioOk && !audioStatus.audioWarning) {
    return (
      <div className="rounded-xl border border-emerald-500/20 bg-emerald-500/5 px-3 py-2.5 shrink-0">
        <div className="flex items-center gap-2">
          <span className="inline-block w-2 h-2 rounded-full bg-emerald-500 shadow-[0_0_6px_rgba(16,185,129,0.5)] shrink-0" />
          <p className="text-xs text-emerald-400/90">
            Audio · <span className="font-medium text-emerald-300">{driverLabel}</span>
          </p>
        </div>
      </div>
    )
  }

  return (
    <div className="rounded-xl border border-amber-500/30 bg-amber-500/5 px-3 py-2.5 shrink-0">
      <div className="flex items-center gap-2">
        <span className="inline-block w-2 h-2 rounded-full bg-amber-500 shrink-0" />
        <p className="text-xs font-medium text-amber-400">
          Audio · {driverLabel}
          {!audioStatus.audioOk && ' (no disponible)'}
        </p>
      </div>
      {audioStatus.audioWarning && (
        <p className="mt-1.5 text-xs text-zinc-400 leading-relaxed pl-4">{audioStatus.audioWarning}</p>
      )}
    </div>
  )
}
