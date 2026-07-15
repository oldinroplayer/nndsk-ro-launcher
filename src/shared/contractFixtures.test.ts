import { describe, expect, it } from 'vitest'
import fixtures from '../../contract-fixtures/server-configs.json'
import { mergeAutobuffConfig } from '../features/autobuff/autobuff.logic'
import { mergeAutopotConfig } from '../features/autopot/autopot.logic'
import { mergeSpammerConfig } from '../features/spammer/spammer.logic'
import type { ServerConfig } from './types'
import { validateServerConfig } from './contracts'

describe('shared Rust/TypeScript contract fixtures', () => {
  it('agrees on valid and invalid server boundaries', () => {
    expect(
      validateServerConfig(fixtures.validServer as ServerConfig),
    ).toBeNull()
    for (const fixture of fixtures.invalidServers) {
      expect(
        validateServerConfig(fixture.server as ServerConfig),
      ).not.toBeNull()
    }
  })

  it('keeps common configuration defaults in parity', () => {
    expect(fixtures.defaults.combatInputBackend).toBe('uinput')
    expect(mergeAutopotConfig()).toMatchObject(fixtures.defaults.autopot)
    expect(mergeSpammerConfig()).toEqual(fixtures.defaults.spammer)
    expect(mergeAutobuffConfig()).toEqual(fixtures.defaults.autobuff)
  })

  it('rejects an unknown combat input backend', () => {
    expect(
      validateServerConfig({
        ...(fixtures.validServer as ServerConfig),
        combatInputBackend: 'unknown' as ServerConfig['combatInputBackend'],
      }),
    ).not.toBeNull()
  })

  it('documents legacy combat backends as uinput migrations', () => {
    expect(fixtures.legacyCombatInputBackends).toEqual([
      { input: 'stable', expectedCanonical: 'uinput' },
      { input: 'lowLatency', expectedCanonical: 'uinput' },
    ])
  })

  it('migrates the legacy gear schema identically', () => {
    const config = mergeSpammerConfig(
      fixtures.legacySpammer.input as Parameters<typeof mergeSpammerConfig>[0],
    )
    expect(config.gearSwitch.rules).toEqual(
      fixtures.legacySpammer.expectedRules,
    )
    expect(config).toEqual(fixtures.legacySpammer.expectedCanonical)
  })
})
