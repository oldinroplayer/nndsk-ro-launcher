import { runSafely } from '../async'

/** Persiste vía API y sincroniza el campo `error` del store. */
export async function persistWithError(
  setError: (error: string | null) => void,
  save: () => Promise<void>,
  options?: { rethrow?: boolean },
): Promise<boolean> {
  const result = await runSafely(save)
  if (!result.ok) {
    setError(result.error)
    if (options?.rethrow) throw new Error(result.error)
    return false
  }
  setError(null)
  return true
}
