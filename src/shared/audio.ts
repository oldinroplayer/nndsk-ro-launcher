import type { AudioStatus, DependencyStatus } from './types'

export function audioFromDeps(deps: DependencyStatus): AudioStatus {
  return {
    audioOk: deps.audioOk,
    audioDriver: deps.audioDriver,
    audioWarning: deps.audioWarning,
  }
}

export function audioDriverLabel(driver: string): string {
  switch (driver) {
    case 'pulse':
      return 'PulseAudio'
    case 'alsa':
      return 'ALSA'
    default:
      return 'sin driver'
  }
}
