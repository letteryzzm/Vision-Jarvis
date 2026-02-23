import type { ProviderType } from './tauri-api'

export interface ProviderRegistryEntry {
  id: string
  name: string
  apiBaseUrl: string
  providerType: ProviderType
  models: string[]
  videoModels?: string[]
  isThirdParty: boolean
}

export const PROVIDER_REGISTRY: ProviderRegistryEntry[] = [
  {
    id: 'openai',
    name: 'OpenAI',
    apiBaseUrl: 'https://api.openai.com',
    providerType: 'OpenAI',
    models: ['gpt-4o', 'gpt-4o-mini', 'gpt-4-turbo', 'gpt-3.5-turbo'],
    isThirdParty: false,
  },
  {
    id: 'claude',
    name: 'Claude',
    apiBaseUrl: 'https://api.anthropic.com',
    providerType: 'Claude',
    models: ['claude-opus-4-6', 'claude-sonnet-4-5', 'claude-3-5-sonnet-20241022'],
    isThirdParty: false,
  },
  {
    id: 'gemini',
    name: 'Gemini',
    apiBaseUrl: 'https://generativelanguage.googleapis.com',
    providerType: 'Gemini',
    models: ['gemini-3-flash-preview'],
    isThirdParty: false,
  },
  {
    id: 'qwen',
    name: 'Qwen',
    apiBaseUrl: 'https://dashscope.aliyuncs.com/compatible-mode',
    providerType: 'Qwen',
    models: ['qwen3-vl-plus', 'qwen3-vl-flash-2026-01-22'],
    isThirdParty: false,
  },
  {
    id: 'siliconflow',
    name: 'SiliconFlow',
    apiBaseUrl: 'https://api.siliconflow.cn',
    providerType: 'SiliconFlow',
    models: ['Pro/MiniMaxAI/MiniMax-M2.5', 'Pro/zai-org/GLM-4.7', 'Qwen/Qwen3-VL-8B-Instruct'],
    videoModels: ['Qwen/Qwen3-VL-8B-Instruct', 'Pro/zai-org/GLM-4.7'],
    isThirdParty: true,
  },
]
