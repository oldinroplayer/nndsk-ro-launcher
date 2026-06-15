import { describe, expect, it } from 'vitest'
import { SPAMMER_KEYS } from '../../shared/constants'
import {
  formatSpammerKeys,
  mergeSpammerConfig,
  toggleSpammerKey,
  withSpammerPatch,
} from './spammer.logic'

describe('mergeSpammerConfig', () => {
  it('defaults keys to F1 when missing', () => {
    expect(mergeSpammerConfig({ delayMs: 20 })).toMatchObject({
      keys: ['F1'],
      delayMs: 20,
      enabled: false,
    })
  })

  it('normalizes and deduplicates keys', () => {
    expect(
      mergeSpammerConfig({ keys: ['z', 'f2', 'F1', 'F2', 'q', 'SPACE'] }),
    ).toMatchObject({
      keys: ['F1', 'F2', 'Q', 'Z'],
    })
  })

  it('keeps every supported key', () => {
    expect(mergeSpammerConfig({ keys: [...SPAMMER_KEYS] }).keys).toEqual(
      SPAMMER_KEYS,
    )
  })
})

describe('toggleSpammerKey', () => {
  it('adds and removes keys', () => {
    const base = mergeSpammerConfig({ keys: ['F1'] })
    const withF2 = toggleSpammerKey(base, 'F2')
    expect(withF2.keys).toEqual(['F1', 'F2'])
    const withLetter = toggleSpammerKey(withF2, 'p')
    expect(withLetter.keys).toEqual(['F1', 'F2', 'P'])
    expect(toggleSpammerKey(withLetter, 'F1').keys).toEqual(['F2', 'P'])
  })
})

describe('withSpammerPatch', () => {
  it('merges patch into persisted config', () => {
    const next = withSpammerPatch(mergeSpammerConfig(), { keys: ['F3', 'F4'] })
    expect(next.keys).toEqual(['F3', 'F4'])
  })
})

describe('formatSpammerKeys', () => {
  it('formats empty and non-empty lists', () => {
    expect(formatSpammerKeys([])).toBe('—')
    expect(formatSpammerKeys(['F1', 'F2'])).toBe('F1 · F2')
  })
})
