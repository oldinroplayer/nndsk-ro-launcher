import { api } from '../../shared/api'
import { resolveRunner, withResolvedRunner } from '../../shared/resolveRunner'
import type { ServerConfig } from '../../shared/types'
import { useSettingsStore } from '../settings/settings.store'
import { useLauncherTask } from './useLauncherTask'

export function useLaunchGame(server: ServerConfig | null) {
  const selectedRunner = useSettingsStore((s) => s.selectedRunner)
  const {
    status,
    setupProgress,
    error,
    setStatus,
    setProgress,
    setError,
    addLog,
    runTask,
    isBusy,
  } = useLauncherTask()

  const handleLaunch = async () => {
    if (!server) return
    if (status === 'error') setError(null)

    const runner = resolveRunner(server, selectedRunner)

    await runTask(async () => {
      const deps = await api.checkDependencies(runner)

      if (deps.audioWarning) {
        addLog(deps.audioWarning)
      }

      if (!deps.prefixConfigured) {
        setStatus('setting-up')
        addLog('Configurando entorno por primera vez...')
        await api.setupPrefix()
        setProgress(null)
      }

      setStatus('launching')
      addLog(`Lanzando ${server.name}...`)

      await api.launchGame(withResolvedRunner(server, selectedRunner))
      setStatus('running')
    })
  }

  const handleStop = () => {
    void api.stopGame()
  }

  return {
    status,
    setupProgress,
    error,
    isBusy,
    handleLaunch,
    handleStop,
  }
}
