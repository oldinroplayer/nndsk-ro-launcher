import type { ServerConfig } from './types'

export function resolveRunner(
  server: ServerConfig,
  selectedRunner: string,
): string | null {
  return server.runner ?? (selectedRunner || null)
}

/** Servidor con runner efectivo (override del servidor o global). */
export function withResolvedRunner(
  server: ServerConfig,
  selectedRunner: string,
): ServerConfig {
  const runner = resolveRunner(server, selectedRunner)
  return { ...server, runner: runner ?? undefined }
}
