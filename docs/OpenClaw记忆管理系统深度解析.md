# 🧠 OpenClaw 记忆管理系统深度解析

## 📋 目录
1. [系统概述](#系统概述)
2. [核心架构](#核心架构)
3. [数据存储结构](#数据存储结构)
4. [向量检索实现](#向量检索实现)
5. [混合搜索机制](#混合搜索机制)
6. [Embedding 管理](#embedding-管理)
7. [工具接口](#工具接口)
8. [实现细节](#实现细节)
9. [二次开发指南](#二次开发指南)

---

## 系统概述

### 核心理念
OpenClaw 的记忆系统采用 **"Markdown 即数据库"** 的设计哲学：
- ✅ **纯文本存储**：所有记忆以 Markdown 文件保存
- ✅ **向量化索引**：使用 Embedding 进行语义检索
- ✅ **混合搜索**：结合 BM25 关键词 + 向量语义搜索
- ✅ **自动同步**：文件监听 + 智能增量更新

### 为什么选择这种架构？
1. **可读性**：用户可以直接查看和编辑记忆文件
2. **可移植性**：不依赖特定数据库，纯文本易于备份
3. **可扩展性**：支持多种 Embedding 后端（OpenAI、Gemini、Local）
4. **高效性**：SQLite + sqlite-vec 提供快速向量检索

---

## 核心架构

### 文件结构
```
~/.openclaw/
├── workspace/                    # 工作空间
│   ├── MEMORY.md                # 长期记忆（主文件）
│   └── memory/                  # 日志式记忆
│       ├── 2024-01-15.md       # 每日记忆日志
│       └── 2024-01-16.md
├── memory/                       # 向量索引数据库
│   └── main.sqlite              # SQLite 索引文件
└── agents/
    └── main/
        └── qmd/                 # QMD 后端（可选）
```

### 核心模块关系图

```
┌─────────────────────────────────────────────────────────┐
│              Agent Tools Layer                          │
│  ┌──────────────────┐    ┌─────────────────┐           │
│  │  memory_search   │    │   memory_get    │           │
│  └────────┬─────────┘    └────────┬────────┘           │
└───────────┼──────────────────────┼─────────────────────┘
            │                      │
            └──────────┬───────────┘
                       ▼
┌─────────────────────────────────────────────────────────┐
│          Memory Search Manager                          │
│  ┌──────────────────────────────────────────────────┐  │
│  │  getMemorySearchManager()                        │  │
│  │  - 后端选择（builtin vs QMD）                     │  │
│  │  - 自动 fallback 机制                            │  │
│  └──────────────────────────────────────────────────┘  │
└───────────┬─────────────────────────────────────────────┘
            │
    ┌───────┴────────┐
    │                │
    ▼                ▼
┌─────────────┐  ┌─────────────────┐
│   Builtin   │  │   QMD Backend   │
│   Backend   │  │  (experimental) │
└──────┬──────┘  └────────┬────────┘
       │                  │
       ▼                  ▼
┌──────────────────────────────────┐
│    MemoryIndexManager            │
│  ┌────────────────────────────┐  │
│  │  - Embedding Provider      │  │
│  │  - SQLite + sqlite-vec     │  │
│  │  - Hybrid Search           │  │
│  │  - File Watcher            │  │
│  │  - Chunk Management        │  │
│  └────────────────────────────┘  │
└─────────┬────────────────────────┘
          │
    ┌─────┴──────┬──────────┬──────────┐
    ▼            ▼          ▼          ▼
┌────────┐  ┌────────┐ ┌────────┐ ┌────────┐
│ OpenAI │  │ Gemini │ │ Voyage │ │ Local  │
│Embed   │  │ Embed  │ │ Embed  │ │(llama) │
└────────┘  └────────┘ └────────┘ └────────┘
```

### 关键类和接口

#### 1. `MemorySearchManager` 接口 (types.ts)
```typescript
export interface MemorySearchManager {
  // 语义搜索
  search(
    query: string,
    opts?: {
      maxResults?: number;
      minScore?: number;
      sessionKey?: string
    }
  ): Promise<MemorySearchResult[]>;

  // 读取文件
  readFile(params: {
    relPath: string;
    from?: number;
    lines?: number;
  }): Promise<{ text: string; path: string }>;

  // 获取状态
  status(): MemoryProviderStatus;

  // 同步索引
  sync?(params?: {
    reason?: string;
    force?: boolean;
    progress?: (update: MemorySyncProgressUpdate) => void;
  }): Promise<void>;

  // 检查可用性
  probeEmbeddingAvailability(): Promise<MemoryEmbeddingProbeResult>;
  probeVectorAvailability(): Promise<boolean>;

  // 清理资源
  close?(): Promise<void>;
}
```

#### 2. `MemoryIndexManager` 类 (manager.ts)
核心实现类，管理整个记忆索引生命周期。

**主要职责**：
- **索引管理**：扫描 Markdown 文件并分块
- **向量生成**：调用 Embedding Provider 生成向量
- **持久化**：SQLite 存储文件元数据、分块、向量
- **搜索**：混合搜索（Vector + BM25）
- **同步**：文件变更监听 + 增量更新

---

## 数据存储结构

### SQLite Schema (memory-schema.ts)

#### 1. `meta` 表 - 元数据
```sql
CREATE TABLE meta (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
```
存储索引元数据（provider、model、chunk 参数等）

#### 2. `files` 表 - 文件追踪
```sql
CREATE TABLE files (
  path TEXT PRIMARY KEY,           -- 相对路径
  source TEXT NOT NULL DEFAULT 'memory',  -- 来源: memory/sessions
  hash TEXT NOT NULL,              -- 文件哈希
  mtime INTEGER NOT NULL,          -- 修改时间戳
  size INTEGER NOT NULL            -- 文件大小
);
```

#### 3. `chunks` 表 - 文本分块
```sql
CREATE TABLE chunks (
  id TEXT PRIMARY KEY,             -- UUID
  path TEXT NOT NULL,              -- 文件路径
  source TEXT NOT NULL DEFAULT 'memory',
  start_line INTEGER NOT NULL,    -- 起始行号
  end_line INTEGER NOT NULL,      -- 结束行号
  hash TEXT NOT NULL,              -- 分块哈希
  model TEXT NOT NULL,             -- Embedding 模型
  text TEXT NOT NULL,              -- 原始文本
  embedding TEXT NOT NULL,         -- 向量（JSON 或 blob）
  updated_at INTEGER NOT NULL     -- 更新时间
);

CREATE INDEX idx_chunks_path ON chunks(path);
CREATE INDEX idx_chunks_source ON chunks(source);
```

#### 4. `embedding_cache` 表 - Embedding 缓存
```sql
CREATE TABLE embedding_cache (
  provider TEXT NOT NULL,          -- openai/gemini/local
  model TEXT NOT NULL,             -- 模型名
  provider_key TEXT NOT NULL,      -- provider 标识
  hash TEXT NOT NULL,              -- 文本哈希
  embedding TEXT NOT NULL,         -- 向量数据
  dims INTEGER,                    -- 向量维度
  updated_at INTEGER NOT NULL,    -- 缓存时间
  PRIMARY KEY (provider, model, provider_key, hash)
);

CREATE INDEX idx_embedding_cache_updated_at
  ON embedding_cache(updated_at);
```

**缓存机制的作用**：
- ✅ 避免重复计算相同文本的 Embedding
- ✅ 加速增量索引（只计算新内容）
- ✅ 支持跨文件去重（相同文本共享向量）

#### 5. `chunks_fts` 表 - 全文搜索索引 (FTS5)
```sql
CREATE VIRTUAL TABLE chunks_fts USING fts5(
  text,                            -- 全文索引文本
  id UNINDEXED,                   -- chunk ID（不索引）
  path UNINDEXED,
  source UNINDEXED,
  model UNINDEXED,
  start_line UNINDEXED,
  end_line UNINDEXED
);
```
FTS5 提供 BM25 关键词搜索。

#### 6. `chunks_vec` 表 - 向量索引 (sqlite-vec)
```sql
-- 通过 sqlite-vec 扩展创建
CREATE VIRTUAL TABLE chunks_vec USING vec0(
  embedding float[N]               -- N = embedding 维度
);
```
**sqlite-vec** 是高效的 SQLite 向量搜索扩展。

---

## 向量检索实现

### Embedding Provider 架构

#### 支持的 Provider

| Provider | 模型 | 特点 |
|----------|------|------|
| **OpenAI** | `text-embedding-3-small` | 默认，快速，支持批量 API |
| **Gemini** | `gemini-embedding-001` | Google 生态，批量支持 |
| **Voyage** | `voyage-3-lite` | 专注 embedding |
| **Local** | `embeddinggemma-300M` | 离线运行，隐私友好 |

#### Embedding 生成流程

```typescript
// embeddings.ts

export type EmbeddingProvider = {
  id: string;
  model: string;
  embedQuery: (text: string) => Promise<number[]>;
  embedBatch: (texts: string[]) => Promise<number[][]>;
};

// 创建 Provider
export async function createEmbeddingProvider(
  options: EmbeddingProviderOptions
): Promise<EmbeddingProviderResult> {
  const requestedProvider = options.provider;

  // 1. 尝试创建主 Provider
  try {
    if (requestedProvider === 'local') {
      return await createLocalEmbeddingProvider(options);
    } else if (requestedProvider === 'gemini') {
      return await createGeminiEmbeddingProvider(options);
    } else if (requestedProvider === 'openai') {
      return await createOpenAiEmbeddingProvider(options);
    }
  } catch (err) {
    // 2. 失败时尝试 Fallback
    const fallback = options.fallback;
    if (fallback !== 'none') {
      return await createFallbackProvider(fallback, options);
    }
    throw err;
  }
}
```

#### 向量归一化
```typescript
// embeddings.ts
function sanitizeAndNormalizeEmbedding(vec: number[]): number[] {
  // 1. 处理非法值
  const sanitized = vec.map(value =>
    Number.isFinite(value) ? value : 0
  );

  // 2. L2 归一化
  const magnitude = Math.sqrt(
    sanitized.reduce((sum, value) => sum + value * value, 0)
  );

  if (magnitude < 1e-10) {
    return sanitized;
  }

  return sanitized.map(value => value / magnitude);
}
```

### 文本分块策略 (Chunking)

#### 分块参数
```typescript
// internal.ts
const CHUNK_TARGET_TOKENS = 400;  // 目标分块大小
const CHUNK_OVERLAP_TOKENS = 80;  // 重叠 token 数
```

#### 分块逻辑
```typescript
export function chunkMarkdown(params: {
  text: string;
  targetTokens: number;
  overlapTokens: number;
}): MemoryChunk[] {
  const lines = params.text.split('\n');
  const chunks: MemoryChunk[] = [];

  let currentChunk: string[] = [];
  let currentTokens = 0;
  let startLine = 1;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const lineTokens = estimateTokens(line);

    // 超过目标大小，切分
    if (currentTokens + lineTokens > params.targetTokens) {
      if (currentChunk.length > 0) {
        chunks.push({
          startLine,
          endLine: i,
          text: currentChunk.join('\n'),
          hash: hashText(currentChunk.join('\n'))
        });

        // 保留 overlap 行
        currentChunk = currentChunk.slice(-params.overlapTokens);
        startLine = i - currentChunk.length;
        currentTokens = estimateTokens(currentChunk.join('\n'));
      }
    }

    currentChunk.push(line);
    currentTokens += lineTokens;
  }

  // 处理最后一块
  if (currentChunk.length > 0) {
    chunks.push({
      startLine,
      endLine: lines.length,
      text: currentChunk.join('\n'),
      hash: hashText(currentChunk.join('\n'))
    });
  }

  return chunks;
}
```

**为什么需要 Overlap？**
- ✅ 避免重要信息被截断
- ✅ 上下文连贯性
- ✅ 提高召回率

---

## 混合搜索机制

### 为什么需要混合搜索？

| 搜索类型 | 优势 | 劣势 |
|---------|------|------|
| **向量搜索** | 语义理解强，支持同义词 | 精确匹配弱（ID、代码） |
| **BM25 关键词** | 精确 token 匹配 | 无法理解语义 |
| **混合搜索** | 兼顾语义 + 精确 | 复杂度稍高 |

### 实现细节 (hybrid.ts)

#### 1. BM25 查询构建
```typescript
export function buildFtsQuery(raw: string): string | null {
  // 提取 token
  const tokens = raw
    .match(/[A-Za-z0-9_]+/g)
    ?.map(t => t.trim())
    .filter(Boolean) ?? [];

  if (tokens.length === 0) {
    return null;
  }

  // 构建 FTS5 查询（AND 连接）
  const quoted = tokens.map(t => `"${t.replaceAll('"', '')}"`);
  return quoted.join(' AND ');
}
```

#### 2. BM25 分数归一化
```typescript
export function bm25RankToScore(rank: number): number {
  // rank 越小越好（FTS5 返回值）
  const normalized = Number.isFinite(rank) ? Math.max(0, rank) : 999;
  return 1 / (1 + normalized);  // 转换为 0-1 分数
}
```

#### 3. 结果合并
```typescript
export function mergeHybridResults(params: {
  vector: HybridVectorResult[];
  keyword: HybridKeywordResult[];
  vectorWeight: number;  // 默认 0.7
  textWeight: number;    // 默认 0.3
}) {
  const byId = new Map();

  // 1. 添加向量结果
  for (const r of params.vector) {
    byId.set(r.id, {
      ...r,
      vectorScore: r.vectorScore,
      textScore: 0
    });
  }

  // 2. 合并关键词结果
  for (const r of params.keyword) {
    const existing = byId.get(r.id);
    if (existing) {
      existing.textScore = r.textScore;
    } else {
      byId.set(r.id, {
        ...r,
        vectorScore: 0,
        textScore: r.textScore
      });
    }
  }

  // 3. 计算混合分数
  const merged = Array.from(byId.values()).map(entry => {
    const score =
      params.vectorWeight * entry.vectorScore +
      params.textWeight * entry.textScore;

    return { ...entry, score };
  });

  // 4. 按分数排序
  return merged.toSorted((a, b) => b.score - a.score);
}
```

### 搜索配置
```json5
{
  agents: {
    defaults: {
      memorySearch: {
        query: {
          hybrid: {
            enabled: true,
            vectorWeight: 0.7,      // 向量权重 70%
            textWeight: 0.3,        // ���键词权重 30%
            candidateMultiplier: 4  // 候选池放大倍数
          }
        }
      }
    }
  }
}
```

---

## Embedding 管理

### 批量处理 (Batch API)

#### OpenAI Batch API
OpenAI 提供了异步批量 Embedding API，优势：
- ✅ **50% 折扣价格**
- ✅ **更高吞吐量**
- ✅ **异步处理**

```typescript
// batch-openai.ts
export async function runOpenAiEmbeddingBatches(params: {
  requests: OpenAiBatchRequest[];
  concurrency: number;        // 并发数
  pollIntervalMs: number;     // 轮询间隔
  timeoutMs: number;          // 超时时间
  wait: boolean;              // 是否等待完成
}): Promise<Map<string, number[]>> {
  const client = params.requests[0].client;
  const results = new Map<string, number[]>();

  // 1. 构建 batch 请求
  const batchLines = params.requests.map((req, idx) => ({
    custom_id: req.id,
    method: 'POST',
    url: '/v1/embeddings',
    body: {
      model: req.model,
      input: req.text
    }
  }));

  // 2. 提交 batch
  const batch = await client.batches.create({
    input_file: uploadBatchFile(batchLines),
    endpoint: '/v1/embeddings',
    completion_window: '24h'
  });

  // 3. 轮询状态
  if (params.wait) {
    let status = batch.status;
    while (status === 'in_progress') {
      await sleep(params.pollIntervalMs);
      const updated = await client.batches.retrieve(batch.id);
      status = updated.status;
    }

    // 4. 获取结果
    if (status === 'completed') {
      const output = await client.files.retrieve(batch.output_file_id);
      // 解析结果...
    }
  }

  return results;
}
```

### Embedding 缓存机制

#### 缓存键计算
```typescript
// manager-cache-key.ts
function buildCacheKey(params: {
  provider: string;
  model: string;
  textHash: string;
}): string {
  return `${params.provider}:${params.model}:${params.textHash}`;
}
```

#### 缓存查询
```typescript
// manager.ts (MemoryIndexManager)
private async getEmbeddingFromCache(
  textHash: string
): Promise<number[] | null> {
  if (!this.cache.enabled) {
    return null;
  }

  const stmt = this.db.prepare(`
    SELECT embedding
    FROM embedding_cache
    WHERE provider = ?
      AND model = ?
      AND provider_key = ?
      AND hash = ?
  `);

  const row = stmt.get(
    this.provider.id,
    this.provider.model,
    this.providerKey,
    textHash
  );

  if (!row) {
    return null;
  }

  return parseEmbedding(row.embedding);
}
```

#### 缓���写入
```typescript
private async saveEmbeddingToCache(
  textHash: string,
  embedding: number[]
): Promise<void> {
  if (!this.cache.enabled) {
    return;
  }

  const stmt = this.db.prepare(`
    INSERT OR REPLACE INTO embedding_cache
    (provider, model, provider_key, hash, embedding, dims, updated_at)
    VALUES (?, ?, ?, ?, ?, ?, ?)
  `);

  stmt.run(
    this.provider.id,
    this.provider.model,
    this.providerKey,
    textHash,
    JSON.stringify(embedding),
    embedding.length,
    Date.now()
  );

  // 清理过期缓存
  await this.evictOldCacheEntries();
}
```

### 自动索引同步

#### 文件监听
```typescript
// manager.ts
private setupFileWatcher(): void {
  const memoryDir = path.join(this.workspaceDir, 'memory');
  const memoryFile = path.join(this.workspaceDir, 'MEMORY.md');

  this.watcher = chokidar.watch(
    [memoryDir, memoryFile],
    {
      ignoreInitial: true,
      awaitWriteFinish: {
        stabilityThreshold: 500,  // 文件稳定 500ms 后触发
        pollInterval: 100
      }
    }
  );

  this.watcher.on('all', (event, filePath) => {
    if (!filePath.endsWith('.md')) {
      return;
    }

    // 标记为 dirty，延迟同步
    this.markDirty();
  });
}

private markDirty(): void {
  this.dirty = true;

  // 防抖：1.5 秒内连续修改只触发一次
  if (this.watchTimer) {
    clearTimeout(this.watchTimer);
  }

  this.watchTimer = setTimeout(() => {
    this.sync({ reason: 'file-change' });
  }, 1500);
}
```

#### 增量同步流程
```typescript
async sync(params?: {
  reason?: string;
  force?: boolean;
  progress?: (update) => void;
}): Promise<void> {
  // 1. 扫描文件
  const currentFiles = await listMemoryFiles(this.workspaceDir);

  // 2. 对比哈希，找出变更文件
  const changedFiles = [];
  for (const absPath of currentFiles) {
    const stat = await fs.stat(absPath);
    const content = await fs.readFile(absPath, 'utf-8');
    const hash = hashText(content);

    const existing = this.getFileEntry(absPath);
    if (!existing || existing.hash !== hash) {
      changedFiles.push({ absPath, content, hash });
    }
  }

  // 3. 删除已移除文件的索引
  await this.removeDeletedFiles(currentFiles);

  // 4. 重新索引变更文件
  for (const file of changedFiles) {
    await this.indexFile(file);
  }

  // 5. 更新元数据
  await this.updateMetadata();
}

private async indexFile(file: {
  absPath: string;
  content: string;
  hash: string;
}): Promise<void> {
  // 1. 分块
  const chunks = chunkMarkdown({
    text: file.content,
    targetTokens: 400,
    overlapTokens: 80
  });

  // 2. 生成 Embedding
  const embeddings = await this.provider.embedBatch(
    chunks.map(c => c.text)
  );

  // 3. 保存到数据库
  for (let i = 0; i < chunks.length; i++) {
    const chunk = chunks[i];
    const embedding = embeddings[i];

    await this.saveChunk({
      id: randomUUID(),
      path: file.absPath,
      chunk,
      embedding
    });
  }
}
```

---

## 工具接口

### `memory_search` 工具

#### 工具定义
```typescript
// tools/memory-tool.ts
export function createMemorySearchTool(options: {
  config?: OpenClawConfig;
  agentSessionKey?: string;
}): AnyAgentTool | null {
  return {
    label: 'Memory Search',
    name: 'memory_search',
    description:
      '语义搜索 MEMORY.md + memory/*.md；' +
      '返回相关片段（含路径和行号）',

    parameters: Type.Object({
      query: Type.String(),
      maxResults: Type.Optional(Type.Number()),
      minScore: Type.Optional(Type.Number())
    }),

    execute: async (_toolCallId, params) => {
      const { manager, error } = await getMemorySearchManager({
        cfg,
        agentId
      });

      if (!manager) {
        return { results: [], disabled: true, error };
      }

      const results = await manager.search(params.query, {
        maxResults: params.maxResults,
        minScore: params.minScore
      });

      return {
        results,
        provider: manager.status().provider,
        model: manager.status().model
      };
    }
  };
}
```

#### 返回结果格式
```typescript
type MemorySearchResult = {
  path: string;          // 文件路径，如 "memory/2024-01-15.md"
  startLine: number;     // 起始行号
  endLine: number;       // 结束行号
  score: number;         // 相似度分数 (0-1)
  snippet: string;       // 匹配片段（最多 700 字符）
  source: 'memory' | 'sessions';  // 来源
  citation?: string;     // 引用格式（可选）
};
```

#### 调用示例
```typescript
// Agent 调用示例
const results = await memory_search({
  query: "如何配置 Telegram 机器人？",
  maxResults: 5,
  minScore: 0.3
});

// 结果示例
[
  {
    path: "memory/2024-01-10.md",
    startLine: 45,
    endLine: 62,
    score: 0.87,
    snippet: "## Telegram 配置步骤\n1. 联系 @BotFather...",
    source: "memory",
    citation: "memory/2024-01-10.md#L45-62"
  }
]
```

### `memory_get` 工具

#### 工具定义
```typescript
export function createMemoryGetTool(options: {
  config?: OpenClawConfig;
  agentSessionKey?: string;
}): AnyAgentTool | null {
  return {
    label: 'Memory Get',
    name: 'memory_get',
    description:
      '读取 MEMORY.md 或 memory/*.md 的特定内容；' +
      '支持按行范围读取',

    parameters: Type.Object({
      path: Type.String(),
      from: Type.Optional(Type.Number()),  // 起始行
      lines: Type.Optional(Type.Number())  // 读取行数
    }),

    execute: async (_toolCallId, params) => {
      const { manager } = await getMemorySearchManager({
        cfg,
        agentId
      });

      const result = await manager.readFile({
        relPath: params.path,
        from: params.from,
        lines: params.lines
      });

      return {
        path: result.path,
        text: result.text
      };
    }
  };
}
```

#### 调用示例
```typescript
// 读取完整文件
const full = await memory_get({
  path: "MEMORY.md"
});

// 读取指定行范围
const partial = await memory_get({
  path: "memory/2024-01-15.md",
  from: 10,    // 从第 10 行开始
  lines: 20    // 读取 20 行
});
```

---

## 实现细节

### 1. 向量相似度计算

#### 余弦相似度
```typescript
function cosineSimilarity(a: number[], b: number[]): number {
  if (a.length !== b.length) {
    throw new Error('Vector dimensions mismatch');
  }

  let dotProduct = 0;
  let magA = 0;
  let magB = 0;

  for (let i = 0; i < a.length; i++) {
    dotProduct += a[i] * b[i];
    magA += a[i] * a[i];
    magB += b[i] * b[i];
  }

  const magnitude = Math.sqrt(magA) * Math.sqrt(magB);

  if (magnitude === 0) {
    return 0;
  }

  return dotProduct / magnitude;
}
```

#### sqlite-vec 加速查询
```typescript
// manager-search.ts
export async function searchVector(params: {
  db: DatabaseSync;
  queryEmbedding: number[];
  maxResults: number;
  minScore?: number;
  source?: string;
  useVectorTable: boolean;
}): Promise<VectorSearchResult[]> {
  if (params.useVectorTable) {
    // 使用 sqlite-vec 扩展
    const stmt = params.db.prepare(`
      SELECT
        c.id,
        c.path,
        c.start_line,
        c.end_line,
        c.source,
        c.text,
        vec_distance_cosine(v.embedding, ?) AS distance
      FROM chunks c
      JOIN chunks_vec v ON c.rowid = v.rowid
      WHERE v.embedding MATCH ?
        AND vec_distance_cosine(v.embedding, ?) < ?
      ORDER BY distance ASC
      LIMIT ?
    `);

    const blob = vectorToBlob(params.queryEmbedding);
    const maxDistance = 1 - (params.minScore ?? 0);

    return stmt.all(blob, blob, maxDistance, params.maxResults);
  } else {
    // 回退到 JS 实现
    return searchVectorInMemory(params);
  }
}
```

### 2. 文本哈希计算
```typescript
// internal.ts
export function hashText(text: string): string {
  return crypto
    .createHash('sha256')
    .update(text, 'utf-8')
    .digest('hex')
    .substring(0, 16);  // 取前 16 位
}
```

### 3. 并发控制
```typescript
// internal.ts
export async function runWithConcurrency<T, R>(params: {
  items: T[];
  concurrency: number;
  fn: (item: T, index: number) => Promise<R>;
}): Promise<R[]> {
  const results: R[] = [];
  const queue = [...params.items];
  let running = 0;
  let index = 0;

  return new Promise((resolve, reject) => {
    const processNext = async () => {
      if (queue.length === 0 && running === 0) {
        resolve(results);
        return;
      }

      while (running < params.concurrency && queue.length > 0) {
        const item = queue.shift()!;
        const currentIndex = index++;
        running++;

        params.fn(item, currentIndex)
          .then(result => {
            results[currentIndex] = result;
            running--;
            processNext();
          })
          .catch(reject);
      }
    };

    processNext();
  });
}
```

### 4. 会话记忆增量索引

#### 会话文件监听
```typescript
private setupSessionFileWatcher(): void {
  const transcriptsDir = resolveSessionTranscriptsDirForAgent(
    this.cfg,
    this.agentId
  );

  // 监听会话文件变更
  this.sessionUnsubscribe = onSessionTranscriptUpdate({
    agentId: this.agentId,
    callback: (sessionKey, update) => {
      this.handleSessionUpdate(sessionKey, update);
    }
  });
}

private handleSessionUpdate(
  sessionKey: string,
  update: { deltaBytes: number; deltaMessages: number }
): void {
  const delta = this.sessionDeltas.get(sessionKey) || {
    lastSize: 0,
    pendingBytes: 0,
    pendingMessages: 0
  };

  delta.pendingBytes += update.deltaBytes;
  delta.pendingMessages += update.deltaMessages;

  this.sessionDeltas.set(sessionKey, delta);

  // 达到阈值，触发索引
  const config = this.settings.sync?.sessions;
  if (
    delta.pendingBytes >= (config?.deltaBytes ?? 100000) ||
    delta.pendingMessages >= (config?.deltaMessages ?? 50)
  ) {
    this.markSessionsDirty([sessionKey]);
  }
}
```

---

## 二次开发指南

### 场景 1：添加自定义 Embedding Provider

假设你想集成 Cohere 的 Embedding API。

#### 步骤 1：创建 Provider 文件
```typescript
// src/memory/embeddings-cohere.ts

import type { EmbeddingProvider } from './embeddings.js';

export const DEFAULT_COHERE_EMBEDDING_MODEL = 'embed-multilingual-v3.0';

export type CohereEmbeddingClient = {
  apiKey: string;
  baseUrl?: string;
};

export async function createCohereEmbeddingProvider(options: {
  remote?: {
    baseUrl?: string;
    apiKey?: string;
  };
  model: string;
}): Promise<{
  provider: EmbeddingProvider;
  client: CohereEmbeddingClient;
}> {
  const apiKey = options.remote?.apiKey;
  if (!apiKey) {
    throw new Error('Cohere API key required');
  }

  const baseUrl = options.remote?.baseUrl || 'https://api.cohere.ai/v1';
  const client: CohereEmbeddingClient = { apiKey, baseUrl };

  const provider: EmbeddingProvider = {
    id: 'cohere',
    model: options.model || DEFAULT_COHERE_EMBEDDING_MODEL,

    async embedQuery(text: string): Promise<number[]> {
      const response = await fetch(`${baseUrl}/embed`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${apiKey}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          texts: [text],
          model: this.model,
          input_type: 'search_query'
        })
      });

      const data = await response.json();
      return data.embeddings[0];
    },

    async embedBatch(texts: string[]): Promise<number[][]> {
      const response = await fetch(`${baseUrl}/embed`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${apiKey}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          texts,
          model: this.model,
          input_type: 'search_document'
        })
      });

      const data = await response.json();
      return data.embeddings;
    }
  };

  return { provider, client };
}
```

#### 步骤 2：集成到 Provider 工厂
```typescript
// src/memory/embeddings.ts

import { createCohereEmbeddingProvider } from './embeddings-cohere.js';

export async function createEmbeddingProvider(
  options: EmbeddingProviderOptions
): Promise<EmbeddingProviderResult> {
  // ... 现有代码

  const createProvider = async (
    id: 'openai' | 'local' | 'gemini' | 'voyage' | 'cohere'
  ) => {
    if (id === 'cohere') {
      const { provider, client } = await createCohereEmbeddingProvider(options);
      return { provider, cohere: client };
    }

    // ... 其他 provider
  };

  // ... 后续逻辑
}
```

#### 步骤 3：配置支持
```typescript
// 在配置中启用
{
  agents: {
    defaults: {
      memorySearch: {
        provider: "cohere",
        model: "embed-multilingual-v3.0",
        remote: {
          apiKey: "YOUR_COHERE_API_KEY"
        }
      }
    }
  }
}
```

---

### 场景 2：自定义记忆存储路径

#### 需求
将记忆文件存储到 Dropbox 同步文件夹。

#### 实现
```typescript
// config.json5
{
  agents: {
    defaults: {
      workspace: "~/Dropbox/openclaw-memory",
      memorySearch: {
        enabled: true,
        extraPaths: [
          "~/Dropbox/notes",           // 额外索引笔记目录
          "~/Dropbox/projects/*.md"    // glob 模式
        ]
      }
    }
  }
}
```

---

### 场景 3：实现自定义记忆触发器

#### 需求
当用户说 "记住这个" 时，自动保存到 MEMORY.md。

#### 实现
```typescript
// src/agents/hooks/memory-auto-save.ts

export function createMemoryAutoSaveHook(options: {
  workspaceDir: string;
}): Hook {
  return {
    name: 'memory-auto-save',

    async onUserMessage(context) {
      const message = context.message.text;

      // 检测触发词
      const triggers = ['记住', '记下', '保存这个'];
      const shouldSave = triggers.some(t => message.includes(t));

      if (!shouldSave) {
        return;
      }

      // 提取要保存的内容
      const content = extractMemoryContent(message);

      // 追加到 MEMORY.md
      const memoryFile = path.join(options.workspaceDir, 'MEMORY.md');
      const timestamp = new Date().toISOString();
      const entry = `\n\n## ${timestamp}\n${content}\n`;

      await fs.appendFile(memoryFile, entry, 'utf-8');

      // 通知用户
      await context.reply('✅ 已保存到记忆');
    }
  };
}

function extractMemoryContent(message: string): string {
  // 简单实现：移除触发词
  return message
    .replace(/记住|记下|保存这个/g, '')
    .trim();
}
```

---

### 场景 4：添加记忆统计面板

#### 需求
查看记忆使用情况（文件数、向量数、存储大小）。

#### 实现
```typescript
// src/cli/commands/memory-stats.ts

import { Command } from 'commander';
import { getMemorySearchManager } from '../../memory/index.js';

export const memoryStatsCommand = new Command('memory:stats')
  .description('显示记忆系统统计信息')
  .action(async () => {
    const config = await loadConfig();
    const { manager } = await getMemorySearchManager({
      cfg: config,
      agentId: 'main'
    });

    if (!manager) {
      console.error('记忆系统未启用');
      return;
    }

    const status = manager.status();

    console.log('📊 记忆系统统计\n');
    console.log(`  Backend:  ${status.backend}`);
    console.log(`  Provider: ${status.provider}`);
    console.log(`  Model:    ${status.model}`);
    console.log(`  文件数:   ${status.files}`);
    console.log(`  分块数:   ${status.chunks}`);

    if (status.cache) {
      console.log(`  缓存数:   ${status.cache.entries} / ${status.cache.maxEntries}`);
    }

    if (status.vector) {
      console.log(`  向量维度: ${status.vector.dims}`);
      console.log(`  向量加速: ${status.vector.available ? '✅' : '❌'}`);
    }

    if (status.fts) {
      console.log(`  全文搜索: ${status.fts.available ? '✅' : '❌'}`);
    }

    // 数据库大小
    if (status.dbPath) {
      const stat = await fs.stat(status.dbPath);
      const sizeMB = (stat.size / 1024 / 1024).toFixed(2);
      console.log(`  索引大小: ${sizeMB} MB`);
    }
  });
```

注册命令：
```typescript
// src/cli/index.ts
program.addCommand(memoryStatsCommand);
```

使用：
```bash
openclaw memory:stats
```

---

### 场景 5：实现记忆导出功能

#### 需求
导出所有记忆为单个 Markdown 文件。

#### 实现
```typescript
// src/cli/commands/memory-export.ts

import { Command } from 'commander';

export const memoryExportCommand = new Command('memory:export')
  .description('导出记忆为 Markdown')
  .option('-o, --output <file>', '输出文件路径')
  .action(async (options) => {
    const config = await loadConfig();
    const workspaceDir = resolveAgentWorkspaceDir(config, 'main');

    const memoryFiles = await listMemoryFiles(workspaceDir);

    let output = '# OpenClaw 记忆导出\n\n';
    output += `导出时间: ${new Date().toISOString()}\n\n`;
    output += '---\n\n';

    for (const filePath of memoryFiles) {
      const relPath = path.relative(workspaceDir, filePath);
      const content = await fs.readFile(filePath, 'utf-8');

      output += `\n## 文件: ${relPath}\n\n`;
      output += content;
      output += '\n\n---\n\n';
    }

    const outputPath = options.output || 'openclaw-memory-export.md';
    await fs.writeFile(outputPath, output, 'utf-8');

    console.log(`✅ 记忆已导出到: ${outputPath}`);
  });
```

---

## 🎯 总结

### 核心优势
1. **可读性**：纯 Markdown 文件，人类可读可编辑
2. **高效性**：SQLite + sqlite-vec 提供快速向量检索
3. **灵活性**：支持多种 Embedding 后端
4. **智能性**：混合搜索（语义 + 关键词）
5. **实时性**：文件监听 + 增量索引

### 学习路径建议
1. ✅ **理解架构**：从 `types.ts` 和 `search-manager.ts` 开始
2. ✅ **阅读核心实现**：`manager.ts`（2000+ 行，核心逻辑）
3. ✅ **研究搜索算法**：`hybrid.ts`、`manager-search.ts`
4. ✅ **查看工具接口**：`tools/memory-tool.ts`
5. ✅ **测试验证**：运行 `src/memory/*.test.ts`

### 扩展方向
- ✨ 实现图像记忆（OCR + 多模态 Embedding）
- ✨ 集成知识图谱（三元组提取）
- ✨ 添加记忆去重（语义相似度聚类）
- ✨ 支持记忆版本控制（Git 集成）

---

**关键文件清单**：
- `src/memory/manager.ts` - 核心索引管理器
- `src/memory/search-manager.ts` - 搜索管理器工厂
- `src/memory/embeddings.ts` - Embedding Provider 抽象
- `src/memory/hybrid.ts` - 混合搜索实现
- `src/memory/internal.ts` - 工具函数（分块、哈希等）
- `src/agents/tools/memory-tool.ts` - Agent 工具接口

祝你学习顺利！🚀
