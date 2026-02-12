# 测试实例管理

## 强制清理规则 ⚠️

**CRITICAL**: 如果在会话中**主动启动**了任何测试实例（Tauri、Vite、npm dev 等），必须在对话结束前**只关闭自己启动的实例**。

**重要**: 不要关闭用户在其他终端自己启动的实例！

## 实例跟踪方法

### 正确做法：记录 PID

当使用 `run_in_background: true` 启动进程时：
- ✅ 会返回 `task_id`
- ✅ 使用 `TaskStop` 工具关闭该任务
- ✅ 只影响当前对话启动的实例

```python
# 启动时
Bash(command="npm run tauri:dev", run_in_background=true)
# 返回: task_id: "abc123"

# 关闭时
TaskStop(task_id="abc123")  # 只关闭这个实例
```

### 错误做法：全局杀死

❌ **永远不要使用**：
```bash
pkill -9 -f "vision-jarvis"  # 会杀死用户的实例！
killall vision-jarvis        # 会杀死用户的实例！
```

## 何时需要清理

只在以下情况需要清理：
- 使用 `Bash` 工具 + `run_in_background: true` 启动了进程
- 手动启动了后台任务用于测试

## 清理检查清单

对话结束前：

1. **检查是否启动了后台任务**
   - 查看对话历史中的 `task_id`
   - 使用 `TaskOutput` 检查任务状态

2. **只关闭自己的任务**
   ```python
   TaskStop(task_id="记录的task_id")
   ```

3. **验证清理**
   - 使用 `TaskOutput` 确认任务已停止
   - 不要使用 `ps` 或 `pkill` 全局检查

## 最佳实践

### ✅ 推荐方式

1. **前台启动**（用户可以 Ctrl+C）
   ```bash
   npm run tauri:dev  # 不使用 run_in_background
   ```

2. **记录 task_id**
   ```python
   # 启动时记录
   task_id = Bash(command="...", run_in_background=true)
   # 在对话中保存 task_id

   # 关闭时使用
   TaskStop(task_id=task_id)
   ```

### ❌ 避免的做法

1. 不要全局杀进程
2. 不要关闭端口（用户可能在用）
3. 不要假设所有实例都是你启动的

## 示例：正确的启动和关闭流程

```python
# === 会话开始 ===

# 1. 启动测试实例
result = Bash(
    command="npm run tauri:dev",
    run_in_background=true,
    description="启动 Tauri 开发服务器用于测试"
)
# 记录返回的 task_id: "abc123"

# 2. 进行测试...

# === 会话结束前 ===

# 3. 只关闭自己启动的实例
TaskStop(task_id="abc123")

# 4. 验证
TaskOutput(task_id="abc123", block=false)
# 应该显示任务已停止
```

## 如何判断实例是否是自己启动的

**简单规则**:
- 如果你没有在当前对话中调用 `Bash` 工具启动它 → **不要管它**
- 如果用户说"我在终端启动了..." → **不要关闭它**
- 只关闭你明确记录了 `task_id` 的后台任务

## 端口冲突处理

如果用户报告端口冲突：
1. ✅ 提示用户检查是否有其他实例在运行
2. ✅ 提供手动检查命令让用户执行
3. ❌ 不要自动杀死占用端口的进程

```bash
# 提供给用户的检查命令
lsof -i:4321  # 让用户自己看是什么进程
```

---

**核心原则**: **只管理自己启动的，不要碰用户的实例！**
