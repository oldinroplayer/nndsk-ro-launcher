import { useCallback, useState } from 'react'
import { toErrorMessage } from '../errors'

export type { AsyncResult } from '../async'
export { runSafely } from '../async'

/** Estado de busy/error para acciones async con claves opcionales. */
export function useAsyncAction<K extends string>() {
  const [busyKey, setBusyKey] = useState<K | null>(null)
  const [error, setError] = useState<string | null>(null)

  const run = useCallback(
    async (key: K, fn: () => Promise<void>): Promise<boolean> => {
      setBusyKey(key)
      setError(null)
      try {
        await fn()
        return true
      } catch (err) {
        setError(toErrorMessage(err))
        return false
      } finally {
        setBusyKey(null)
      }
    },
    [],
  )

  const isBusy = useCallback((key: K) => busyKey === key, [busyKey])

  return { error, setError, run, isBusy, busyKey }
}
