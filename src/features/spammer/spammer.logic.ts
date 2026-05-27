import type { SpammerConfig } from '../../shared/types'
import { DEFAULT_SPAMMER_CONFIG } from '../../shared/constants'

export function mergeSpammerConfig(config?: SpammerConfig): SpammerConfig {
  return {
    ...DEFAULT_SPAMMER_CONFIG,
    ...config,
    enabled: false,
  }
}

export type PersistedSpammerPatch = Partial<Omit<SpammerConfig, 'enabled'>>

export function withSpammerPatch(
  config: SpammerConfig,
  patch: PersistedSpammerPatch,
): SpammerConfig {
  return mergeSpammerConfig({ ...config, ...patch })
}
