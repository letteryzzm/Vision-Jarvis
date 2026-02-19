import { useStore } from '@nanostores/react'
import { useState, useEffect, useRef } from 'react'
import {
  $settings, loadSettings, toggleAutoStart, updateLaunchText,
  toggleMorningReminder, toggleWaterReminder, toggleSedentaryReminder,
  toggleScreenInactivityReminder, updateSettings,
} from '@/stores/settingsStore'
import { TauriAPI } from '@/lib/tauri-api'
import type { AIConfig, AIProviderConfig } from '@/lib/tauri-api'
import { Toggle } from '@/components/ui/Toggle'
import { showNotification } from '@/lib/utils'

const INPUT = 'w-full h-12 px-4 bg-input rounded-xl border border-secondary focus:border-glow outline-none text-sm text-primary'
const TEXTAREA = 'w-full p-4 bg-input rounded-xl border border-secondary focus:border-glow outline-none text-sm text-primary resize-none'
const SAVE_BTN = 'px-4 py-2 bg-input rounded-lg border border-secondary hover:border-glow transition-colors text-sm text-primary'
const CARD = 'p-8 bg-card rounded-[24px] border border-primary'

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
          <h2 className="text-2xl font-semibold text-primary mb-2">{title}</h2>
          <p className="text-sm text-muted">{desc}</p>
        </div>
        <Toggle enabled={enabled} onChange={onToggle} size="lg" />
      </div>
      <div className="space-y-4">
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
  const [selectedProvider, setSelectedProvider] = useState<string | null>(null)
  const [openaiKey, setOpenaiKey] = useState('')
  const [openaiModel, setOpenaiModel] = useState('gpt-4o')
  const [claudeKey, setClaudeKey] = useState('')
  const [claudeModel, setClaudeModel] = useState('claude-sonnet-4-5-20250514')
  const [customUrl, setCustomUrl] = useState('')
  const [customKey, setCustomKey] = useState('')
  const [customModel, setCustomModel] = useState('')
  const debounceRefs = useRef<Record<string, number>>({})

  useEffect(() => {
    loadSettings()
    TauriAPI.getAIConfig().then(cfg => {
      setAiConfig(cfg)
      cfg.providers.forEach(p => {
        if (p.id === 'openai' || p.name === 'OpenAI') { setOpenaiKey(p.api_key); setOpenaiModel(p.model) }
        else if (p.id === 'claude' || p.name === 'Claude') { setClaudeKey(p.api_key); setClaudeModel(p.model) }
        else { setCustomUrl(p.api_base_url); setCustomKey(p.api_key); setCustomModel(p.model) }
      })
      if (cfg.active_provider_id) {
        const active = cfg.providers.find(p => p.id === cfg.active_provider_id)
        if (active) setSelectedProvider(
          active.id === 'openai' || active.name === 'OpenAI' ? 'openai' :
          active.id === 'claude' || active.name === 'Claude' ? 'claude' : 'custom'
        )
      }
    }).catch(console.error)
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

  function getProviderConfig(): (Omit<AIProviderConfig, 'enabled' | 'is_active'>) | null {
    if (!selectedProvider) { showNotification('请先选择一个提供商类型', 'error'); return null }
    if (selectedProvider === 'openai') {
      if (!openaiKey) { showNotification('请输入 OpenAI API Key', 'error'); return null }
      return { id: 'openai', name: 'OpenAI', api_base_url: 'https://api.openai.com', api_key: openaiKey, model: openaiModel }
    }
    if (selectedProvider === 'claude') {
      if (!claudeKey) { showNotification('请输入 Claude API Key', 'error'); return null }
      return { id: 'claude', name: 'Claude', api_base_url: 'https://api.anthropic.com', api_key: claudeKey, model: claudeModel }
    }
    if (!customUrl.startsWith('http')) { showNotification('API 地址必须以 http:// 或 https:// 开头', 'error'); return null }
    if (!customKey) { showNotification('请输入 API Key', 'error'); return null }
    if (!customModel) { showNotification('请输入模型名称', 'error'); return null }
    return { id: 'custom', name: 'Custom', api_base_url: customUrl, api_key: customKey, model: customModel }
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
      <div className="max-w-5xl mx-auto">
        <div className="mb-12">
          <h1 className="text-4xl font-bold text-primary mb-2">设置</h1>
          <p className="text-lg text-muted">配置您的应用偏好</p>
        </div>

        {/* Tabs */}
        <div className="flex gap-4 mb-8 border-b border-primary">
          {(['general', 'ai'] as const).map(t => (
            <button key={t} onClick={() => setTab(t)}
              className={`px-6 py-3 text-sm font-medium transition-colors border-b-2 ${
                tab === t ? 'border-info text-info' : 'border-transparent text-muted'
              }`}
            >{t === 'general' ? '通用设置' : 'AI 配置'}</button>
          ))}
        </div>

        {/* General Tab */}
        {tab === 'general' && (
          <div className="grid gap-6">
            {/* Card 1: 启动设置 */}
            <div className={CARD}>
              <h2 className="text-2xl font-semibold text-primary mb-6">启动设置</h2>
              <div className="space-y-4">
                <div className="flex items-center justify-between p-4 bg-input rounded-xl">
                  <span className="text-sm text-secondary">开机自动启动</span>
                  <Toggle enabled={s?.auto_start ?? false} onChange={v =>
                    handleToggle(toggleAutoStart, v, v ? '已启用开机自启' : '已禁用开机自启')
                  } />
                </div>
                <div>
                  <label className="text-sm text-secondary block mb-2">启动提醒文本</label>
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
                <label className="text-sm text-secondary block mb-2">触发时间</label>
                <input id="morning-time" type="time" defaultValue={s?.morning_reminder_time ?? '08:00'}
                  className="w-48 h-12 px-4 bg-input rounded-xl border border-secondary focus:border-glow outline-none text-sm text-primary" />
              </div>
              <div>
                <label className="text-sm text-secondary block mb-2">提醒消息</label>
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
                <label className="text-sm text-secondary block mb-3">提醒时间段</label>
                <div className="flex items-center gap-4">
                  <input id="water-start" type="time" defaultValue={s?.water_reminder_start ?? '09:00'}
                    className="flex-1 h-12 px-4 bg-input rounded-xl border border-secondary focus:border-glow outline-none text-sm text-primary" />
                  <span className="text-muted">至</span>
                  <input id="water-end" type="time" defaultValue={s?.water_reminder_end ?? '21:00'}
                    className="flex-1 h-12 px-4 bg-input rounded-xl border border-secondary focus:border-glow outline-none text-sm text-primary" />
                </div>
              </div>
              <WaterIntervalSlider defaultValue={s?.water_reminder_interval_minutes ?? 60} />
              <div>
                <label className="text-sm text-secondary block mb-2">提醒消息</label>
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
                <label className="text-sm text-secondary block mb-3">生效时间段</label>
                <div className="flex items-center gap-4">
                  <input id="sed-start" type="time" defaultValue={s?.sedentary_reminder_start ?? '09:00'}
                    className="flex-1 h-12 px-4 bg-input rounded-xl border border-secondary focus:border-glow outline-none text-sm text-primary" />
                  <span className="text-muted">至</span>
                  <input id="sed-end" type="time" defaultValue={s?.sedentary_reminder_end ?? '21:00'}
                    className="flex-1 h-12 px-4 bg-input rounded-xl border border-secondary focus:border-glow outline-none text-sm text-primary" />
                </div>
              </div>
              <SedentarySlider defaultValue={s?.sedentary_reminder_threshold_minutes ?? 60} />
              <div>
                <label className="text-sm text-secondary block mb-2">提醒消息</label>
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
                <label className="text-sm text-secondary block mb-2">提醒消息</label>
                <textarea id="screen-msg" defaultValue={s?.screen_inactivity_message ?? ''}
                  className={`${TEXTAREA} h-20`} placeholder="留空则使用 AI 智能建议..." />
              </div>
            </ReminderCard>

            {/* Card 6: 存储设置 */}
            <div className={CARD}>
              <h2 className="text-2xl font-semibold text-primary mb-6">存储设置</h2>
              <div className="space-y-4">
                <div>
                  <label className="text-sm text-secondary block mb-2">存储路径</label>
                  <div className="flex gap-2">
                    <input type="text" readOnly value={s?.storage_path ?? ''}
                      className={`${INPUT} flex-1 font-mono`}
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
          <div className="grid gap-6">
            <div className={CARD}>
              <h2 className="text-2xl font-semibold text-primary mb-6">AI 提供商配置</h2>
              <div className="flex gap-3 mb-6">
                {[['openai', 'OpenAI'], ['claude', 'Claude'], ['custom', '三方供应商']].map(([id, label]) => (
                  <button key={id} onClick={() => setSelectedProvider(id)}
                    className={`flex-1 py-3 rounded-xl border text-sm font-medium transition-colors ${
                      selectedProvider === id
                        ? 'gradient-primary text-white border-transparent'
                        : 'bg-input text-muted border-secondary hover:border-glow'
                    }`}
                  >{label}</button>
                ))}
              </div>

              {selectedProvider === 'openai' && (
                <div className="space-y-4">
                  <div>
                    <label className="text-sm text-secondary block mb-2">API Key</label>
                    <input type="password" value={openaiKey} onChange={e => setOpenaiKey(e.target.value)}
                      className={`${INPUT} font-mono`} placeholder="sk-..." />
                  </div>
                  <div>
                    <label className="text-sm text-secondary block mb-2">模型</label>
                    <select value={openaiModel} onChange={e => setOpenaiModel(e.target.value)} className={INPUT}>
                      {['gpt-4o', 'gpt-4o-mini', 'gpt-4-turbo', 'gpt-4', 'gpt-3.5-turbo'].map(m => (
                        <option key={m} value={m}>{m}</option>
                      ))}
                    </select>
                  </div>
                </div>
              )}

              {selectedProvider === 'claude' && (
                <div className="space-y-4">
                  <div>
                    <label className="text-sm text-secondary block mb-2">API Key</label>
                    <input type="password" value={claudeKey} onChange={e => setClaudeKey(e.target.value)}
                      className={`${INPUT} font-mono`} placeholder="sk-ant-..." />
                  </div>
                  <div>
                    <label className="text-sm text-secondary block mb-2">模型</label>
                    <select value={claudeModel} onChange={e => setClaudeModel(e.target.value)} className={INPUT}>
                      {['claude-sonnet-4-5-20250514', 'claude-3-5-sonnet-20241022', 'claude-3-opus-20240229', 'claude-3-haiku-20240307'].map(m => (
                        <option key={m} value={m}>{m}</option>
                      ))}
                    </select>
                  </div>
                </div>
              )}

              {selectedProvider === 'custom' && (
                <div className="space-y-4">
                  <div>
                    <label className="text-sm text-secondary block mb-2">API 地址</label>
                    <input type="text" value={customUrl} onChange={e => setCustomUrl(e.target.value)}
                      className={`${INPUT} font-mono`} placeholder="https://api.example.com" />
                    <p className="text-xs text-muted mt-1">OpenAI 兼容格式，会自动添加 /v1/chat/completions</p>
                  </div>
                  <div>
                    <label className="text-sm text-secondary block mb-2">API Key</label>
                    <input type="password" value={customKey} onChange={e => setCustomKey(e.target.value)}
                      className={`${INPUT} font-mono`} placeholder="sk-..." />
                  </div>
                  <div>
                    <label className="text-sm text-secondary block mb-2">模型名称</label>
                    <input type="text" value={customModel} onChange={e => setCustomModel(e.target.value)}
                      className={`${INPUT} font-mono`} placeholder="输入模型名称" />
                  </div>
                </div>
              )}

              {selectedProvider && (
                <div className="flex gap-2 pt-4">
                  <button onClick={saveProvider}
                    className="flex-1 py-3 gradient-primary rounded-xl text-white font-medium text-sm hover:opacity-90 transition-opacity">
                    保存配置
                  </button>
                  <button onClick={testProvider}
                    className="py-3 px-6 bg-input rounded-xl text-white font-medium text-sm hover:bg-secondary transition-colors">
                    测试连接
                  </button>
                </div>
              )}
            </div>

            <div className={CARD}>
              <h2 className="text-2xl font-semibold text-primary mb-4">当前状态</h2>
              <AIStatus config={aiConfig} />
            </div>
          </div>
        )}
      </div>
    </div>
  )
}

function WaterIntervalSlider({ defaultValue }: { defaultValue: number }) {
  const [value, setValue] = useState(defaultValue)
  return (
    <div>
      <div className="flex justify-between text-sm mb-3">
        <span className="text-secondary">提醒间隔</span>
        <span className="text-info">每 {value} 分钟</span>
      </div>
      <input type="range" min="15" max="180" step="15" value={value}
        onChange={e => {
          const v = parseInt(e.target.value)
          setValue(v)
          updateSettings({ water_reminder_interval_minutes: v }).catch(console.error)
        }}
        className="w-full" />
      <div className="flex justify-between text-xs text-muted mt-1"><span>15分钟</span><span>180分钟</span></div>
    </div>
  )
}

function SedentarySlider({ defaultValue }: { defaultValue: number }) {
  const [value, setValue] = useState(defaultValue)
  return (
    <div>
      <div className="flex justify-between text-sm mb-3">
        <span className="text-secondary">久坐阈值</span>
        <span className="text-info">{value} 分钟</span>
      </div>
      <input type="range" min="15" max="180" step="15" value={value}
        onChange={e => {
          const v = parseInt(e.target.value)
          setValue(v)
          updateSettings({ sedentary_reminder_threshold_minutes: v }).catch(console.error)
        }}
        className="w-full" />
      <div className="flex justify-between text-xs text-muted mt-1"><span>15分钟</span><span>180分钟</span></div>
    </div>
  )
}

function ScreenInactivitySlider({ defaultValue }: { defaultValue: number }) {
  const [value, setValue] = useState(defaultValue)
  return (
    <div>
      <div className="flex justify-between text-sm mb-3">
        <span className="text-secondary">无变化阈值</span>
        <span className="text-info">{value} 分钟</span>
      </div>
      <input type="range" min="5" max="60" step="5" value={value}
        onChange={e => {
          const v = parseInt(e.target.value)
          setValue(v)
          updateSettings({ screen_inactivity_minutes: v }).catch(console.error)
        }}
        className="w-full" />
      <div className="flex justify-between text-xs text-muted mt-1"><span>5分钟</span><span>60分钟</span></div>
    </div>
  )
}

function StorageLimitSlider({ defaultValue }: { defaultValue: number }) {
  const [value, setValue] = useState(defaultValue)
  return (
    <div>
      <div className="flex justify-between text-sm mb-3">
        <span className="text-secondary">存储容量限制</span>
        <span className="text-info">{value} MB</span>
      </div>
      <input type="range" min="512" max="10240" step="512" value={value}
        onChange={e => {
          const v = parseInt(e.target.value)
          setValue(v)
          updateSettings({ storage_limit_mb: v }).catch(console.error)
        }}
        className="w-full" />
      <div className="flex justify-between text-xs text-muted mt-1"><span>512MB</span><span>10GB</span></div>
    </div>
  )
}

function AIStatus({ config }: { config: AIConfig | null }) {
  if (!config || config.providers.length === 0)
    return <p className="text-sm text-muted">未配置任何 AI 提供商</p>
  const active = config.providers.find(p => p.id === config.active_provider_id)
  if (!active) return <p className="text-sm text-muted">已配置提供商但未激活</p>
  const masked = active.api_key.length > 8
    ? active.api_key.slice(0, 4) + '****' + active.api_key.slice(-4)
    : '****'
  return (
    <div className="text-sm">
      <div className="flex items-center gap-2 mb-2">
        <div className="w-2 h-2 rounded-full bg-green-400" />
        <span className="text-primary font-medium">{active.name}</span>
        <span className="text-xs text-success">[活跃]</span>
      </div>
      <div className="text-xs text-muted space-y-1">
        <p>模型: {active.model}</p>
        <p>Key: {masked}</p>
        {active.api_base_url && <p>API: {active.api_base_url}</p>}
      </div>
    </div>
  )
}
