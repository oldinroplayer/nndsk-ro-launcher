import { invoke } from '@tauri-apps/api/core'
import { useEffect, useState } from 'react'
import { useServersStore } from '../features/servers/servers.store'
import { useSettingsStore } from '../features/settings/settings.store'

export function useAppInit() {
  const [ready, setReady] = useState(false)

  useEffect(() => {
    void Promise.all([
      useServersStore.getState().loadServers(),
      useSettingsStore.getState().init(),
    ]).finally(() => {
      void invoke('show_main_window')
      setReady(true)
    })
  }, [])

  return { ready }
}
