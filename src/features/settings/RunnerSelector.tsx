import { useSettingsStore } from './settings.store'
import { Panel } from '../../shared/ui/Panel'
import { DarkSelect } from '../../shared/ui/DarkSelect'

export function RunnerSelector() {
  const { runners, selectedRunner, setRunner } = useSettingsStore()

  if (runners.length === 0) return null

  return (
    <Panel title="Runner" className="shrink-0">
      <DarkSelect
        value={selectedRunner}
        options={runners.map((r) => ({ value: r.path, label: r.name }))}
        onChange={setRunner}
      />
    </Panel>
  )
}
