import { invoke } from '@tauri-apps/api/core'
import type { DependencyStatus, ServerConfig } from '../../shared/types'
import { useLauncherStore, isLauncherBusy } from './launcher.store'
import { useLogsStore } from '../logs/logs.store'
import { useSettingsStore } from '../settings/settings.store'
import { resolveRunner, withResolvedRunner } from '../../shared/resolveRunner'

interface Props {
  server: ServerConfig | null
}

export function LaunchButton({ server }: Props) {
  const { status, setupProgress, error, setStatus, setProgress, setError } =
    useLauncherStore()
  const addLog = useLogsStore((s) => s.addLog)
  const selectedRunner = useSettingsStore((s) => s.selectedRunner)

  const handleLaunch = async () => {
    if (!server) return
    if (status === 'error') setError(null)

    try {
      const deps = await invoke<DependencyStatus>('check_dependencies', {
        runner: resolveRunner(server, selectedRunner),
      })

      if (deps.audioWarning) {
        addLog(deps.audioWarning)
      }

      if (!deps.prefixConfigured) {
        setStatus('setting-up')
        addLog('Configurando entorno por primera vez...')
        await invoke('setup_prefix')
        setProgress(null)
      }

      setStatus('launching')
      addLog(`Lanzando ${server.name} con ${selectedRunner || 'wine'}...`)

      await invoke('launch_game', {
        server: withResolvedRunner(server, selectedRunner),
      })
      setStatus('running')
    } catch (err) {
      const msg = String(err)
      setError(msg)
      setStatus('error')
      addLog(`Error: ${msg}`)
    }
  }

  const isDisabled = !server || isLauncherBusy(status)

  const labels: Record<typeof status, string> = {
    idle: 'JUGAR',
    'setting-up': setupProgress?.step ?? 'Configurando...',
    launching: 'Iniciando...',
    running: 'Jugando...',
    error: 'Reintentar',
  }

  return (
    <div className="flex flex-col gap-2 shrink-0">
      {status === 'setting-up' && setupProgress && (
        <div className="w-full bg-zinc-800/80 rounded-full h-1 overflow-hidden">
          <div
            className="bg-gradient-to-r from-amber-600 to-amber-400 h-1 rounded-full transition-all duration-500"
            style={{ width: `${setupProgress.percent}%` }}
          />
        </div>
      )}
      <button
        onClick={handleLaunch}
        disabled={isDisabled}
        className="w-full py-3.5 px-6 rounded-xl font-bold text-lg tracking-[0.2em] transition-all duration-200
          bg-gradient-to-b from-amber-400 to-amber-500 hover:from-amber-300 hover:to-amber-400
          active:from-amber-500 active:to-amber-600 text-zinc-950 shadow-lg shadow-amber-500/10
          disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:from-amber-400 disabled:hover:to-amber-500 disabled:shadow-none"
      >
        {labels[status]}
      </button>
      {status === 'running' && (
        <button
          onClick={() => invoke('stop_game')}
          className="w-full py-2 rounded-xl text-xs text-zinc-500 hover:text-red-400 border border-zinc-800/80
            hover:border-red-500/30 hover:bg-red-500/5 transition-colors"
        >
          Detener juego
        </button>
      )}
      {status === 'error' && error && (
        <p className="text-red-400 text-xs text-center px-2 leading-relaxed">{error}</p>
      )}
    </div>
  )
}
