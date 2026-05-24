import type { ServerConfig } from '../features/servers/servers.types'

export function resolveRunner(
  server: ServerConfig,
  selectedRunner: string,
): string | null {
  return server.runner ?? (selectedRunner || null)
}
