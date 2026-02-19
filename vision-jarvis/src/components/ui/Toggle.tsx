interface ToggleProps {
  enabled: boolean
  onChange: (enabled: boolean) => void
  size?: 'sm' | 'lg'
}

export function Toggle({ enabled, onChange, size = 'sm' }: ToggleProps) {
  const cls = size === 'lg' ? 'w-16 h-8 px-1' : 'w-12 h-6 px-0.5'
  const ball = size === 'lg' ? 'w-6 h-6' : 'w-5 h-5'
  return (
    <div
      onClick={() => onChange(!enabled)}
      className={`${cls} rounded-full flex items-center cursor-pointer transition-all duration-300 ${
        enabled ? 'gradient-success justify-end' : 'bg-secondary'
      }`}
    >
      <div className={`${ball} bg-white rounded-full transition-all duration-300 pointer-events-none`} />
    </div>
  )
}
