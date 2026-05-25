import type {
  ExitEventPayload,
  LogEventPayload,
  ProgressPayload,
} from '../../shared/types'
import { useLauncherStore } from './launcher.store'
import { useLogsStore } from '../logs/logs.store'
import { LAUNCHER_EVENTS } from '../../shared/constants'
import { useTauriEvent } from '../../shared/hooks/useTauriEvent'

export function useLauncherEvents() {
  const setStatus = useLauncherStore((s) => s.setStatus)
  const setProgress = useLauncherStore((s) => s.setProgress)
  const setError = useLauncherStore((s) => s.setError)
  const addLog = useLogsStore((s) => s.addLog)

  useTauriEvent<LogEventPayload>(
    LAUNCHER_EVENTS.LOG,
    (payload) => addLog(payload.line),
    [addLog],
  )

  useTauriEvent<ProgressPayload>(
    LAUNCHER_EVENTS.PROGRESS,
    (payload) => setProgress(payload),
    [setProgress],
  )

  useTauriEvent<ExitEventPayload>(
    LAUNCHER_EVENTS.GAME_EXIT,
    (payload) => {
      const { code } = payload
      if (code !== 0) {
        const msg = `El juego cerró inesperadamente (código ${code})`
        addLog(msg)
        setError(msg)
        setStatus('error')
      } else {
        addLog('Juego cerrado')
        setStatus('idle')
      }
    },
    [addLog, setError, setStatus],
  )
}
