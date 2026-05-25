import { create } from 'zustand'
import { api } from '../../shared/api'
import { runSafely } from '../../shared/async'
import { audioFromDeps } from '../../shared/audio'
import type { AudioStatus, RunnerInfo } from '../../shared/types'
import { resolveRunnerAfterLoad } from './settings.logic'

interface SettingsState {
  runners: RunnerInfo[]
  selectedRunner: string
  audioStatus: AudioStatus | null
  init: () => Promise<void>
  loadSettings: () => Promise<void>
  loadRunners: () => Promise<void>
  loadAudioStatus: (runner: string) => Promise<void>
  setRunner: (path: string) => Promise<void>
}

export const useSettingsStore = create<SettingsState>((set, get) => ({
  runners: [],
  selectedRunner: '',
  audioStatus: null,

  init: async () => {
    await get().loadSettings()
    await get().loadRunners()
  },

  loadSettings: async () => {
    const settings = await api.loadSettings()
    set({ selectedRunner: settings.defaultRunner })
  },

  loadRunners: async () => {
    const runners = await api.listRunners()
    set({ runners })

    const resolution = resolveRunnerAfterLoad(get().selectedRunner, runners)
    if (!resolution) return

    if (resolution.persist) {
      const result = await runSafely(() =>
        api.saveSettings({ defaultRunner: resolution.path }),
      )
      if (!result.ok) return
    }

    set({ selectedRunner: resolution.path })
    await get().loadAudioStatus(resolution.path)
  },

  loadAudioStatus: async (runner: string) => {
    const result = await runSafely(() => api.checkDependencies(runner))
    set({ audioStatus: result.ok ? audioFromDeps(result.value) : null })
  },

  setRunner: async (path) => {
    const previous = get().selectedRunner
    const result = await runSafely(() => api.saveSettings({ defaultRunner: path }))
    if (!result.ok) {
      set({ selectedRunner: previous })
      return
    }
    set({ selectedRunner: path })
    await get().loadAudioStatus(path)
  },
}))
