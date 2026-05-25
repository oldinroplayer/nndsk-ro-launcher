export const LAUNCHER_EVENTS = {
  LOG: 'ro-launcher://log',
  PROGRESS: 'ro-launcher://progress',
  GAME_EXIT: 'ro-launcher://game-exit',
} as const

/** Carpeta del compatibility tool en Steam (debe coincidir con PROTON_CACHYOS_SLR en Rust). */
export const PROTON_CACHYOS_SLR = 'proton-cachyos-slr'

/** Id generado por list_runners: `proton-{folder}`. */
export const PREFERRED_PROTON_ID = `proton-${PROTON_CACHYOS_SLR}` as const

/** Ruta por defecto del WINEPREFIX (solo para mensajes al usuario). */
export const DEFAULT_PREFIX_PATH = '~/.local/share/ro-launcher/prefix'

export const LEGACY_DEFAULT_WINE = '/usr/bin/wine'
