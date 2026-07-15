import { CheckCircle2, X } from 'lucide-react'

import type { StorageNotice } from '../shared/types'

interface MaintenanceNoticeProps {
  notices: StorageNotice[]
  onDismiss: () => void
}

export function MaintenanceNotice({
  notices,
  onDismiss,
}: MaintenanceNoticeProps) {
  if (notices.length === 0) return null

  return (
    <div
      className="mx-3 mt-3 flex shrink-0 items-center gap-3 rounded-lg border border-amber-500/30 bg-amber-500/10 px-3 py-2 text-amber-100"
      role="status"
    >
      <CheckCircle2 className="h-4 w-4 shrink-0 text-amber-400" aria-hidden />
      <div className="min-w-0 flex-1">
        <p className="text-xs font-semibold">Mantenimiento completado</p>
        <p className="truncate text-[10px] text-amber-200/70">
          {notices.map((notice) => notice.message).join(' · ')}
        </p>
      </div>
      <button
        type="button"
        onClick={onDismiss}
        aria-label="Descartar aviso"
        className="flex h-6 w-6 items-center justify-center rounded text-amber-300/70 transition-colors hover:bg-amber-500/10 hover:text-amber-100"
      >
        <X className="h-3.5 w-3.5" aria-hidden />
      </button>
    </div>
  )
}
