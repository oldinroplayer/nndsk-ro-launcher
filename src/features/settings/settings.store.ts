import { invoke } from '@tauri-apps/api/core'
import { create } from 'zustand'
import type { DependencyStatus, RunnerInfo, AudioStatus } from '../../shared/types'

interface SettingsState {
  runners: RunnerInfo[]
  selectedRunner: string
  audioStatus: AudioStatus | null
  loadSettings: () => Promise<void>
  loadRunners: () => Promise<void>
  loadAudioStatus: (runner: string) => Promise<void>
  setRunner: (path: string) => Promise<void>
}

export const useSettingsStore = create<SettingsState>((set, get) => ({
  runners: [],
  selectedRunner: '',
  audioStatus: null,

  loadSettings: async () => {
    const settings = await invoke<{ defaultRunner: string }>('load_settings')
    set({ selectedRunner: settings.defaultRunner })
  },

  loadRunners: async () => {
    const runners = await invoke<RunnerInfo[]>('list_runners')
    set({ runners })

    const current = get().selectedRunner
    const preferred = runners.find((r) => r.id === 'proton-proton-cachyos-slr')
    const fallback = runners[0]

    if (!current && fallback) {
      const runner = preferred?.path ?? fallback.path
      set({ selectedRunner: runner })
      await get().loadAudioStatus(runner)
      return
    }

    // Migrate stale duplicate paths: prefer canonical proton if user had wine default
    if (current === '/usr/bin/wine' && preferred) {
      set({ selectedRunner: preferred.path })
      await invoke('save_settings', { settings: { defaultRunner: preferred.path } })
      await get().loadAudioStatus(preferred.path)
      return
    }

    if (current) {
      await get().loadAudioStatus(current)
    }
  },

  loadAudioStatus: async (runner: string) => {
    try {
      const deps = await invoke<DependencyStatus>('check_dependencies', { runner })
      set({
        audioStatus: {
          audioOk: deps.audioOk,
          audioDriver: deps.audioDriver,
          audioWarning: deps.audioWarning,
        },
      })
    } catch {
      set({ audioStatus: null })
    }
  },

  setRunner: async (path) => {
    set({ selectedRunner: path })
    await invoke('save_settings', { settings: { defaultRunner: path } })
    await get().loadAudioStatus(path)
  },
}))
