import { useLaunchGame } from './useLaunchGame'
import { useSelectedServer } from '../servers/useSelectedServer'
import { useSettingsStore } from '../settings/settings.store'

export function LaunchButton() {
  const server = useSelectedServer()
  const prefixConfigured = useSettingsStore((s) => s.prefixConfigured)
  const { status, setupProgress, error, isBusy, handleLaunch, handleStop } = useLaunchGame(server)

  const isDisabled = !server || isBusy
  const buildMode = status === 'idle' && !prefixConfigured

  const labels: Record<typeof status, string> = {
    idle: buildMode ? 'Construir entorno' : 'Jugar',
    'setting-up': 'Configurando...',
    launching: 'Iniciando...',
    running: 'Jugando...',
    error: 'Reintentar',
  }

  const baseBtn =
    'w-full py-2.5 px-4 rounded-xl text-sm font-semibold transition-colors duration-200 disabled:opacity-50 disabled:cursor-not-allowed'

  let buttonClass: string
  if (status === 'running') {
    buttonClass = `${baseBtn} border border-emerald-500/30 bg-emerald-500/10 text-emerald-300 cursor-default`
  } else if (status === 'error') {
    buttonClass = `${baseBtn} border border-red-500/30 bg-red-500/10 text-red-300 hover:bg-red-500/15 hover:border-red-500/50 active:bg-red-500/20`
  } else if (buildMode) {
    buttonClass = `${baseBtn} border border-zinc-700/80 bg-zinc-900/40 text-zinc-300 hover:border-amber-500/40 hover:text-amber-400 hover:bg-amber-500/5 active:bg-amber-500/10 disabled:hover:border-zinc-700/80 disabled:hover:text-zinc-300 disabled:hover:bg-zinc-900/40`
  } else {
    buttonClass = `${baseBtn} border border-amber-500/30 bg-amber-500/10 text-amber-100 hover:bg-amber-500/15 hover:border-amber-500/50 active:bg-amber-500/20 disabled:hover:bg-amber-500/10 disabled:hover:border-amber-500/30`
  }

  return (
    <div className="flex flex-col gap-2 shrink-0 border-t border-zinc-800/80 pt-3">
      {status === 'setting-up' && setupProgress && (
        <div className="space-y-1">
          <div className="flex justify-between gap-2 text-[10px] text-zinc-500">
            <span className="truncate">{setupProgress.step}</span>
            <span className="shrink-0 tabular-nums">{setupProgress.percent}%</span>
          </div>
          <div className="w-full bg-zinc-800 rounded-full h-1.5 overflow-hidden">
            <div
              className="h-full bg-gradient-to-r from-amber-600 to-amber-400 rounded-full transition-all duration-500"
              style={{ width: `${setupProgress.percent}%` }}
            />
          </div>
        </div>
      )}
      <button
        type="button"
        onClick={handleLaunch}
        disabled={isDisabled}
        className={buttonClass}
      >
        {labels[status]}
      </button>
      {status === 'running' && (
        <button
          type="button"
          onClick={handleStop}
          className="w-full py-2 rounded-xl text-xs text-zinc-500 hover:text-red-400 border border-zinc-800/80
            hover:border-red-500/30 hover:bg-red-500/5 transition-colors"
        >
          Detener juego
        </button>
      )}
      {status === 'error' && error && (
        <p className="text-red-400 text-[11px] text-center px-2 leading-relaxed">{error}</p>
      )}
    </div>
  )
}
