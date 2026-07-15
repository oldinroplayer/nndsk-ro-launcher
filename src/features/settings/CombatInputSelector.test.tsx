// @vitest-environment jsdom

import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { api } from '../../shared/api'
import type { ServerConfig } from '../../shared/types'
import { useLauncherStore } from '../launcher/launcher.store'
import { useServersStore } from '../servers/servers.store'
import { CombatInputSelector } from './CombatInputSelector'

const server: ServerConfig = {
  id: 'server-1',
  name: 'Test RO',
  executablePath: '/games/test/Ragexe.exe',
}

describe('CombatInputSelector', () => {
  beforeEach(() => {
    useServersStore.setState({
      servers: [server],
      selectedId: server.id,
      loading: false,
      error: null,
    })
    useLauncherStore.setState({ status: 'idle', error: null })
    vi.spyOn(api, 'saveServers').mockResolvedValue()
  })

  afterEach(() => {
    cleanup()
    vi.restoreAllMocks()
  })

  it('persists ydotool compatibility for the selected server', async () => {
    render(<CombatInputSelector />)

    fireEvent.click(
      screen.getByRole('button', { name: 'Estable · uinput 10 ms' }),
    )
    fireEvent.click(
      screen.getByRole('button', {
        name: 'Compatibilidad · ydotool',
      }),
    )

    await waitFor(() => {
      expect(useServersStore.getState().servers[0].combatInputBackend).toBe('ydotool')
    })
    expect(api.saveServers).toHaveBeenCalledWith([
      { ...server, combatInputBackend: 'ydotool' },
    ])
  })

  it.each(['launching', 'running'] as const)(
    'is locked while launcher is %s',
    (status) => {
      useLauncherStore.setState({ status })
      render(<CombatInputSelector />)

      expect(
        screen.getByRole('button', { name: 'Estable · uinput 10 ms' }),
      ).toBeDisabled()
    },
  )
})
