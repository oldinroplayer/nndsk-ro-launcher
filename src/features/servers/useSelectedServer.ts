import { useServersStore } from './servers.store'
import { findSelectedServer } from './servers.logic'

/** Servidor activo; se re-suscribe a `servers` y `selectedId`. */
export function useSelectedServer() {
  return useServersStore((s) => findSelectedServer(s.servers, s.selectedId))
}
