# Vision-Jarvis 详细功能规格

> **文档类型**: 功能规格说明书 (Functional Specification)
> **创建日期**: 2026-02-01
> **版本**: v1.0
> **优先级**: 最高 (P0)

## 📋 为什么要先做功能规格？

- **明确实现细节**: 技术人员需要知道每个功能具体怎么实现
- **定义清晰边界**: 输入/输出/状态机/边界条件
- **避免理解偏差**: 减少开发过程中的沟通成本
- **质量保证基础**: 为测试用例提供依据

---

## 功能 1: 屏幕截图采集

### 1.1 功能描述

持续获取用户屏幕截图，记录当前使用的应用程序，作为记忆系统的数据源。

### 1.2 输入规格

| 输入项 | 类型 | 来源 | 约束条件 | 示例 |
|--------|------|------|----------|------|
| 截图触发 | 定时/事件 | 系统定时器 | 间隔 5-30秒可配置 | 每10秒触发一次 |
| 应用信息 | 系统API | 操作系统 | 必须包含进程名和窗口标题 | `{app: "chrome.exe", title: "..."}` |
| 用户权限 | 布尔值 | 用户授权 | 必须获得屏幕录制权限 | `screen_permission: true` |

### 1.3 处理逻辑

```
[定时器触发]
    ↓
[检查权限]
    ↓ 有权限
[获取当前活跃应用]
    ↓
[判断是否需要截图]
    ├─ 应用切换 → 立即截图
    ├─ 内容变化 → 立即截图
    └─ 定时触发 → 截图
    ↓
[执行截图]
    ↓
[压缩处理]
    ↓
[保存到本地]
    ↓
[创建 D1 记录]
    ↓
[触发 AI 分析队列]
```

### 1.4 输出规格

| 输出项 | 类型 | 格式 | 存储位置 | 示例 |
|--------|------|------|----------|------|
| 截图文件 | 图片 | PNG/JPG | 本地存储 | `/screenshots/2026/02/01/screenshot_1738406435_001.png` |
| D1 记录 | 数据库记录 | SQL | SQLite | `{id: 1, file_path: "...", ...}` |

### 1.5 状态机

```
[空闲状态 (Idle)]
    ↓ 定时器触发
[准备截图 (Ready)]
    ↓ 权限检查通过
[截图中 (Capturing)]
    ↓ 截图成功
[处理中 (Processing)]
    ↓ 保存成功
[完成 (Completed)]
    ↓ 等待下次触发
[空闲状态 (Idle)]

异常流程:
[任意状态] → [错误 (Error)] → [重试 (Retry)] → [空闲状态]
```

### 1.6 边界条件

| 场景 | 条件 | 处理方式 | 预期结果 |
|------|------|----------|----------|
| **无权限** | 用户未授权屏幕录制 | 显示权限请求弹窗 | 引导用户授权 |
| **磁盘空间不足** | 剩余空间 < 1GB | 停止截图，清理旧文件 | 提示用户清理空间 |
| **应用最小化** | 所有窗口最小化 | 跳过本次截图 | 等待下次触发 |
| **屏幕锁定** | 系统锁屏状态 | 暂停截图 | 解锁后恢复 |
| **隐私模式** | 用户开启隐私保护 | 不截图 | 记录空白时段 |
| **截图失败** | API 调用失败 | 重试 3 次 | 失败后记录错误日志 |
| **文件过大** | 截图 > 10MB | 压缩到 5MB 以下 | 保存压缩后的文件 |

### 1.7 性能要求

| 指标 | 目标值 | 测量方法 |
|------|--------|----------|
| 截图耗时 | < 500ms | 从触发到保存完成 |
| CPU 占用 | < 5% | 截图过程中的平均 CPU |
| 内存占用 | < 200MB | 截图缓存占用 |
| 存储增长 | < 500MB/天 | 按默认配置计算 |

### 1.8 异常处理代码示例

```rust
// src-tauri/src/services/screen_capture/capturer.rs

use crate::error::{AppError, Result};
use crate::database::repositories::screenshot_repo::ScreenshotRepository;
use screenshots::Screen;
use std::path::PathBuf;

pub struct ScreenCapturer {
    storage_dir: PathBuf,
    screenshot_repo: ScreenshotRepository,
    retry_count: u8,
}

impl ScreenCapturer {
    pub async fn capture_screenshot(&mut self) -> Result<ScreenshotInfo> {
        // 1. 检查权限
        if !self.has_permission() {
            return Err(AppError::ScreenCapture(
                "需要屏幕录制权限".to_string()
            ));
        }

        // 2. 检查磁盘空间
        if self.get_free_space()? < 1_000_000_000 {  // 1GB
            self.cleanup_old_files().await?;
        }

        // 3. 获取活跃应用
        let active_app = self.get_active_app()?;
        if active_app.is_none() {
            tracing::info!("无活跃应用，跳过截图");
            return Err(AppError::ScreenCapture("无活跃应用".to_string()));
        }

        // 4. 执行截图
        let screenshot = self.take_screenshot()?;

        // 5. 压缩处理
        let processed = if screenshot.size > 10_000_000 {  // 10MB
            self.compress(screenshot, 5_000_000)?
        } else {
            screenshot
        };

        // 6. 保存文件
        let file_path = self.save_file(&processed).await?;

        // 7. 创建数据库记���
        let record = self.screenshot_repo.create(ScreenshotRecord {
            file_path,
            file_size: processed.size,
            width: processed.width,
            height: processed.height,
            timestamp: chrono::Utc::now().timestamp(),
            app_name: active_app.map(|a| a.name),
            ..Default::default()
        }).await?;

        // 8. 重置重试计数
        self.retry_count = 0;

        Ok(record.into())
    }

    // 权限检查
    fn has_permission(&self) -> bool {
        #[cfg(target_os = "macos")]
        {
            // macOS 权限检查逻辑
            // 使用 CGPreflightScreenCaptureAccess
            true // 简化示例
        }

        #[cfg(not(target_os = "macos"))]
        {
            true
        }
    }

    // 获取磁盘剩余空间
    fn get_free_space(&self) -> Result<u64> {
        // 实现磁盘空间检查
        Ok(10_000_000_000) // 简化示例
    }

    // 清理旧文件
    async fn cleanup_old_files(&self) -> Result<()> {
        // 删除 30 天前的截图
        tracing::info!("清理旧截图文件");
        Ok(())
    }
}
```

---

## 功能 2: AI 内容分析

### 2.1 功能描述

对截图进行 OCR 识别和 AI 内容理解，提取关键信息，生成结构化的分析结果。

### 2.2 输入规格

| 输入项 | 类型 | 来源 | 约束条件 | 示例 |
|--------|------|------|----------|------|
| 截图 ID | i64 | D1 表 | 必须存在 | 123 |
| 图片文件 | 图片 | 本地存储 | PNG/JPG, <10MB | `/screenshots/...` |
| 应用上下文 | JSON | 数据库 | 可选 | `{app_name: "Chrome", ...}` |

### 2.3 处理逻辑

```
[从队列获取待分析截图]
    ↓
[更新状态: analyzing]
    ↓
[图片预处理]
    ├─ 去噪
    ├─ 增强对比度
    └─ 尺寸标准化
    ↓
[OCR 文字识别]（可选）
    ├─ 调用 OCR API
    ├─ 语言检测
    └─ 文字提取
    ↓
[AI 内容理解]
    ├─ 调用 Claude Vision API
    ├─ 提取关键词
    ├─ 内容分类
    ├─ 生成摘要
    └─ 计算置信度
    ↓
[结构化结果]
    ↓
[更新 D1 记录]
    ↓
[更新状态: completed]
```

### 2.4 输出规格

| 输出项 | 类型 | 格式 | 示例 |
|--------|------|------|------|
| OCR 文本 | String | 纯文本 | `项目进度表\n任务1: 开发...` |
| 内容摘要 | String | 纯文本 | `工作文档-项目进度跟踪` |
| 关键词 | Array | JSON 数��� | `["项目进度", "开发", "测试"]` |
| 分类 | String | 枚举值 | `工作文档` |
| 置信度 | Float | 0.00-1.00 | `0.95` |

### 2.5 状态机

```
[待分析 (pending)]
    ↓ 进入分析队列
[分析中 (analyzing)]
    ↓ AI 分析完成
[分析完成 (completed)]

异常流程:
[分析中] → [失败 (failed)] → [重试 (retry)] → [分析中]
                ↓ 重试 3 次后
            [永久失败 (permanently_failed)]
```

### 2.6 边界条件

| 场景 | 条件 | 处理方式 | 预期结果 |
|------|------|----------|----------|
| **无文字内容** | OCR 识别为空 | 标记为"无文字图片" | 跳过 OCR，仅视觉分析 |
| **API 限流** | 超过调用频率 | 延迟重试 | 等待配额恢复 |
| **API 超时** | 响应时间 > 30s | 重试 | 最多重试 3 次 |
| **图片损坏** | 无法读取 | 标记失败 | 记录错误日志 |
| **低置信度** | confidence < 0.5 | 标记为低质量 | 需要人工审核 |
| **敏感内容** | 检测到隐私信息 | 模糊处理 | 不保存敏感信息 |

### 2.7 性能要求

| 指标 | 目标值 | 测量方法 |
|------|--------|----------|
| AI 分析耗时 | < 5s | 单张图片分析时间 |
| 总耗时 | < 10s | 从入队到完成 |
| 准确率 | > 90% | 人工抽样验证 |
| 并发处理 | 10 张/秒 | 队列吞吐量 |

### 2.8 代码实现示例

```rust
// src-tauri/src/services/ai_processor/mod.rs

use crate::error::{AppError, Result};
use crate::database::repositories::screenshot_repo::ScreenshotRepository;
use super::claude::ClaudeClient;

pub struct AIProcessorService {
    claude_client: ClaudeClient,
    screenshot_repo: ScreenshotRepository,
}

impl AIProcessorService {
    pub async fn analyze_screenshot(
        &self,
        screenshot_id: i64,
    ) -> Result<AnalysisResult> {
        // 1. 获取截图记录
        let screenshot = self.screenshot_repo
            .find_by_id(screenshot_id)
            .await?;

        // 2. 更新状态为 analyzing
        self.screenshot_repo
            .update_status(screenshot_id, "analyzing")
            .await?;

        // 3. 读取图片并转换为 base64
        let image_data = tokio::fs::read(&screenshot.file_path).await?;
        let base64_image = base64::encode(&image_data);

        // 4. 调用 Claude Vision API
        let analysis = self.claude_client
            .analyze_image(
                base64_image,
                Some("分析这张截图，提供：1) 简要描述 2) 关键信息 3) 内容分类".to_string())
            )
            .await
            .map_err(|e| AppError::AIProcessing(e.to_string()))?;

        // 5. 解析 AI 响应并提取结构化信息
        let result = self.parse_analysis_result(&analysis)?;

        // 6. 更新数据库
        self.screenshot_repo.update_analysis(
            screenshot_id,
            &result
        ).await?;

        // 7. 更新状态为 completed
        self.screenshot_repo
            .update_status(screenshot_id, "completed")
            .await?;

        Ok(result)
    }

    fn parse_analysis_result(&self, analysis: &str) -> Result<AnalysisResult> {
        // 解析 AI 返回的文本，提取关键词、分类等
        // 这里需要根据实际 prompt 的响应格式来解析

        Ok(AnalysisResult {
            description: analysis.to_string(),
            keywords: vec!["项目进度".to_string(), "开发".to_string()],
            categories: vec!["工作文档".to_string()],
            confidence: 0.95,
            ..Default::default()
        })
    }
}
```

---

## 功能 3: 短期记忆生成

### 3.1 功能描述

根据一段时间内的截图和应用使用记录，识别用户意图并生成事项记忆片段。

### 3.2 输入规格

| 输入项 | 类型 | 来源 | 约束条件 | 示例 |
|--------|------|------|----------|------|
| 时间窗口 | 时间范围 | 系统配置 | 15分钟 - 2小时 | `[13:00:00, 15:00:00]` |
| 截图列表 | Array | D1 表 | 至少 3 张 | `[1, 2, 3, 4, 5]` |
| 应用记录 | Array | 系统 | 可选 | `[{app: "Chrome", duration: 3600}]` |
| 用户 Todo | Array | 用户输入 | 可选 | `["写需求文档", "开发功能"]` |

### 3.3 处理逻辑

```
[检测触发条件]
    ├─ 应用切换
    ├─ 时间窗口结束
    └─ 用户主动触发
    ↓
[收集时间窗口内的数据]
    ├─ 截图列表 (D1)
    ├─ 应用使用记录
    └─ 用户 Todo 列表
    ↓
[意图识别]
    ├─ 分析截图内容
    ├─ 分析应用使用模式
    ├─ 匹配 Todo 列表
    └─ 识别事项类型
    ↓
[生成记忆片段]
    ├─ 提取关键点
    ├─ 生成总结
    └─ 计算时长
    ↓
[创建 D2 记录]
    ↓
[关联截图]
```

### 3.4 输出规格

| 输出项 | 类型 | 格式 | 示例 |
|--------|------|------|------|
| 记忆 ID | i64 | 数据库主键 | `1` |
| 意图类型 | String | 枚举值 | `工作任务-需求分析` |
| 模型总结 | String | 纯文本 | `这个���务涉及新功能的需求分析...` |
| 关键点 | Array | JSON 数组 | `["查看 PRD", "绘制流程图"]` |
| 截图范围 | Object | JSON 对象 | `{start: 1, end: 5}` |

### 3.5 边界条件

| 场景 | 条件 | 处理方式 | 预期结果 |
|------|------|----------|----------|
| **截图过少** | < 3 张 | 不生成记忆 | 等待更多数据 |
| **时间过短** | < 5 分钟 | 合并到下一个窗口 | 延迟生成 |
| **意图不明确** | 置信度 < 0.6 | 标记为"未分类活动" | 需要用户确认 |
| **多个意图** | 检测到 2+ 个事项 | 拆分为多条记忆 | 创建多条 D2 记录 |
| **被打断** | 中间有长时间空白 | 标记打断点 | 生成打断提醒 |

### 3.6 代码实现示例

```rust
// src-tauri/src/services/memory/short_term.rs

use crate::error::Result;
use crate::database::repositories::{
    screenshot_repo::ScreenshotRepository,
    short_memory_repo::ShortMemoryRepository,
};
use crate::services::ai_processor::AIProcessorService;

pub struct ShortTermMemoryService {
    screenshot_repo: ScreenshotRepository,
    short_memory_repo: ShortMemoryRepository,
    ai_processor: AIProcessorService,
}

impl ShortTermMemoryService {
    pub async fn generate_memory(
        &self,
        time_range: (i64, i64),
    ) -> Result<ShortTermMemory> {
        // 1. 收集时间窗口内的截图
        let screenshots = self.screenshot_repo
            .find_by_time_range(time_range.0, time_range.1)
            .await?;

        // 边界条件：截图过少
        if screenshots.len() < 3 {
            return Err(AppError::InvalidInput(
                "截图数量不足，无法生成记忆".to_string()
            ));
        }

        // 2. 提取所有截图的分析内容
        let descriptions: Vec<String> = screenshots
            .iter()
            .filter_map(|s| s.ai_description.clone())
            .collect();

        // 3. 调用 AI 生成总结
        let summary_prompt = format!(
            "根据以下一段时间内的活动记录，生成简洁的总结和关键点：\n{}",
            descriptions.join("\n---\n")
        );

        let summary = self.ai_processor
            .generate_summary(summary_prompt)
            .await?;

        // 4. 识别意图类型
        let intent_type = self.classify_intent(&summary)?;

        // 5. 创建短期记忆记录
        let memory = self.short_memory_repo.create(ShortMemoryRecord {
            topic: intent_type.clone(),
            summary,
            screenshot_start_id: screenshots.first().unwrap().id,
            screenshot_end_id: screenshots.last().unwrap().id,
            relevance_score: 0.85,
            created_at: chrono::Utc::now().timestamp(),
            expires_at: Some(chrono::Utc::now().timestamp() + 7 * 24 * 3600), // 7天后过期
            ..Default::default()
        }).await?;

        Ok(memory)
    }

    fn classify_intent(&self, summary: &str) -> Result<String> {
        // 简单的意图分类逻辑
        if summary.contains("文档") || summary.contains("需求") {
            Ok("工作任务-文档编写".to_string())
        } else if summary.contains("代码") || summary.contains("开发") {
            Ok("工作任务-代码开发".to_string())
        } else {
            Ok("未分类活动".to_string())
        }
    }
}
```

---

## 功能 4: 主动推送提醒

### 4.1 功能描述

根据用户行为模式，主动推送温馨提醒、合理建议和打断衔接，提升工作效率和健康。

### 4.2 输入规格

| 输入项 | 类型 | 来源 | 约束条件 | 示例 |
|--------|------|------|----------|------|
| 用户行为数据 | 实时流 | 系统监控 | 持续监控 | 应用使用记录 |
| 历史记忆 | 数据库 | D2/D3 表 | 最近 7 天 | 工作模式分析 |
| 推送规则 | 配置 | 系统设置 | 可自定义 | 工作 1.5 小时提醒 |

### 4.3 Case 1: 温馨提醒

**处理逻辑**:

```
[持续监控工作时长]
    ↓
[检测连续工作时间]
    ↓ 超过阈值 (默认 1.5 小时)
[生成提醒内容]
    ├─ 计算工作时长
    ├─ 分析工作强度
    └─ 生成个性化文案
    ↓
[检查推送时机]
    ├─ 用户不在输入状态
    ├─ 不在会议中
    └─ 不在专注模式
    ↓ 时机合适
[推送提醒]
    ↓
[记录推送日志]
```

**推送规则**:

| 触发条件 | 阈值 | 提醒内容模板 | 推送方式 |
|----------|------|--------------|----------|
| 连续工作 | 1.5 小时 | "你已经连续工作 {duration}，该休息一下了" | 桌面通知 |
| 高强度工作 | 键盘输入 >500 次/10 分钟 | "检测到高强度工作，注意劳逸结合" | 桌面通知 |
| 久坐提醒 | 2 小时无离开 | "已经坐了 {duration}，站起来活动一下吧" | 桌面通知 |

### 4.4 Case 2: 合理建议

**处理逻辑**:

```
[分析完成的事项]
    ↓
[提取工作模式]
    ├─ 使用的工具
    ├─ 操作流程
    └─ 耗时分布
    ↓
[匹配优化建议库]
    ↓ 找到匹配建议
[生成个性化建议]
    ↓
[推送建议]
```

**建议规则**:

| 检测场景 | 触发条件 | 建议内容 | 示例 |
|----------|----------|----------|------|
| 工具切换频繁 | 5 分钟内切换 >10 次 | 推荐使用多屏幕/分屏工具 | "检测到频繁切换窗口，试试分屏？" |
| 重复操作 | 相同操作 >5 次 | 推荐使用快捷键/自动化 | "这个操作可以用快捷键 Ctrl+X" |
| 低效流程 | 耗时 > 预期 2 倍 | 推荐优化流程 | "这个流程可以这样优化..." |

### 4.5 Case 3: 打断衔接

**处理逻辑**:

```
[检测用户回归]
    ↓
[分析离开前的状态]
    ├─ 最后使用的应用
    ├─ 未完成的事项
    └─ 工作进度
    ↓
[生成衔接提醒]
    ↓
[推送提醒]
```

**打断检测规则**:

| 场景 | 检测条件 | 提醒内容 | 推送时机 |
|------|----------|----------|----------|
| 短暂离开 | 5-15 分钟无操作 | "刚才在进行 {task}，继续吗？" | 回归后立即 |
| 长时间离开 | >30 分钟无操作 | "离开前在 {app} 做 {task}，需要回顾吗？" | 回归后 30 秒 |
| 被打断 | 突然切换到通讯工具 | "刚才被打断了，之前在 {task}" | 打断结束后 |

### 4.6 输出规格

| 输出项 | 类型 | 格式 | 示例 |
|--------|------|------|------|
| 推送消息 | 通知 | 桌面通知 | 标题 + 内容 + 操作按钮 |
| 推送日志 | 数据库记录 | JSON | 记录推送时间、类型、用户反馈 |

### 4.7 边界条件

| 场景 | 条件 | 处理方式 | 预期结果 |
|------|------|----------|----------|
| **用户正在输入** | 键盘活跃 | 延迟推送 | 等待输入结束 |
| **全屏应用** | 检测到全屏 (游戏/视频) | 不推送 | 避免打扰 |
| **专注模式** | 用户开启勿扰 | 不推送 | 记录待推送列表 |
| **推送过频** | 10 分钟内已推送 | 不推送 | 避免骚扰 |
| **用户忽略** | 连续 3 次不响应 | 降低推送频率 | 学习用户偏好 |

### 4.8 代码实现示例

```rust
// src-tauri/src/services/notification/mod.rs

use tauri::{AppHandle, Manager};

pub struct NotificationService {
    app_handle: AppHandle,
}

impl NotificationService {
    pub async fn send_rest_reminder(&self, duration_minutes: u32) -> Result<()> {
        // 检查推送时机
        if !self.is_appropriate_time().await? {
            tracing::info!("当前不适合推送，延迟");
            return Ok(());
        }

        // 生成提醒内容
        let title = "该���息一下了".to_string();
        let body = format!(
            "你已经连续工作 {} 分钟，休息一下吧！",
            duration_minutes
        );

        // 发送桌面通知
        self.app_handle.emit_all("notification:show", Notification {
            id: uuid::Uuid::new_v4().to_string(),
            notification_type: "rest_reminder".to_string(),
            title,
            body,
            actions: vec![
                NotificationAction {
                    label: "休息 5 分钟".to_string(),
                    action: "rest_5min".to_string(),
                },
                NotificationAction {
                    label: "忽略".to_string(),
                    action: "dismiss".to_string(),
                },
            ],
        })?;

        Ok(())
    }

    async fn is_appropriate_time(&self) -> Result<bool> {
        // 检查用户是否在输入
        // 检查是否在会议中
        // 检查是否在专注模式
        Ok(true) // 简化示例
    }
}
```

---

## 相关文档

- [数据库设计](./database-design.md) - D1/D2/D3 表结构
- [API 接口文档](./api-reference.md) - Tauri Commands API
- [后端架构](./backend-architecture.md) - ��体架构设计
- [验收标准](../development/acceptance-criteria.md) - 功能验收标准
