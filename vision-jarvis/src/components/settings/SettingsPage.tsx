import { useStore } from '@nanostores/react'
import { useState, useEffect, useRef } from 'react'
import {
  $settings, loadSettings, toggleAutoStart, updateLaunchText,
  toggleMorningReminder, toggleWaterReminder, toggleSedentaryReminder,
  toggleScreenInactivityReminder, updateSettings,
} from '@/stores/settingsStore'
import { TauriAPI } from '@/lib/tauri-api'
import type { AIConfig, AIProviderConfig } from '@/lib/tauri-api'
import { PROVIDER_REGISTRY } from '@/lib/provider-registry'
import { Toggle } from '@/components/ui/Toggle'
import { showNotification } from '@/lib/utils'

const INPUT = [
  'w-full h-11 px-4 rounded-xl outline-none text-sm text-primary',
  'bg-input border border-primary',
  'transition-all duration-200 ease-out',
  'focus:border-active focus:bg-secondary',
  'placeholder:text-placeholder',
].join(' ')

const TEXTAREA = [
  'w-full p-4 rounded-xl outline-none text-sm text-primary resize-none',
  'bg-input border border-primary',
  'transition-all duration-200 ease-out',
  'focus:border-active focus:bg-secondary',
  'placeholder:text-placeholder',
].join(' ')

const SAVE_BTN = [
  'px-5 py-2.5 rounded-xl text-sm font-medium',
  'bg-secondary border border-primary text-secondary',
  'hover:bg-hover hover:border-active hover:text-primary',
  'transition-all duration-200 ease-out',
  'active:scale-[0.98]',
].join(' ')

const CARD = [
  'p-8 rounded-2xl border border-primary bg-card',
  'backdrop-blur-sm',
].join(' ')

function ReminderCard({
  title, desc, enabled, onToggle, children, onSave,
}: {
  title: string; desc: string; enabled: boolean
  onToggle: (v: boolean) => void; children: React.ReactNode; onSave: () => void
}) {
  return (
    <div className={CARD}>
      <div className="flex items-start justify-between mb-6">
        <div>
          <h2 className="text-lg font-medium text-primary mb-1">{title}</h2>
          <p className="text-xs text-muted">{desc}</p>
        </div>
        <Toggle enabled={enabled} onChange={onToggle} size="lg" />
      </div>
      <div
        className="space-y-4 overflow-hidden transition-all duration-300 ease-out"
        style={{ opacity: enabled ? 1 : 0.4, pointerEvents: enabled ? 'auto' : 'none' }}
      >
        {children}
        <button onClick={onSave} className={SAVE_BTN}>保存</button>
      </div>
    </div>
  )
}

export function SettingsPage() {
  const settings = useStore($settings)
  const [tab, setTab] = useState<'general' | 'ai'>('general')
  const [aiConfig, setAiConfig] = useState<AIConfig | null>(null)
  const [selectedProviderId, setSelectedProviderId] = useState<string | null>(null)
  const [apiKey, setApiKey] = useState('')
  const [model, setModel] = useState('')
  const [videoModel, setVideoModel] = useState('')
  const debounceRefs = useRef<Record<string, number>>({})

  useEffect(() => {
    loadSettings()
    TauriAPI.getAIConfig().then(cfg => {
      setAiConfig(cfg)
      if (cfg.active_provider_id) {
        const active = cfg.providers.find(p => p.id === cfg.active_provider_id)
        if (active) {
          setSelectedProviderId(active.id)
          setApiKey(active.api_key)
          setModel(active.model)
          setVideoModel((active as AIProviderConfig).video_model ?? '')
        }
      }
    }).catch(() => {})
  }, [])

  function debounce(key: string, fn: () => void, ms = 500) {
    if (debounceRefs.current[key]) clearTimeout(debounceRefs.current[key])
    debounceRefs.current[key] = window.setTimeout(fn, ms)
  }

  async function handleToggle(fn: (v: boolean) => Promise<void>, enabled: boolean, successMsg: string) {
    try { await fn(enabled); showNotification(successMsg, 'success') }
    catch (err) { showNotification('切换失败: ' + err, 'error') }
  }

  async function handleSave(fn: () => Promise<void>, successMsg: string) {
    try { await fn(); showNotification(successMsg, 'success') }
    catch (err) { showNotification('保存失败: ' + err, 'error') }
  }

  function handleProviderSelect(registryId: string) {
    setSelectedProviderId(registryId)
    const entry = PROVIDER_REGISTRY.find(p => p.id === registryId)
    if (!entry) return
    const saved = aiConfig?.providers.find(p => p.id === registryId)
    if (saved) {
      setApiKey(saved.api_key)
      setModel(saved.model)
      setVideoModel(saved.video_model ?? '')
    } else {
      setApiKey('')
      setModel(entry.models[0] ?? '')
      setVideoModel(entry.videoModels?.[0] ?? '')
    }
  }

  function getProviderConfig(): (Omit<AIProviderConfig, 'enabled' | 'is_active'>) | null {
    if (!selectedProviderId) { showNotification('请先选择一个提供商', 'error'); return null }
    const entry = PROVIDER_REGISTRY.find(p => p.id === selectedProviderId)
    if (!entry) { showNotification('未知的提供商', 'error'); return null }
    if (!apiKey) { showNotification(`请输入 ${entry.name} API Key`, 'error'); return null }
    if (!model) { showNotification('请选择模型', 'error'); return null }
    return {
      id: entry.id,
      name: entry.name,
      api_base_url: entry.apiBaseUrl,
      api_key: apiKey,
      model,
      provider_type: entry.providerType,
      video_model: entry.isThirdParty && videoModel ? videoModel : null,
    }
  }

  async function saveProvider() {
    const data = getProviderConfig()
    if (!data) return
    try {
      await TauriAPI.updateAIProviderConfig({ ...data, enabled: true, is_active: true })
      await TauriAPI.setActiveAIProvider(data.id)
      const cfg = await TauriAPI.getAIConfig()
      setAiConfig(cfg)
      showNotification(`已保存 ${data.name} 配置`, 'success')
    } catch (err) { showNotification('保存失败: ' + err, 'error') }
  }

  async function testProvider() {
    const data = getProviderConfig()
    if (!data) return
    try {
      showNotification('正在测试连接...', 'info')
      await TauriAPI.updateAIProviderConfig({ ...data, enabled: true, is_active: false })
      const result = await TauriAPI.testAIConnection(data.id)
      showNotification('连接成功: ' + result, 'success')
    } catch (err) { showNotification('连接失败: ' + err, 'error') }
  }

  const s = settings

  return (
    <div className="min-h-screen bg-app p-12">
      <div className="max-w-4xl mx-auto">
        {/* Header */}
        <div className="mb-10">
          <h1 className="text-3xl font-semibold text-primary mb-1 tracking-tight">设置</h1>
          <p className="text-sm text-muted">配置您的应用偏好</p>
        </div>

        {/* Tabs */}
        <div className="flex gap-1 mb-8 p-1 rounded-xl bg-secondary w-fit">
          {(['general', 'ai'] as const).map(t => (
            <button key={t} onClick={() => setTab(t)}
              className={[
                'px-5 py-2 rounded-lg text-sm font-medium',
                'transition-all duration-200 ease-out',
                tab === t
                  ? 'bg-white/90 text-black shadow-sm'
                  : 'text-muted hover:text-secondary',
              ].join(' ')}
            >{t === 'general' ? '通用设置' : 'AI 配置'}</button>
          ))}
        </div>

        {/* General Tab */}
        {tab === 'general' && (
          <div className="grid gap-4">
            {/* Card 1: 启动设置 */}
            <div className={CARD}>
              <h2 className="text-lg font-medium text-primary mb-5">启动设置</h2>
              <div className="space-y-4">
                <div className="flex items-center justify-between py-3 px-4 bg-secondary rounded-xl">
                  <span className="text-sm text-secondary">开机自动启动</span>
                  <Toggle enabled={s?.auto_start ?? false} onChange={v =>
                    handleToggle(toggleAutoStart, v, v ? '已启用开机自启' : '已禁用开机自启')
                  } />
                </div>
                <div>
                  <label className="text-xs text-muted block mb-2 uppercase tracking-wider">启动提醒文本</label>
                  <textarea defaultValue={s?.app_launch_text ?? ''} id="launch-text"
                    className={`${TEXTAREA} h-24`} placeholder="输入启动提醒文本..." />
                  <button className={`mt-2 ${SAVE_BTN}`} onClick={() => {
                    const el = document.getElementById('launch-text') as HTMLTextAreaElement
                    handleSave(() => updateLaunchText(el.value), '启动文本已保存')
                  }}>保存文本</button>
                </div>
              </div>
            </div>

            {/* Card 2: 早安提醒 */}
            <ReminderCard title="早安提醒" desc="每天定时发送一条激励消息"
              enabled={s?.morning_reminder_enabled ?? false}
              onToggle={v => handleToggle(toggleMorningReminder, v, v ? '已启用早安提醒' : '已禁用早安提醒')}
              onSave={() => {
                const time = (document.getElementById('morning-time') as HTMLInputElement).value
                const msg = (document.getElementById('morning-msg') as HTMLTextAreaElement).value
                handleSave(() => updateSettings({ morning_reminder_time: time, morning_reminder_message: msg }), '早安提醒已保存')
              }}
            >
              <div>
                <label className="text-xs text-muted block mb-2 uppercase tracking-wider">触发时间</label>
                <input id="morning-time" type="time" defaultValue={s?.morning_reminder_time ?? '08:00'}
                  className="w-40 h-11 px-4 bg-input rounded-xl border border-primary focus:border-active outline-none text-sm text-primary transition-all duration-200" />
              </div>
              <div>
                <label className="text-xs text-muted block mb-2 uppercase tracking-wider">提醒消息</label>
                <textarea id="morning-msg" defaultValue={s?.morning_reminder_message ?? ''}
                  className={`${TEXTAREA} h-20`} placeholder="输入早安提醒消息..." />
              </div>
            </ReminderCard>

            {/* Card 3: 喝水提醒 */}
            <ReminderCard title="喝水提醒" desc="在工作时间段内定期提醒喝水"
              enabled={s?.water_reminder_enabled ?? false}
              onToggle={v => handleToggle(toggleWaterReminder, v, v ? '已启用喝水提醒' : '已禁用喝水提醒')}
              onSave={() => {
                const start = (document.getElementById('water-start') as HTMLInputElement).value
                const end = (document.getElementById('water-end') as HTMLInputElement).value
                const msg = (document.getElementById('water-msg') as HTMLTextAreaElement).value
                handleSave(() => updateSettings({ water_reminder_start: start, water_reminder_end: end, water_reminder_message: msg }), '喝水提醒已保存')
              }}
            >
              <div>
                <label className="text-xs text-muted block mb-2 uppercase tracking-wider">提醒时间段</label>
                <div className="flex items-center gap-3">
                  <input id="water-start" type="time" defaultValue={s?.water_reminder_start ?? '09:00'}
                    className="flex-1 h-11 px-4 bg-input rounded-xl border border-primary focus:border-active outline-none text-sm text-primary transition-all duration-200" />
                  <span className="text-muted text-sm">—</span>
                  <input id="water-end" type="time" defaultValue={s?.water_reminder_end ?? '21:00'}
                    className="flex-1 h-11 px-4 bg-input rounded-xl border border-primary focus:border-active outline-none text-sm text-primary transition-all duration-200" />
                </div>
              </div>
              <WaterIntervalSlider defaultValue={s?.water_reminder_interval_minutes ?? 60} />
              <div>
                <label className="text-xs text-muted block mb-2 uppercase tracking-wider">提醒消息</label>
                <textarea id="water-msg" defaultValue={s?.water_reminder_message ?? ''}
                  className={`${TEXTAREA} h-20`} placeholder="输入喝水提醒消息..." />
              </div>
            </ReminderCard>

            {/* Card 4: 久坐提醒 */}
            <ReminderCard title="久坐提醒" desc="连续工作超过阈值时提醒休息"
              enabled={s?.sedentary_reminder_enabled ?? false}
              onToggle={v => handleToggle(toggleSedentaryReminder, v, v ? '已启用久坐提醒' : '已禁用久坐提醒')}
              onSave={() => {
                const start = (document.getElementById('sed-start') as HTMLInputElement).value
                const end = (document.getElementById('sed-end') as HTMLInputElement).value
                const msg = (document.getElementById('sed-msg') as HTMLTextAreaElement).value
                handleSave(() => updateSettings({ sedentary_reminder_start: start, sedentary_reminder_end: end, sedentary_reminder_message: msg }), '久坐提醒已保存')
              }}
            >
              <div>
                <label className="text-xs text-muted block mb-2 uppercase tracking-wider">生效时间段</label>
                <div className="flex items-center gap-3">
                  <input id="sed-start" type="time" defaultValue={s?.sedentary_reminder_start ?? '09:00'}
                    className="flex-1 h-11 px-4 bg-input rounded-xl border border-primary focus:border-active outline-none text-sm text-primary transition-all duration-200" />
                  <span className="text-muted text-sm">—</span>
                  <input id="sed-end" type="time" defaultValue={s?.sedentary_reminder_end ?? '21:00'}
                    className="flex-1 h-11 px-4 bg-input rounded-xl border border-primary focus:border-active outline-none text-sm text-primary transition-all duration-200" />
                </div>
              </div>
              <SedentarySlider defaultValue={s?.sedentary_reminder_threshold_minutes ?? 60} />
              <div>
                <label className="text-xs text-muted block mb-2 uppercase tracking-wider">提醒消息</label>
                <textarea id="sed-msg" defaultValue={s?.sedentary_reminder_message ?? ''}
                  className={`${TEXTAREA} h-20`} placeholder="输入久坐提醒消息..." />
              </div>
            </ReminderCard>

            {/* Card 5: 屏幕无变化提醒 */}
            <ReminderCard title="屏幕无变化提醒" desc="检测屏幕长时间无变化时提醒"
              enabled={s?.screen_inactivity_reminder_enabled ?? false}
              onToggle={v => handleToggle(toggleScreenInactivityReminder, v, v ? '已启用屏幕无变化提醒' : '已禁用屏幕无变化提醒')}
              onSave={() => {
                const msg = (document.getElementById('screen-msg') as HTMLTextAreaElement).value
                handleSave(() => updateSettings({ screen_inactivity_message: msg }), '屏幕无变化提醒已保存')
              }}
            >
              <ScreenInactivitySlider defaultValue={s?.screen_inactivity_minutes ?? 10} />
              <div>
                <label className="text-xs text-muted block mb-2 uppercase tracking-wider">提醒消息</label>
                <textarea id="screen-msg" defaultValue={s?.screen_inactivity_message ?? ''}
                  className={`${TEXTAREA} h-20`} placeholder="留空则使用 AI 智能建议..." />
              </div>
            </ReminderCard>

            {/* Card 6: 存储设置 */}
            <div className={CARD}>
              <h2 className="text-lg font-medium text-primary mb-5">存储设置</h2>
              <div className="space-y-4">
                <div>
                  <label className="text-xs text-muted block mb-2 uppercase tracking-wider">存储路径</label>
                  <div className="flex gap-2">
                    <input type="text" readOnly value={s?.storage_path ?? ''}
                      className={`${INPUT} flex-1 font-mono text-xs`}
                      placeholder="~/Library/Application Support/vision-jarvis/screenshots" />
                    <button className={SAVE_BTN} onClick={() => s && TauriAPI.openFolder(s.storage_path)}>打开</button>
                  </div>
                </div>
                <StorageLimitSlider defaultValue={s?.storage_limit_mb ?? 1024} />
              </div>
            </div>
          </div>
        )}

        {/* AI Tab */}
        {tab === 'ai' && (
          <div className="grid gap-4">
            <div className={CARD}>
              <h2 className="text-lg font-medium text-primary mb-5">AI 提供商</h2>

              {/* Provider chips */}
              <div className="flex flex-wrap gap-2 mb-6">
                {PROVIDER_REGISTRY.map(entry => {
                  const isActive = selectedProviderId === entry.id
                  return (
                    <button key={entry.id} onClick={() => handleProviderSelect(entry.id)}
                      className={[
                        'px-4 py-2 rounded-xl text-sm font-medium',
                        'transition-all duration-200 ease-out',
                        'active:scale-[0.97]',
                        isActive
                          ? 'bg-white/92 text-black border border-transparent shadow-sm'
                          : 'bg-transparent text-secondary border border-primary hover:border-active hover:text-primary',
                      ].join(' ')}
                    >
                      {entry.name}{entry.isThirdParty ? ' ·' : ''}
                    </button>
                  )
                })}
              </div>

              {selectedProviderId && (() => {
                const entry = PROVIDER_REGISTRY.find(p => p.id === selectedProviderId)
                if (!entry) return null
                return (
                  <div className="space-y-4">
                    <div>
                      <label className="text-xs text-muted block mb-2 uppercase tracking-wider">API Key</label>
                      <input type="password" value={apiKey} onChange={e => setApiKey(e.target.value)}
                        className={`${INPUT} font-mono`} placeholder={`输入 ${entry.name} API Key`} />
                    </div>
                    <div>
                      <label className="text-xs text-muted block mb-2 uppercase tracking-wider">
                        {entry.isThirdParty ? '语言模型' : '模型'}
                      </label>
                      <select value={model} onChange={e => setModel(e.target.value)}
                        className={`${INPUT} appearance-none`}>
                        {entry.models.map(m => (
                          <option key={m} value={m}>{m}</option>
                        ))}
                      </select>
                    </div>
                    {entry.isThirdParty && entry.videoModels && (
                      <div>
                        <label className="text-xs text-muted block mb-2 uppercase tracking-wider">视频模型</label>
                        <select value={videoModel} onChange={e => setVideoModel(e.target.value)}
                          className={`${INPUT} appearance-none`}>
                          {entry.videoModels.map(m => (
                            <option key={m} value={m}>{m}</option>
                          ))}
                        </select>
                        <p className="text-xs text-muted mt-1.5">用于视频 / 图像分析</p>
                      </div>
                    )}
                    <div className="flex gap-2 pt-2">
                      <button onClick={saveProvider}
                        className={[
                          'flex-1 py-3 rounded-xl text-sm font-medium',
                          'bg-white/92 text-black',
                          'hover:bg-white transition-all duration-200 ease-out',
                          'active:scale-[0.98]',
                        ].join(' ')}>
                        保存配置
                      </button>
                      <button onClick={testProvider}
                        className={[
                          'py-3 px-6 rounded-xl text-sm font-medium',
                          'bg-secondary border border-primary text-secondary',
                          'hover:bg-hover hover:border-active hover:text-primary',
                          'transition-all duration-200 ease-out active:scale-[0.98]',
                        ].join(' ')}>
                        测试连接
                      </button>
                    </div>
                  </div>
                )
              })()}
            </div>

            <div className={CARD}>
              <h2 className="text-lg font-medium text-primary mb-4">当前状态</h2>
              <AIStatus config={aiConfig} />
            </div>
          </div>
        )}
      </div>
    </div>
  )
}

function MonoSlider({
  label, value, min, max, step, unit, onChange,
}: {
  label: string; value: number; min: number; max: number
  step: number; unit: string; onChange: (v: number) => void
}) {
  const pct = ((value - min) / (max - min)) * 100
  return (
    <div>
      <div className="flex justify-between text-xs mb-3">
        <span className="text-muted uppercase tracking-wider">{label}</span>
        <span className="text-secondary tabular-nums">{value} {unit}</span>
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
          type="range" min={min} max={max} step={step} value={value}
          onChange={e => onChange(parseInt(e.target.value))}
          className="mono-slider"
        />
      </div>
      <div className="flex justify-between text-[10px] text-muted mt-1.5">
        <span>{min}{unit}</span>
        <span>{max}{unit}</span>
      </div>
    </div>
  )
}

function WaterIntervalSlider({ defaultValue }: { defaultValue: number }) {
  const [value, setValue] = useState(defaultValue)
  return (
    <MonoSlider label="提醒间隔" value={value} min={15} max={180} step={15} unit="分钟"
      onChange={v => { setValue(v); updateSettings({ water_reminder_interval_minutes: v }).catch(() => {}) }} />
  )
}

function SedentarySlider({ defaultValue }: { defaultValue: number }) {
  const [value, setValue] = useState(defaultValue)
  return (
    <MonoSlider label="久坐阈值" value={value} min={15} max={180} step={15} unit="分钟"
      onChange={v => { setValue(v); updateSettings({ sedentary_reminder_threshold_minutes: v }).catch(() => {}) }} />
  )
}

function ScreenInactivitySlider({ defaultValue }: { defaultValue: number }) {
  const [value, setValue] = useState(defaultValue)
  return (
    <MonoSlider label="无变化阈值" value={value} min={5} max={60} step={5} unit="分钟"
      onChange={v => { setValue(v); updateSettings({ screen_inactivity_minutes: v }).catch(() => {}) }} />
  )
}

function StorageLimitSlider({ defaultValue }: { defaultValue: number }) {
  const [value, setValue] = useState(defaultValue)
  const display = value >= 1024 ? `${(value / 1024).toFixed(1)} GB` : `${value} MB`
  return (
    <div>
      <div className="flex justify-between text-xs mb-3">
        <span className="text-muted uppercase tracking-wider">存储容量限制</span>
        <span className="text-secondary tabular-nums">{display}</span>
      </div>
      <div className="relative py-1">
        <div className="absolute top-1/2 -translate-y-1/2 w-full h-[2px] rounded-full overflow-hidden pointer-events-none">
          <div className="h-full bg-white/10 w-full absolute" />
          <div
            className="h-full bg-white/80 absolute left-0 transition-all duration-150 ease-out"
            style={{ width: `${((value - 512) / (10240 - 512)) * 100}%` }}
          />
        </div>
        <input
          type="range" min={512} max={10240} step={512} value={value}
          onChange={e => {
            const v = parseInt(e.target.value)
            setValue(v)
            updateSettings({ storage_limit_mb: v }).catch(() => {})
          }}
          className="mono-slider"
        />
      </div>
      <div className="flex justify-between text-[10px] text-muted mt-1.5">
        <span>512 MB</span>
        <span>10 GB</span>
      </div>
    </div>
  )
}

function AIStatus({ config }: { config: AIConfig | null }) {
  if (!config || config.providers.length === 0)
    return <p className="text-sm text-muted">未配置任何 AI 提供商</p>
  const active = config.providers.find(p => p.id === config.active_provider_id)
  if (!active) return <p className="text-sm text-muted">已配置提供商但未激活</p>
  const masked = active.api_key.length > 8
    ? active.api_key.slice(0, 4) + '····' + active.api_key.slice(-4)
    : '····'
  return (
    <div className="text-sm">
      <div className="flex items-center gap-2 mb-3">
        <div className="w-1.5 h-1.5 rounded-full bg-white/60" />
        <span className="text-primary font-medium">{active.name}</span>
        <span className="text-xs text-muted border border-primary rounded px-1.5 py-0.5">活跃</span>
      </div>
      <div className="text-xs text-muted space-y-1.5 pl-3.5">
        <p>模型 · {active.model}</p>
        {active.video_model && <p>视频 · {active.video_model}</p>}
        <p>Key · {masked}</p>
        {active.api_base_url && <p>API · {active.api_base_url}</p>}
      </div>
    </div>
  )
}
