# AI Config API

AI提供商配置管理 API 接口

## 概述

AI Config API 提供多AI提供商配置管理功能，支持 OpenAI、Anthropic、Google AI、Local (Ollama) 和 Custom 提供商的配置和管理。

## 位置

- **Commands 模块**: `src-tauri/src/commands/ai_config.rs`
- **Service 模块**: `src-tauri/src/ai/providers.rs`
- **行数**: 319 lines (commands)

## API 列表

| Command | 功能 | 参数 | 返回 |
|---------|------|------|------|
| `get_ai_config_summary` | 获取配置摘要 | - | `ApiResponse<AIConfigSummary>` |
| `get_ai_config` | 获取完整配置 | - | `ApiResponse<AIConfigCollection>` |
| `update_ai_api_key` | 更新API密钥 | provider, api_key | `ApiResponse<bool>` |
| `update_ai_provider_config` | 更新提供商配置 | provider, config | `ApiResponse<bool>` |
| `set_active_ai_provider` | 设置活动提供商 | provider | `ApiResponse<bool>` |
| `test_ai_connection` | 测试连接 | provider | `ApiResponse<string>` |
| `get_available_ai_providers` | 获取可用提供商 | - | `ApiResponse<AIProvider[]>` |
| `reset_ai_config` | 重置配置 | - | `ApiResponse<bool>` |

---

## get_ai_config_summary

获取AI配置摘要（用于前端显示，不暴露API密钥）。

### 函数签名

```rust
#[tauri::command]
pub async fn get_ai_config_summary(
    state: State<'_, AIConfigState>,
) -> Result<ApiResponse<AIConfigSummary>, String>
```

### 参数

无

### 返回值

```typescript
interface AIConfigSummary {
  active_provider: AIProvider | null;
  providers: ProviderSummary[];
}

interface ProviderSummary {
  provider: AIProvider;
  display_name: string;
  has_api_key: boolean;  // 不暴露实际key
  enabled: boolean;
  model: string;
}

type AIProvider =
  | "OpenAI"
  | "Anthropic"
  | "Google"
  | "Local"
  | "Custom";
```

### 前端调用

```typescript
import { invoke } from '@tauri-apps/api/core';

const getAIConfigSummary = async () => {
  const response = await invoke<ApiResponse<AIConfigSummary>>('get_ai_config_summary');

  if (response.success && response.data) {
    const { active_provider, providers } = response.data;

    console.log(`Active: ${active_provider}`);

    providers.forEach(p => {
      console.log(`${p.display_name}: ${p.enabled ? 'enabled' : 'disabled'}, ` +
                  `key: ${p.has_api_key ? 'set' : 'not set'}, ` +
                  `model: ${p.model}`);
    });
  }
};
```

### 示例响应

```json
{
  "success": true,
  "data": {
    "active_provider": "OpenAI",
    "providers": [
      {
        "provider": "OpenAI",
        "display_name": "OpenAI",
        "has_api_key": true,
        "enabled": true,
        "model": "gpt-4"
      },
      {
        "provider": "Anthropic",
        "display_name": "Anthropic Claude",
        "has_api_key": false,
        "enabled": true,
        "model": "claude-3-5-sonnet-20241022"
      },
      {
        "provider": "Google",
        "display_name": "Google AI",
        "has_api_key": false,
        "enabled": true,
        "model": "gemini-2.0-flash-exp"
      },
      {
        "provider": "Local",
        "display_name": "Ollama (Local)",
        "has_api_key": false,
        "enabled": true,
        "model": "llama2"
      },
      {
        "provider": "Custom",
        "display_name": "Custom Provider",
        "has_api_key": false,
        "enabled": true,
        "model": "custom-model"
      }
    ]
  },
  "error": null
}
```

---

## get_ai_config

获取完整的AI配置（包括API密钥，仅后端使用）。

### 函数签名

```rust
#[tauri::command]
pub async fn get_ai_config(
    state: State<'_, AIConfigState>,
) -> Result<ApiResponse<AIConfigCollection>, String>
```

### 参数

无

### 返回值

```typescript
interface AIConfigCollection {
  configs: Record<AIProvider, AIProviderConfig>;
  active_provider: AIProvider | null;
}

interface AIProviderConfig {
  provider: AIProvider;
  api_key: string | null;  // 遮蔽显示为 "***"
  base_url: string;
  model: string;
  enabled: boolean;
}
```

### 安全注意

- API密钥在序列化时会被遮蔽为 "***"
- 不推荐在前端使用此API，使用 `get_ai_config_summary` 代替

---

## update_ai_api_key

更新指定AI提供商的API密钥。

### 函数签名

```rust
#[tauri::command]
pub async fn update_ai_api_key(
    state: State<'_, AIConfigState>,
    provider: AIProvider,
    api_key: Option<String>,
) -> Result<ApiResponse<bool>, String>
```

### 参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| provider | AIProvider | 是 | AI提供商 |
| api_key | string \| null | 否 | API密钥（null表示清空） |

### 返回值

`boolean` - 更新是否成功

### 前端调用

```typescript
// 设置 OpenAI API key
const setOpenAIKey = async (apiKey: string) => {
  const response = await invoke<ApiResponse<boolean>>('update_ai_api_key', {
    provider: 'OpenAI',
    apiKey
  });

  if (response.success) {
    alert('API密钥已更新');
  } else {
    alert(`更新失败: ${response.error}`);
  }
};

// 清空 API key
const clearAPIKey = async (provider: AIProvider) => {
  const response = await invoke<ApiResponse<boolean>>('update_ai_api_key', {
    provider,
    apiKey: null
  });

  if (response.success) {
    alert('API密钥已清空');
  }
};
```

### 示例响应

**成功**:
```json
{
  "success": true,
  "data": true,
  "error": null
}
```

**失败**:
```json
{
  "success": false,
  "data": null,
  "error": "更新 API 密钥失败: 配置无效"
}
```

---

## update_ai_provider_config

更新指定AI提供商的完整配置。

### 函数签名

```rust
#[tauri::command]
pub async fn update_ai_provider_config(
    state: State<'_, AIConfigState>,
    provider: AIProvider,
    config_update: AIProviderConfig,
) -> Result<ApiResponse<bool>, String>
```

### 参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| provider | AIProvider | 是 | AI提供商 |
| config_update | AIProviderConfig | 是 | 新的配置 |

### 安全验证

- 验证 `config_update.provider` 必须与 `provider` 参数匹配
- 防止配置混淆攻击

### 前端调用

```typescript
// 更新 Anthropic 配置
const updateAnthropicConfig = async () => {
  const config: AIProviderConfig = {
    provider: 'Anthropic',
    api_key: 'sk-ant-...',
    base_url: 'https://api.anthropic.com',
    model: 'claude-3-5-sonnet-20241022',
    enabled: true
  };

  const response = await invoke<ApiResponse<boolean>>('update_ai_provider_config', {
    provider: 'Anthropic',
    configUpdate: config
  });

  if (response.success) {
    alert('配置已更新');
  } else {
    alert(`更新失败: ${response.error}`);
  }
};
```

### 示例响应

**成功**:
```json
{
  "success": true,
  "data": true,
  "error": null
}
```

**失败（provider不匹配）**:
```json
{
  "success": false,
  "data": null,
  "error": "Provider不匹配: 期望 OpenAI，实际配置为 Anthropic"
}
```

---

## set_active_ai_provider

设置活动的AI提供商。

### 函数签名

```rust
#[tauri::command]
pub async fn set_active_ai_provider(
    state: State<'_, AIConfigState>,
    provider: AIProvider,
) -> Result<ApiResponse<bool>, String>
```

### 参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| provider | AIProvider | 是 | 要设置的提供商 |

### 验证规则

- 提供商必须已启用（enabled=true）
- 提供商配置必须有效（有API密钥等）

### 前端调用

```typescript
// 切换到 Anthropic
const switchToAnthropic = async () => {
  const response = await invoke<ApiResponse<boolean>>('set_active_ai_provider', {
    provider: 'Anthropic'
  });

  if (response.success) {
    alert('已切换到 Anthropic Claude');
  } else {
    alert(`切换失败: ${response.error}`);
  }
};

// 带UI反馈的切换
const switchProvider = async (provider: AIProvider) => {
  try {
    const response = await invoke<ApiResponse<boolean>>('set_active_ai_provider', {
      provider
    });

    if (response.success) {
      // 更新UI
      updateActiveProviderUI(provider);
    } else {
      showError(response.error);
    }
  } catch (error) {
    showError('切换失败，请检查网络连接');
  }
};
```

### 示例响应

**成功**:
```json
{
  "success": true,
  "data": true,
  "error": null
}
```

**失败**:
```json
{
  "success": false,
  "data": null,
  "error": "设置提供商失败: 提供商未启用或配置无效"
}
```

---

## test_ai_connection

测试指定AI提供商的连接。

### 函数签名

```rust
#[tauri::command]
pub async fn test_ai_connection(
    state: State<'_, AIConfigState>,
    provider: AIProvider,
) -> Result<ApiResponse<String>, String>
```

### 参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| provider | AIProvider | 是 | 要测试的提供商 |

### 返回值

`string` - 测试结果消息

### 测试��法

每个提供商有不同的测试方法：

| Provider | 测试方法 | 端点 |
|----------|---------|------|
| OpenAI | GET /models | 列出可用模型 |
| Anthropic | POST /v1/messages | 发送测试消息 |
| Google | GET /models?key=... | 列出模型 |
| Local (Ollama) | GET /tags | 列出本地模型 |
| Custom | GET base_url | 基本连接测试 |

### 前端调用

```typescript
// 测试 OpenAI 连接
const testOpenAI = async () => {
  const response = await invoke<ApiResponse<string>>('test_ai_connection', {
    provider: 'OpenAI'
  });

  if (response.success && response.data) {
    alert(response.data); // "OpenAI 连接成功"
  } else if (response.error) {
    alert(`连接失败: ${response.error}`);
  }
};

// 测试所有提供商
const testAllProviders = async () => {
  const providers: AIProvider[] = ['OpenAI', 'Anthropic', 'Google', 'Local', 'Custom'];

  for (const provider of providers) {
    const response = await invoke<ApiResponse<string>>('test_ai_connection', {
      provider
    });

    console.log(`${provider}: ${response.success ? '✓' : '✗'} ${response.data || response.error}`);
  }
};
```

### 示例响应

**成功 (OpenAI)**:
```json
{
  "success": true,
  "data": "OpenAI 连接成功",
  "error": null
}
```

**失败 (API密钥无效)**:
```json
{
  "success": false,
  "data": null,
  "error": "配置无效: 缺少 API 密钥"
}
```

**失败 (网络错误)**:
```json
{
  "success": false,
  "data": null,
  "error": "网络错误: connection refused"
}
```

**失败 (401 Unauthorized)**:
```json
{
  "success": false,
  "data": null,
  "error": "Anthropic 连接失败: 401 - API 密钥无效"
}
```

---

## get_available_ai_providers

获取所有已启用的AI提供商列表。

### 函数签名

```rust
#[tauri::command]
pub async fn get_available_ai_providers(
    state: State<'_, AIConfigState>,
) -> Result<ApiResponse<Vec<AIProvider>>, String>
```

### 参数

无

### 返回值

`AIProvider[]` - 已启用的提供商列表

### 前端调用

```typescript
const getAvailableProviders = async () => {
  const response = await invoke<ApiResponse<AIProvider[]>>('get_available_ai_providers');

  if (response.success && response.data) {
    console.log('Available providers:', response.data);
    // ["OpenAI", "Anthropic", "Local"]
  }
};
```

### 示例响应

```json
{
  "success": true,
  "data": ["OpenAI", "Anthropic", "Google", "Local", "Custom"],
  "error": null
}
```

---

## reset_ai_config

重置AI配置为默认值。

### 函数签名

```rust
#[tauri::command]
pub async fn reset_ai_config(
    state: State<'_, AIConfigState>,
) -> Result<ApiResponse<bool>, String>
```

### 参数

无

### 返回值

`boolean` - 重置是否成功

### 前端调用

```typescript
const resetConfig = async () => {
  if (confirm('确定要重置所有AI配置吗？此操作不可撤销。')) {
    const response = await invoke<ApiResponse<boolean>>('reset_ai_config');

    if (response.success) {
      alert('配置已重置为默认值');
      // 刷新UI
      location.reload();
    } else {
      alert(`重置失败: ${response.error}`);
    }
  }
};
```

### 示例响应

```json
{
  "success": true,
  "data": true,
  "error": null
}
```

---

## 类型定义

完整的 TypeScript 类型定义：

```typescript
// API Response
interface ApiResponse<T> {
  success: boolean;
  data: T | null;
  error: string | null;
}

// AI Provider
type AIProvider =
  | "OpenAI"
  | "Anthropic"
  | "Google"
  | "Local"
  | "Custom";

// AI Config Summary
interface AIConfigSummary {
  active_provider: AIProvider | null;
  providers: ProviderSummary[];
}

interface ProviderSummary {
  provider: AIProvider;
  display_name: string;
  has_api_key: boolean;
  enabled: boolean;
  model: string;
}

// AI Provider Config
interface AIProviderConfig {
  provider: AIProvider;
  api_key: string | null;  // 遮蔽显示
  base_url: string;
  model: string;
  enabled: boolean;
}

// AI Config Collection
interface AIConfigCollection {
  configs: Record<AIProvider, AIProviderConfig>;
  active_provider: AIProvider | null;
}
```

## 默认配置

各提供商的默认配置：

```typescript
const DEFAULT_CONFIGS = {
  OpenAI: {
    base_url: 'https://api.openai.com/v1',
    model: 'gpt-4',
    enabled: true
  },
  Anthropic: {
    base_url: 'https://api.anthropic.com',
    model: 'claude-3-5-sonnet-20241022',
    enabled: true
  },
  Google: {
    base_url: 'https://generativelanguage.googleapis.com/v1',
    model: 'gemini-2.0-flash-exp',
    enabled: true
  },
  Local: {
    base_url: 'http://localhost:11434/api',
    model: 'llama2',
    enabled: true
  },
  Custom: {
    base_url: 'http://localhost:8080',
    model: 'custom-model',
    enabled: true
  }
};
```

## 完整使用示例

### AI配置管理页面

```typescript
import { invoke } from '@tauri-apps/api/core';
import { useState, useEffect } from 'react';

function AIConfigPage() {
  const [summary, setSummary] = useState<AIConfigSummary | null>(null);
  const [activeProvider, setActiveProvider] = useState<AIProvider | null>(null);

  // 加载配置
  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    const response = await invoke<ApiResponse<AIConfigSummary>>('get_ai_config_summary');
    if (response.success && response.data) {
      setSummary(response.data);
      setActiveProvider(response.data.active_provider);
    }
  };

  // 更新API密钥
  const updateAPIKey = async (provider: AIProvider, apiKey: string) => {
    const response = await invoke<ApiResponse<boolean>>('update_ai_api_key', {
      provider,
      apiKey
    });

    if (response.success) {
      alert('API密钥已更新');
      loadConfig();
    } else {
      alert(`更新失败: ${response.error}`);
    }
  };

  // 切换提供商
  const switchProvider = async (provider: AIProvider) => {
    const response = await invoke<ApiResponse<boolean>>('set_active_ai_provider', {
      provider
    });

    if (response.success) {
      setActiveProvider(provider);
      alert(`已切换到 ${provider}`);
    } else {
      alert(`切换失败: ${response.error}`);
    }
  };

  // 测试连接
  const testConnection = async (provider: AIProvider) => {
    const response = await invoke<ApiResponse<string>>('test_ai_connection', {
      provider
    });

    if (response.success && response.data) {
      alert(response.data);
    } else {
      alert(`测试失败: ${response.error}`);
    }
  };

  return (
    <div>
      <h1>AI 配置管理</h1>

      {summary?.providers.map(provider => (
        <div key={provider.provider}>
          <h3>{provider.display_name}</h3>
          <p>Model: {provider.model}</p>
          <p>API Key: {provider.has_api_key ? '已设置' : '未设置'}</p>
          <p>Status: {provider.enabled ? '已启用' : '已禁用'}</p>

          <input
            type="password"
            placeholder="API Key"
            onBlur={(e) => updateAPIKey(provider.provider, e.target.value)}
          />

          <button onClick={() => switchProvider(provider.provider)}>
            设为活动
          </button>

          <button onClick={() => testConnection(provider.provider)}>
            测试连接
          </button>
        </div>
      ))}
    </div>
  );
}
```

## 最佳实践

1. **API密钥安全**:
   ```typescript
   // 使用 get_ai_config_summary 而不是 get_ai_config
   const summary = await invoke('get_ai_config_summary');

   // 不要在前端存储API密钥
   // ❌ localStorage.setItem('api_key', apiKey);

   // 输入API密钥后立即发送到后端
   const handleAPIKeySubmit = async (key: string) => {
     await invoke('update_ai_api_key', { provider, apiKey: key });
     // 清空输入框
     setAPIKey('');
   };
   ```

2. **错误处理**:
   ```typescript
   const handleProviderSwitch = async (provider: AIProvider) => {
     try {
       const response = await invoke<ApiResponse<boolean>>('set_active_ai_provider', {
         provider
       });

       if (response.success) {
         showSuccess(`已切换到 ${provider}`);
       } else {
         // 业务错误
         showError(response.error || '切换失败');
       }
     } catch (error) {
       // 系统错误
       showError('系统错误，请稍后重试');
       console.error(error);
     }
   };
   ```

3. **配置验证**:
   ```typescript
   // 切换前先测试连接
   const safeSwitch = async (provider: AIProvider) => {
     const testResult = await invoke<ApiResponse<string>>('test_ai_connection', {
       provider
     });

     if (!testResult.success) {
       if (confirm(`${provider} 连接失败，是否仍要切换？`)) {
         await invoke('set_active_ai_provider', { provider });
       }
     } else {
       await invoke('set_active_ai_provider', { provider });
     }
   };
   ```

## 相关文档

- [AI Providers Service](../../backend/services/ai-providers-service.md) - Backend service reference
- [API Settings Page](../../frontend/pages/api-settings.md) - Frontend implementation
- [Security Guidelines](../../backend/security.md) - API key security best practices
