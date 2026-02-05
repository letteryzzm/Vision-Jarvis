# AI 服务 (AIService)

> **最后更新**: 2026-02-05
> **版本**: v1.0
> **功能**: OpenAI API集成、截图分析、向量嵌入
> **实现状态**: ✅ Phase 2 已实现

**实现文件**:
- `src-tauri/src/ai/mod.rs` - OpenAI客户端
- `src-tauri/src/ai/analyzer.rs` - 截图分析器
- `src-tauri/src/ai/embeddings.rs` - 向量嵌入生成器
- `src-tauri/src/memory/vector_store.rs` - 向量存储

---

## 功能概述

AI 服务负责：

1. **截图分析**: 使用 GPT-4o 分析截图内容
2. **向量嵌入**: 生成文本的向量表示
3. **语义搜索**: 基于向量相似度的记忆搜索

---

## OpenAI API 客户端

### 支持的模型

- **GPT-4o**: 截图分析（支持Vision）
- **text-embedding-3-small**: 文本向量化（1536维）

### 功能

```rust
// 聊天完成
pub async fn chat_completion(&self, request: ChatCompletionRequest)
    -> Result<ChatCompletionResponse>

// 创建嵌入
pub async fn create_embedding(&self, request: EmbeddingRequest)
    -> Result<EmbeddingResponse>
```

### 配置

- API Key: 从 AppSettings 获取
- 超时: 60秒
- Base URL: https://api.openai.com/v1

---

## 截图分析器

### 分析流程

```
[读取截图]
    ↓
[转换为Base64]
    ↓
[构建多模态消息]
    ↓
[发送到GPT-4o]
    ↓
[解析JSON响应]
    ↓
[返回AnalysisResult]
```

### 提取信息

- **activity**: 主要活动（如"编程"、"浏览网页"）
- **application**: 应用程序名称
- **description**: 简要描述（1-2句话）
- **category**: 分类（work/entertainment/communication/other）

### 示例响应

```json
{
  "activity": "编程",
  "application": "VS Code",
  "description": "用户正在编写Rust代码，实现AI集成模块",
  "category": "work"
}
```

---

## 向量嵌入生成器

### 功能

- 生成单个文本的嵌入向量
- 批量生成（带速率限制）
- 计算余弦相似度

### 使用场景

1. **记忆索引**: 为每个记忆生成向量
2. **语义搜索**: 查询时生成查询向量，搜索相似记忆
3. **聚类分析**: 基于向量相似度聚类活动

---

## 向量存储

### 存储方式

使用 SQLite 存储向量：
- 向量序列化为 BLOB
- 元数据存储为 JSON
- 支持按时间范围删除

### 搜索算法

1. 加载所有向量
2. 计算余弦相似度
3. 按相似度排序
4. 返回 top-k 结果

### 优化策略

- 未来可升级为专用向量数据库（Qdrant、Milvus等）
- 当前实现适合中小规模数据（< 10万条）

---

## 测试覆盖

- ✅ OpenAI客户端创建和序列化: 4/4
- ✅ 截图分析器: 3/3（2个需要真实API的测试标记为ignore）
- ✅ 向量嵌入生成器: 4/4（1个需要真实API的测试标记为ignore）
- ✅ 向量存储: 5/5

**总计**: 16/16 测试通过（3个需要API key的测试忽略）

---

## API 密钥安全

- ❌ 不要硬编码API key
- ✅ 从 AppSettings 读取
- ✅ 用户在设置页面配置
- ✅ 使用环境变量或加密存储
