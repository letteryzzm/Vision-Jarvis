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

  return (
    <div className="flex h-screen bg-app">
      {/* Left Sidebar */}
      <div className="w-80 bg-sidebar border-r border-primary p-6 flex flex-col gap-6 overflow-y-auto custom-scrollbar">
        {/* Memory Toggle */}
        <div className="flex items-center justify-between">
          <span className="text-sm font-medium text-secondary">全局记忆</span>
          <Toggle
            enabled={settings?.memory_enabled ?? false}
            onChange={handleMemoryToggle}
            size="lg"
          />
        </div>

        {/* Date Selector */}
        <div>
          <button className="w-full h-12 bg-input rounded-xl border border-secondary hover:border-glow transition-colors flex items-center justify-between px-4">
            <span className="text-sm text-primary">{new Date().toISOString().slice(0, 10)}</span>
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#00D4FF" strokeWidth="2">
              <rect x="3" y="4" width="18" height="18" rx="2" ry="2"/>
              <line x1="16" y1="2" x2="16" y2="6"/>
              <line x1="8" y1="2" x2="8" y2="6"/>
              <line x1="3" y1="10" x2="21" y2="10"/>
            </svg>
          </button>
        </div>

        {/* Short-term Memory List */}
        <div className="flex-1 flex flex-col gap-4">
          <h3 className="text-sm font-semibold text-primary">短期记忆</h3>
          <div className="flex flex-col gap-2">
            <div className="text-xs text-muted px-2 py-1">早晨</div>
            <div className="memory-item p-3 bg-item rounded-lg hover:bg-secondary cursor-pointer transition-colors">
              <div className="text-xs text-info mb-1">08:00-09:30</div>
              <div className="text-sm text-primary">开发 Vision-Jarvis 项目</div>
            </div>
            <div className="text-xs text-muted px-2 py-1 mt-2">下午</div>
            <div className="memory-item p-3 bg-item rounded-lg hover:bg-secondary cursor-pointer transition-colors">
              <div className="text-xs text-info mb-1">14:00-15:30</div>
              <div className="text-sm text-primary">设计前端架构</div>
            </div>
          </div>
        </div>

        {/* Settings */}
        <div className="border-t border-primary pt-4 space-y-4">
          <div>
            <div className="flex justify-between text-xs mb-2">
              <span className="text-secondary">录制分段</span>
              <span className="text-info">{Math.floor(interval / 60)}分钟</span>
            </div>
            <input
              type="range"
              min="30"
              max="300"
              step="30"
              value={interval}
              onChange={(e) => handleIntervalChange(parseInt(e.target.value))}
              className="w-full"
            />
          </div>
          <button className="w-full h-10 bg-input rounded-lg border border-secondary hover:border-glow transition-colors text-sm text-primary">
            文件存储设置
          </button>
        </div>
      </div>

      {/* Right Main Content */}
      <div className="flex-1 flex flex-col">
        <div className="h-20 px-8 flex items-center border-b border-primary">
          <div className="flex-1 relative">
            <input
              type="text"
              placeholder="搜索记忆..."
              className="w-full h-12 pl-12 pr-4 bg-input rounded-full border border-secondary focus:border-glow outline-none text-sm text-primary placeholder:text-placeholder"
            />
            <svg className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="#888899" strokeWidth="2">
              <circle cx="11" cy="11" r="8"/>
              <path d="m21 21-4.35-4.35"/>
            </svg>
          </div>
        </div>

        <div className="flex-1 overflow-y-auto custom-scrollbar p-8 flex items-center justify-center">
          <div className="text-center">
            <div className="text-6xl mb-4">🧠</div>
            <h2 className="text-2xl font-semibold text-primary mb-2">想找哪段记忆</h2>
            <p className="text-lg text-muted">我都记着呢，随便问</p>
          </div>
        </div>
      </div>
    </div>
  )
}
