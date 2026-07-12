import { useEffect } from 'react'
import { useLauncherStore } from '../features/launcher/launcher.store'
import { modeForStatus, useUiModeStore } from './uiMode.store'

export function useUiModeTransition() {
  const status = useLauncherStore((s) => s.status)

  useEffect(() => {
    const next = modeForStatus(status)
    if (next === useUiModeStore.getState().mode) return
    useUiModeStore.getState().setMode(next)
  }, [status])
}
