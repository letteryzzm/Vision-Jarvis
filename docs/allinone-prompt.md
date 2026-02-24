# 屏幕录制分析 All-in-One Prompt

用于 `recording_understanding_prompt()` 函数，替换现有 V1 prompt。

---

## Prompt 正文

```
分析这段屏幕录制视频，一次性提取所有结构化信息。严格按JSON格式返回，不要包含任何其他文字。

{
  "application": "主要使用的应用程序名称（如 VS Code、Chrome、微信、Terminal）",
  "window_title": "当前窗口或标签页标题（如 main.rs - vision-jarvis、GitHub - Pull Requests），无法识别返回null",
  "url": "当前访问的URL（仅浏览器场景），非浏览器返回null",

  "activity_category": "work|entertainment|communication|learning|other",
  "productivity_score": 7,
  "focus_level": "deep|normal|fragmented",
  "interaction_mode": "typing|reading|navigating|watching|idle|mixed",
  "is_continuation": false,

  "activity_description": "用户在这段时间内做了什么（一句话，现在时态，要具体，如\"在VS Code中调试Rust内存管理模块\"）",
  "activity_summary": "这段时间的活动概述，2-3句（供时间线展示用）",
  "accomplishments": ["完成了XX功能", "修复了YY问题"],

  "key_elements": ["其他关键元素（排除URL和文件名）"],
  "context_tags": ["rust", "debugging", "memory-system"],
  "project_name": "项目名称，无法识别返回null",
  "people_mentioned": ["张三", "Alice"],
  "technologies": ["rust", "tauri", "sqlite"],

  "ocr_text": "屏幕上的重要文字内容（仅提取有意义的部分，如错误信息、标题、关键代码）",
  "file_names": ["main.rs", "schema.rs", "README.md"],
  "error_indicators": ["error[E0382]: use of moved value", "test failed: assertion failed"]
}

字段说明：

【应用识别】
- application: 识别录制中主要使用的应用程序
- window_title: 提取窗口标题栏或浏览器标签页标题，有助于识别具体内容
- url: 浏览器地址栏中的URL（完整URL或域名均可）

【活动分类】
- activity_category: 只能是 work / entertainment / communication / learning / other 之一
  * work: 编程、写作、设计、分析等生产性工作
  * entertainment: 看视频、游戏、刷社交媒体等娱乐
  * communication: 邮件、聊天、视频会议、社区讨论
  * learning: 看教程、阅读文档、学习课程
  * other: 系统设置、文件管理等其他操作
- productivity_score: 整数1-10（1=纯娱乐 5=一般工作 8=高效工作 10=深度专注）
- focus_level: 专注程度
  * deep: 持续专注于单一任务，无明显切换
  * normal: 正常工作节奏，偶尔切换
  * fragmented: 频繁切换应用或标签，注意力分散
- interaction_mode: 主要交互方式
  * typing: 主要在输入文字（编码、写作）
  * reading: 主要在阅读内容（文档、代码、网页）
  * navigating: 主要在点击/浏览/操作界面
  * watching: 主要在观看视频/演示
  * idle: 屏幕静止，无明显操作
  * mixed: 多种交互方式混合
- is_continuation: 此分段是否明显是上一个分段同一任务的延续（true/false，无上下文时填false）

【活动描述】
- activity_description: 简洁描述用户正在做什么，动词开头，具体到操作层面
- activity_summary: 更详细的活动概述，适合展示在时间线上
- accomplishments: 这段时间的具体成果（1-3条），无明显成果填空数组[]

【上下文增强】
- key_elements: 其他重要元素（不含URL和文件名，如窗口名称、功能区域、特殊界面元素）
- context_tags: 2-5个描述当前情境的标签（英文小写，如 debugging、code-review、meeting）
- project_name: 识别到的项目名称（如 vision-jarvis、论文写作、个人博客），无法识别填null
- people_mentioned: 出现的人名（邮件收件人、PR作者、会议参与者、聊天对象等），空则填[]
- technologies: 识别到的技术栈（编程语言、框架、工具），如 rust、react、docker，空则填[]

【内容提取】
- ocr_text: 屏幕上有意义的文字（优先提取：错误信息、重要标题、关键配置，不要提取无意义的UI文字）
- file_names: 识别到的文件名（编辑器标签、终端路径、文件浏览器中的文件），空则填[]
- error_indicators: 识别到的错误或异常信息（编译错误、测试失败、运行异常），空则填[]

只返回JSON对象，不要包含任何解释文字、markdown代码块标记或其他内容。
```

---

## 字段与下游组件对照表

| 字段                     | activity_grouper | project_extractor | habit_detector | summary_generator | markdown_generator |
| ------------------------ | :--------------: | :---------------: | :-------------: | :---------------: | :----------------: |
| `application`          |    ✅ 分组键    |        —        |       —       |        —        |         ✅         |
| `window_title`         |        —        |    ✅ 辅助识别    |       —       |        —        |    ✅ 丰富输出    |
| `url`                  |        —        |        —        |       —       |        —        |         ✅         |
| `activity_category`    |    ✅ 分组键    |        —        |       ✅       |        —        |         —         |
| `productivity_score`   |       存储       |        —        |       ✅       |        ✅        |         —         |
| `focus_level`          |        —        |        —        | ✅ 深度工作识别 |        ✅        |         —         |
| `interaction_mode`     |        —        |        —        |   ✅ 行为模式   |        —        |         —         |
| `is_continuation`      |   ✅ 合并信号   |        —        |       —       |        —        |         —         |
| `activity_description` |   ✅ 标题生成   |        —        |       —       |        —        |         ✅         |
| `activity_summary`     |    ✅ 时间线    |        —        |       —       |        ✅        |         ✅         |
| `accomplishments`      |        —        |        —        |       —       |    ✅ 核心输入    |         ✅         |
| `key_elements`         |     ✅ 合并     |        —        |       —       |        —        |         —         |
| `context_tags`         |   ✅ 重叠评分   |        —        |       ✅       |        —        |         —         |
| `project_name`         |       存储       |    ✅ 核心输入    |       —       |        ✅        |         ✅         |
| `people_mentioned`     |        —        |        —        |       —       |    ✅ 协作记录    |         ✅         |
| `technologies`         |        —        |   ✅ 技术栈识别   |       —       |        ✅        |         —         |
| `ocr_text`             |        —        |        —        |       —       |        —        |         —         |
| `file_names`           |        —        |    ✅ 文件关联    |       —       |        —        |         —         |
| `error_indicators`     |        —        |        —        |   ✅ 调试习惯   |        —        |         —         |

---

## 枚举值速查

```
activity_category: work | entertainment | communication | learning | other
focus_level:       deep | normal | fragmented
interaction_mode:  typing | reading | navigating | watching | idle | mixed
productivity_score: 1–10 (整数)
is_continuation:   true | false
```

---

## 注意事项

1. **所���可选字段**（`window_title`、`url`、`project_name`、`ocr_text`）在无法识别时填 `null`，不要省略字段
2. **所有数组字段**在无内容时填 `[]`，不要省略字段
3. **is_continuation** 在没有上下文时一律填 `false`
4. **file_names** 只提取实际文件名，不含路径前缀（`main.rs` 而非 `/home/user/main.rs`）
5. **error_indicators** 只提取明确的错误信息，不要将警告或普通日志归入此字段
