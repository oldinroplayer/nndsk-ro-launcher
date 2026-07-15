import { afterEach, describe, expect, it, vi } from 'vitest'

import { api } from '../../shared/api'
import {
  LEGACY_DEFAULT_WINE,
  PREFERRED_PROTON_ID,
} from '../../shared/constants'
import type { RunnerInfo } from '../../shared/types'
import { useSettingsStore } from './settings.store'

const proton: RunnerInfo = {
  id: PREFERRED_PROTON_ID,
  name: 'Proton recomendado',
  path: '/opt/proton/proton',
}

describe('settings store legacy runner migration', () => {
  afterEach(() => {
    vi.restoreAllMocks()
    useSettingsStore.setState({
      runners: [],
      selectedRunner: '',
      error: null,
      notice: null,
    })
  })

  it('changes runner and publishes a notice only after persistence succeeds', async () => {
    vi.spyOn(api, 'listRunners').mockResolvedValue([proton])
    vi.spyOn(api, 'saveSettings').mockResolvedValue(undefined)
    const loadDepsStatus = vi.fn().mockResolvedValue(undefined)
    useSettingsStore.setState({
      selectedRunner: LEGACY_DEFAULT_WINE,
      loadDepsStatus,
    })

    await useSettingsStore.getState().loadRunners()

    expect(api.saveSettings).toHaveBeenCalledWith({
      defaultRunner: proton.path,
    })
    expect(useSettingsStore.getState().selectedRunner).toBe(proton.path)
    expect(useSettingsStore.getState().notice?.kind).toBe('migrated')
    expect(loadDepsStatus).toHaveBeenCalledWith(proton.path)
  })

  it('keeps the legacy runner and propagates persistence failures', async () => {
    vi.spyOn(api, 'listRunners').mockResolvedValue([proton])
    vi.spyOn(api, 'saveSettings').mockRejectedValue(new Error('disk full'))
    useSettingsStore.setState({ selectedRunner: LEGACY_DEFAULT_WINE })

    await expect(useSettingsStore.getState().loadRunners()).rejects.toThrow(
      'disk full',
    )
    expect(useSettingsStore.getState().selectedRunner).toBe(LEGACY_DEFAULT_WINE)
    expect(useSettingsStore.getState().notice).toBeNull()
    expect(useSettingsStore.getState().error).toBe('disk full')
  })
})
