import { Panel, type PanelTone } from '../../shared/ui/Panel'
import { ToggleSwitch } from '../../shared/ui/ToggleSwitch'
import { useSelectedServer } from '../servers/useSelectedServer'
import { useSpammer } from './useSpammer'

function resolveTone(
  available: boolean,
  enabled: boolean,
  armed: boolean,
  hasError: boolean,
): PanelTone {
  if (hasError) return 'danger'
  if (!available) return 'idle'
  if (enabled && armed) return 'warning'
  return 'neutral'
}

export function SpammerPanel() {
  const server = useSelectedServer()
  const { config, status, busy, isRunning, error, setEnabled, updateField } =
    useSpammer(server)

  const available = isRunning && !!server

  const statusLabel = (() => {
    if (!available) return 'Inactivo'
    if (!status.armed) return 'Inactivo'
    if (status.spamming) {
      return `${status.cycleCount.toLocaleString()} ciclos · F1 + click`
    }
    return 'Standby — mantén F1'
  })()

  const statusText = !server
    ? 'Selecciona un servidor'
    : !isRunning
      ? 'Inicia el juego'
      : status.spamming
        ? 'Spameando...'
        : 'Mantén F1 en el juego'

  const tone = resolveTone(available, config.enabled, status.armed, !!error)

  return (
    <Panel title="Spammer" compact tone={tone} className="h-full">
      <div className="space-y-2">
        <div className="flex items-start justify-between gap-2">
          <div className="min-w-0 flex-1">
            <p
              className={`text-sm font-semibold truncate ${
                status.spamming ? 'text-amber-200' : 'text-zinc-100'
              }`}
            >
              {statusLabel}
            </p>
            <p className="text-[10px] text-zinc-600">{statusText}</p>
          </div>
          <ToggleSwitch
            checked={config.enabled && available}
            disabled={!available || busy}
            onChange={(enabled) => void setEnabled(enabled)}
            tone="amber"
          />
        </div>

        <div className="space-y-1.5 rounded-lg bg-zinc-950/40 border border-zinc-800/60 px-2.5 py-2 min-h-[52px]">
          <div className="flex justify-between text-[10px]">
            <span className="text-zinc-600">Modo</span>
            <span
              className={
                available && status.armed
                  ? 'text-amber-400/90 font-medium'
                  : 'text-zinc-700'
              }
            >
              F1 + click
            </span>
          </div>
          <p className="text-[10px] text-zinc-600 leading-snug">
            Skill en barra + target con click izquierdo
          </p>
        </div>

        <div className="flex items-center gap-2">
          <span className="text-[10px] text-zinc-600 uppercase tracking-wide shrink-0">
            Delay
          </span>
          <input
            type="range"
            min={5}
            max={50}
            step={1}
            disabled={!server || busy}
            value={config.delayMs}
            onChange={(e) => void updateField({ delayMs: Number(e.target.value) })}
            className="flex-1 accent-amber-500 disabled:opacity-50"
          />
          <span className="text-[10px] text-zinc-500 w-8 text-right shrink-0">
            {config.delayMs}ms
          </span>
        </div>

        {error && available ? (
          <p className="text-[10px] text-red-400/90 leading-snug">{error}</p>
        ) : null}
      </div>
    </Panel>
  )
}
