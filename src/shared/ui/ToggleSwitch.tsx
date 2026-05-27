interface ToggleSwitchProps {
  checked: boolean
  disabled?: boolean
  onChange: (checked: boolean) => void
  tone?: 'emerald' | 'amber'
}

export function ToggleSwitch({
  checked,
  disabled = false,
  onChange,
  tone = 'emerald',
}: ToggleSwitchProps) {
  const onClass =
    tone === 'emerald'
      ? 'bg-emerald-500/80 border-emerald-400/50'
      : 'bg-amber-500/80 border-amber-400/50'

  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      disabled={disabled}
      onClick={() => onChange(!checked)}
      className={`relative w-9 h-5 rounded-full border transition-colors shrink-0 disabled:opacity-50 disabled:cursor-not-allowed ${
        checked ? onClass : 'bg-zinc-800 border-zinc-700/80'
      }`}
    >
      <span
        className={`absolute top-0.5 left-0.5 w-3.5 h-3.5 rounded-full bg-white shadow transition-transform ${
          checked ? 'translate-x-4' : 'translate-x-0'
        }`}
      />
    </button>
  )
}
