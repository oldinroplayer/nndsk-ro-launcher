import type { DependencyStatus, YdotoolInputStatus } from './types'

export function ydotoolInputFromDeps(
  deps: DependencyStatus,
): YdotoolInputStatus {
  return {
    ydotoolInputOk: deps.ydotoolInputOk,
    ydotoolInputWarning: deps.ydotoolInputWarning,
  }
}
