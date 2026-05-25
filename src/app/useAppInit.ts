import { useEffect } from 'react'
import { useServersStore } from '../features/servers/servers.store'
import { useSettingsStore } from '../features/settings/settings.store'

export function useAppInit() {
  useEffect(() => {
    void Promise.all([
      useServersStore.getState().loadServers(),
      useSettingsStore.getState().init(),
    ])
  }, [])
}
