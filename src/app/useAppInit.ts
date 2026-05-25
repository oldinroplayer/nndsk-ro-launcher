import { useEffect, useState } from 'react'
import { useServersStore } from '../features/servers/servers.store'
import { useSettingsStore } from '../features/settings/settings.store'

export function useAppInit() {
  const [ready, setReady] = useState(false)

  useEffect(() => {
    void Promise.all([
      useServersStore.getState().loadServers(),
      useSettingsStore.getState().init(),
    ]).finally(() => setReady(true))
  }, [])

  return { ready }
}
