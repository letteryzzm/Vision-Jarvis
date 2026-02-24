import { useStore } from '@nanostores/react'
import { useEffect, useRef } from 'react'
import { $settings, loadSettings, toggleMemory, updateCaptureInterval } from '@/stores/settingsStore'
import { Toggle } from '@/components/ui/Toggle'
import { showNotification } from '@/lib/utils'

export function MemoryPage() {
  const settings = useStore($settings)
  const timerRef = useRef<number | null>(null)

  useEffect(() => {
    loadSettings()
  }, [])

  async function handleMemoryToggle(enabled: boolean) {
    try {
      await toggleMemory(enabled)
      showNotification(enabled ? '记忆功能已启用' : '记忆功能已禁用', 'success')
    } catch (err) {
      showNotification('切换失败: ' + err, 'error')
    }
  }

  function handleIntervalChange(value: number) {
    if (timerRef.current !== null) clearTimeout(timerRef.current)
    timerRef.current = window.setTimeout(async () => {
      try {
        await updateCaptureInterval(value)
        showNotification(`录制分段时长已更新为 ${Math.floor(value / 60)} 分钟`, 'success')
      } catch (err) {
        showNotification('更新失败: ' + err, 'error')
      }
    }, 500)
  }

  const interval = settings?.capture_interval_seconds ?? 60
  const pct = ((interval - 30) / (300 - 30)) * 100

  return (
    <div className="flex h-screen bg-app">
      {/* Left Sidebar */}
      <div className="w-72 bg-sidebar border-r border-primary p-5 flex flex-col gap-5 overflow-y-auto custom-scrollbar">

        {/* Memory Toggle */}
        <div className="flex items-center justify-between py-1">
          <span className="text-sm font-medium text-secondary">全局记忆</span>
          <Toggle
            enabled={settings?.memory_enabled ?? false}
            onChange={handleMemoryToggle}
            size="lg"
          />
        </div>

        {/* Divider */}
        <div className="h-px bg-primary" />

        {/* Date Selector */}
        <button className="w-full h-10 bg-input rounded-xl border border-primary hover:border-active transition-all duration-200 ease-out flex items-center justify-between px-3.5 active:scale-[0.99]">
          <span className="text-sm text-secondary">{new Date().toISOString().slice(0, 10)}</span>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" className="text-muted">
            <rect x="3" y="4" width="18" height="18" rx="2" ry="2"/>
            <line x1="16" y1="2" x2="16" y2="6"/>
            <line x1="8" y1="2" x2="8" y2="6"/>
            <line x1="3" y1="10" x2="21" y2="10"/>
          </svg>
        </button>

        {/* Short-term Memory List */}
        <div className="flex-1 flex flex-col gap-3 min-h-0">
          <h3 className="text-xs font-medium text-muted uppercase tracking-wider">短期记忆</h3>
          <div className="flex flex-col gap-1 overflow-y-auto custom-scrollbar -mx-1 px-1">
            <div className="text-[10px] text-muted px-2 py-1 uppercase tracking-wider">早晨</div>
            <div className="memory-item px-3 py-2.5 rounded-lg hover:bg-secondary cursor-pointer transition-all duration-150 ease-out active:scale-[0.99]">
              <div className="text-[10px] text-muted mb-0.5 tabular-nums">08:00 – 09:30</div>
              <div className="text-sm text-secondary">开发 Vision-Jarvis 项目</div>
            </div>
            <div className="text-[10px] text-muted px-2 py-1 mt-1 uppercase tracking-wider">下午</div>
            <div className="memory-item px-3 py-2.5 rounded-lg hover:bg-secondary cursor-pointer transition-all duration-150 ease-out active:scale-[0.99]">
              <div className="text-[10px] text-muted mb-0.5 tabular-nums">14:00 – 15:30</div>
              <div className="text-sm text-secondary">设计前端架构</div>
            </div>
          </div>
        </div>

        {/* Interval Slider */}
        <div className="border-t border-primary pt-4">
          <div className="flex justify-between text-xs mb-3">
            <span className="text-muted uppercase tracking-wider">录制分段</span>
            <span className="text-secondary tabular-nums">{Math.floor(interval / 60)} 分钟</span>
          </div>
          <div className="relative py-1">
            <div className="absolute top-1/2 -translate-y-1/2 w-full h-[2px] rounded-full overflow-hidden pointer-events-none">
              <div className="h-full bg-white/10 w-full absolute" />
              <div
                className="h-full bg-white/80 absolute left-0 transition-all duration-150 ease-out"
                style={{ width: `${pct}%` }}
              />
            </div>
            <input
              type="range" min="30" max="300" step="30" value={interval}
              onChange={e => handleIntervalChange(parseInt(e.target.value))}
              className="mono-slider"
            />
          </div>
          <button className={[
            'mt-3 w-full h-9 rounded-xl text-xs font-medium',
            'bg-transparent border border-primary text-muted',
            'hover:border-active hover:text-secondary',
            'transition-all duration-200 ease-out active:scale-[0.99]',
          ].join(' ')}>
            文件存储设置
          </button>
        </div>
      </div>

      {/* Right Main Content */}
      <div className="flex-1 flex flex-col">
        {/* Search Bar */}
        <div className="h-16 px-6 flex items-center border-b border-primary">
          <div className="flex-1 relative">
            <input
              type="text"
              placeholder="搜索记忆..."
              className={[
                'w-full h-10 pl-10 pr-4 rounded-full outline-none text-sm',
                'bg-input border border-primary text-primary',
                'focus:border-active focus:bg-secondary',
                'placeholder:text-placeholder',
                'transition-all duration-200 ease-out',
              ].join(' ')}
            />
            <svg className="absolute left-3.5 top-1/2 -translate-y-1/2 w-4 h-4 text-muted"
              viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
              <circle cx="11" cy="11" r="8"/>
              <path d="m21 21-4.35-4.35"/>
            </svg>
          </div>
        </div>

        {/* Empty State */}
        <div className="flex-1 overflow-y-auto custom-scrollbar flex items-center justify-center">
          <div className="text-center select-none">
            <div className="text-5xl mb-5 opacity-20">◯</div>
            <h2 className="text-xl font-medium text-primary mb-1.5">想找哪段记忆</h2>
            <p className="text-sm text-muted">我都记着呢，随便问</p>
          </div>
        </div>
      </div>
    </div>
  )
}
