import { listen } from '@tauri-apps/api/event'
import { useEffect } from 'react'
import type { ProgressPayload } from '../../shared/types'
import { useLauncherStore } from './launcher.store'
import { useLogsStore } from '../logs/logs.store'
import { LAUNCHER_EVENTS } from '../../shared/constants'

export function useLauncherEvents() {
  const { setStatus, setProgress, setError } = useLauncherStore()
  const addLog = useLogsStore((s) => s.addLog)

  useEffect(() => {
    const cleanups: Array<() => void> = []

    listen<{ line: string }>(LAUNCHER_EVENTS.LOG, (e) =>
      addLog(e.payload.line),
    ).then((fn) => cleanups.push(fn))

    listen<ProgressPayload>(LAUNCHER_EVENTS.PROGRESS, (e) =>
      setProgress(e.payload),
    ).then((fn) => cleanups.push(fn))

    listen<{ code: number }>(LAUNCHER_EVENTS.GAME_EXIT, (e) => {
      const code = e.payload.code
      if (code !== 0) {
        addLog(`El juego cerró inesperadamente (código ${code})`)
        setError(`El juego cerró inesperadamente (código ${code})`)
        setStatus('error')
      } else {
        addLog('Juego cerrado')
        setStatus('idle')
      }
    }).then((fn) => cleanups.push(fn))

    listen<{ message: string }>(LAUNCHER_EVENTS.ERROR, (e) => {
      setError(e.payload.message)
      setStatus('error')
    }).then((fn) => cleanups.push(fn))

    return () => cleanups.forEach((fn) => fn())
  }, [])
}
