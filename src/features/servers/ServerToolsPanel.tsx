import { invoke } from '@tauri-apps/api/core'
import { useCallback, useEffect, useState } from 'react'
import type { ServerConfig } from './servers.types'
import type { ServerToolsStatus, ToolKind } from '../../shared/types'
import { useSettingsStore } from '../settings/settings.store'
import { Panel } from '../../shared/ui/Panel'
import { ToolRow } from './ToolRow'
import { resolveRunner } from '../../shared/resolveRunner'

interface Props {
  server: ServerConfig | null
}

export function ServerToolsPanel({ server }: Props) {
  const selectedRunner = useSettingsStore((s) => s.selectedRunner)
  const [status, setStatus] = useState<ServerToolsStatus | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [opening, setOpening] = useState<ToolKind | null>(null)
  const [installingDgVoodoo, setInstallingDgVoodoo] = useState(false)
  const [uninstallingDgVoodoo, setUninstallingDgVoodoo] = useState(false)

  const refresh = useCallback(async () => {
    if (!server) {
      setStatus(null)
      setError(null)
      return
    }

    setLoading(true)
    setError(null)
    try {
      const result = await invoke<ServerToolsStatus>('scan_server_tools', { server })
      setStatus(result)
    } catch (err) {
      setStatus(null)
      setError(String(err))
    } finally {
      setLoading(false)
    }
  }, [server])

  useEffect(() => {
    refresh()
  }, [refresh])

  const handleInstallDgVoodoo = async () => {
    if (!server) return

    setInstallingDgVoodoo(true)
    setError(null)
    try {
      const result = await invoke<{ installed: string[]; status: ServerToolsStatus }>(
        'install_dgvoodoo',
        { server },
      )
      setStatus(result.status)
    } catch (err) {
      setError(String(err))
    } finally {
      setInstallingDgVoodoo(false)
    }
  }

  const handleUninstallDgVoodoo = async () => {
    if (!server) return

    const confirmed = window.confirm(
      '¿Desinstalar dgVoodoo de esta carpeta?\n\nSe eliminarán D3DImm.dll, DDraw.dll, dgVoodoo.conf y dgVoodooCpl.exe.',
    )
    if (!confirmed) return

    setUninstallingDgVoodoo(true)
    setError(null)
    try {
      const result = await invoke<{ removed: string[]; status: ServerToolsStatus }>(
        'uninstall_dgvoodoo',
        { server },
      )
      setStatus(result.status)
    } catch (err) {
      setError(String(err))
    } finally {
      setUninstallingDgVoodoo(false)
    }
  }

  const handleOpen = async (tool: ToolKind) => {
    if (!server) return

    setOpening(tool)
    setError(null)
    try {
      await invoke('launch_server_tool', {
        server: {
          ...server,
          runner: resolveRunner(server, selectedRunner),
        },
        tool,
        runner: resolveRunner(server, selectedRunner),
      })
    } catch (err) {
      setError(String(err))
    } finally {
      setOpening(null)
    }
  }

  if (!server) {
    return (
      <Panel title="Herramientas" className="shrink-0">
        <p className="text-sm text-zinc-600 text-center py-2">
          Selecciona un servidor para escanear herramientas
        </p>
      </Panel>
    )
  }

  const dg = status?.dgvoodoo
  const dgvoodooNeedsInstall = dg && !dg.configured && dg.canAutoInstall

  return (
    <Panel
      title="Herramientas"
      className="shrink-0"
      action={
        <button
          type="button"
          onClick={refresh}
          disabled={loading}
          className="text-xs text-zinc-600 hover:text-zinc-400 transition-colors disabled:opacity-40 px-1"
          title="Volver a escanear"
        >
          {loading ? '...' : '↻'}
        </button>
      }
    >
      {error && <p className="text-xs text-red-400 mb-2">{error}</p>}

      {!loading && status && (
        <div>
          <ToolRow
            label="OpenSetup"
            dotStatus={status.openSetup.found ? 'ok' : 'neutral'}
            detail={status.openSetup.label ?? (status.openSetup.found ? 'Detectado' : 'No encontrado')}
            onAction={status.openSetup.found ? () => handleOpen('opensetup') : undefined}
            actionLabel="Abrir"
            actionBusy={opening === 'opensetup'}
            actionDisabled={!status.openSetup.found}
          />
          <ToolRow
            label="Patcher"
            dotStatus={status.patcher.found ? 'ok' : 'neutral'}
            detail={status.patcher.label ?? (status.patcher.found ? 'Detectado' : 'No encontrado')}
            onAction={status.patcher.found ? () => handleOpen('patcher') : undefined}
            actionLabel="Abrir"
            actionBusy={opening === 'patcher'}
            actionDisabled={!status.patcher.found}
          />
          <ToolRow
            label="dgVoodoo"
            dotStatus={status.dgvoodoo.configured ? 'ok' : 'error'}
            detail={
              status.dgvoodoo.configured
                ? 'D3DImm · DDraw · conf OK'
                : 'No detectado'
            }
            warning={
              !status.dgvoodoo.configured && status.dgvoodoo.issues.length > 0
                ? status.dgvoodoo.issues.join(' · ')
                : undefined
            }
            onAction={
              dgvoodooNeedsInstall
                ? handleInstallDgVoodoo
                : status.dgvoodoo.cpl.found
                  ? () => handleOpen('dgvoodoo')
                  : undefined
            }
            actionLabel={dgvoodooNeedsInstall ? 'Instalar' : 'Configurar'}
            actionBusy={dgvoodooNeedsInstall ? installingDgVoodoo : opening === 'dgvoodoo'}
            onSecondary={
              status.dgvoodoo.canUninstall ? handleUninstallDgVoodoo : undefined
            }
            secondaryLabel="Desinstalar"
            secondaryBusy={uninstallingDgVoodoo}
            secondaryDanger
          />
        </div>
      )}

      {loading && !status && (
        <p className="text-xs text-zinc-600 py-2 text-center">Escaneando carpeta...</p>
      )}
    </Panel>
  )
}
