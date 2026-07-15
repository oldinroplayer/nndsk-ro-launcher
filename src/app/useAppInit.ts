import { invoke } from '@tauri-apps/api/core'
import { useCallback, useEffect, useRef, useState } from 'react'
import { toErrorMessage } from '../shared/errors'
import { useServersStore } from '../features/servers/servers.store'
import { useSettingsStore } from '../features/settings/settings.store'
import { api } from '../shared/api'
import type { StorageNotice } from '../shared/types'

export function useAppInit() {
  const [phase, setPhase] = useState<'loading' | 'ready' | 'degraded'>(
    'loading',
  )
  const [errors, setErrors] = useState<string[]>([])
  const [retrying, setRetrying] = useState(false)
  const [notices, setNotices] = useState<StorageNotice[]>([])
  const started = useRef(false)

  const initialize = useCallback(async (initial: boolean) => {
    if (initial) setPhase('loading')
    else setRetrying(true)
    setNotices([])

    const nextErrors: string[] = []
    const [serversOk, settingsOk] = await Promise.all([
      useServersStore.getState().loadServers(),
      useSettingsStore.getState().init(),
    ])
    if (!serversOk) {
      nextErrors.push(
        useServersStore.getState().error ??
          'No se pudieron cargar los servidores',
      )
    }
    if (!settingsOk) {
      nextErrors.push(
        useSettingsStore.getState().error ??
          'No se pudo cargar la configuración',
      )
    }

    try {
      const storageNotices = await api.takeStorageNotices()
      const settingsNotice = useSettingsStore.getState().notice
      setNotices(
        settingsNotice ? [...storageNotices, settingsNotice] : storageNotices,
      )
    } catch (error) {
      nextErrors.push(
        `No se pudieron consultar los avisos de almacenamiento: ${toErrorMessage(error)}`,
      )
    }

    try {
      await invoke('show_main_window')
    } catch (error) {
      nextErrors.push(`No se pudo mostrar la ventana: ${toErrorMessage(error)}`)
    }

    setErrors(nextErrors)
    setPhase(nextErrors.length === 0 ? 'ready' : 'degraded')
    setRetrying(false)
  }, [])

  useEffect(() => {
    if (started.current) return
    started.current = true
    void initialize(true)
  }, [initialize])

  return {
    phase,
    errors,
    retrying,
    notices,
    dismissNotices: () => setNotices([]),
    retry: () => initialize(false),
  }
}
