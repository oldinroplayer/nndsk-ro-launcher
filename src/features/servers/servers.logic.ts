import type { ServerConfig } from '../../shared/types'

/** Calcula el servidor seleccionado tras eliminar uno de la lista. */
export function nextSelectedId(
  currentId: string | null,
  removedId: string,
  servers: ServerConfig[],
): string | null {
  if (currentId !== removedId) return currentId
  return servers[0]?.id ?? null
}

/** Resuelve el servidor activo a partir del id seleccionado. */
export function findSelectedServer(
  servers: ServerConfig[],
  selectedId: string | null,
): ServerConfig | null {
  return servers.find((s) => s.id === selectedId) ?? null
}

/** Primer id de la lista, o null si está vacía. */
export function firstServerId(servers: ServerConfig[]): string | null {
  return servers[0]?.id ?? null
}
