import type { CombatInputBackend } from '../../shared/types'
import { DarkSelect } from '../../shared/ui/DarkSelect'
import { Panel } from '../../shared/ui/Panel'
import { isLauncherBusy, useLauncherStore } from '../launcher/launcher.store'
import { useSelectedServer } from '../servers/useSelectedServer'
import { useServersStore } from '../servers/servers.store'

const OPTIONS = [
  { value: 'uinput', label: 'Estable · uinput 10 ms' },
  { value: 'ydotool', label: 'Compatibilidad · ydotool' },
]

export function CombatInputSelector() {
  const server = useSelectedServer()
  const updateServer = useServersStore((state) => state.updateServer)
  const launchStatus = useLauncherStore((state) => state.status)
  const disabled = !server || isLauncherBusy(launchStatus)

  return (
    <Panel title="Input de combate" className="shrink-0">
      <DarkSelect
        value={server?.combatInputBackend ?? 'uinput'}
        disabled={disabled}
        options={OPTIONS}
        onChange={(value) => {
          if (!server || disabled) return
          void updateServer(server.id, {
            combatInputBackend: value as CombatInputBackend,
          })
        }}
      />
      <p className="mt-1.5 text-[10px] leading-snug text-zinc-600">
        uinput requiere acceso directo a /dev/uinput. No hay fallback silencioso.
      </p>
    </Panel>
  )
}
