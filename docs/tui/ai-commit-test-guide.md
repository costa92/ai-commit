# AI Commit 功能测试指南

## 测试步骤

### 1. 打开 Demo
```bash
open docs/tui/tui-demo.html
```

### 2. 测试 AI Commit 功能的三种入口

#### 方式 1：从 Status 视图进入
1. 按数字键 `4` 切换到 **Status** 视图
2. 点击 **"AI Commit"** 按钮
3. AI Commit 模态框应该打开

#### 方式 2：使用全局快捷键
1. 在任何视图下，按 `Ctrl+A`
2. AI Commit 模态框应该直接打开

#### 方式 3：从 Status 视图的按钮
1. 切换到 Status 视图
2. 点击顶部的 "AI Commit" 按钮

### 3. AI Commit 界面交互测试

#### 3.1 初始状态
打开 AI Commit 后应该看到：
- **AI Provider 选择器**：默认选中 Ollama
- **Model 选择器**：默认选中 Mistral
- **Changes Summary**：显示文件变更摘要
  - Files: 3 changed, +45 -12 lines
  - 列出具体文件路径和状态

#### 3.2 生成 AI 建议
1. 点击 **"Generate Suggestions"** 按钮
2. 观察进度条动画（从 0% 到 100%）
3. 完成后显示 3 个 AI 生成的建议

#### 3.3 选择建议
1. 点击任意一个建议卡片
2. 被选中的建议应该高亮显示（蓝色背景）
3. **"Use Selected"** 按钮应该出现

#### 3.4 使用建议
1. 选择一个建议后
2. 点击 **"Use Selected"** 按钮
3. 应该弹出提示：`Using AI suggestion X for commit`
4. 模态框自动关闭

### 4. AI Provider 切换测试
1. 打开 AI Commit 模态框
2. 切换 Provider 下拉框：
   - Ollama (Local)
   - Deepseek
   - SiliconFlow
   - Kimi
3. 切换 Model 下拉框：
   - Mistral
   - Llama 2
   - Code Llama

### 5. 键盘操作测试
- `Esc` - 关闭 AI Commit 模态框
- `Ctrl+A` - 从任何地方打开 AI Commit

### 6. 完整工作流测试

#### 标准 AI Commit 流程：
1. 按 `4` 切换到 Status 视图
2. 按 `Ctrl+A` 打开 AI Commit
3. 选择 AI Provider 和 Model
4. 点击 "Generate Suggestions"
5. 等待进度条完成
6. 查看 3 个生成的建议：
   - **建议 1**: `fix(ui): 重构侧边栏布局，解决信息层次混乱问题 ⭐`
   - **建议 2**: `fix(tui): 修复侧边栏组件布局和焦点管理问题`
   - **建议 3**: `refactor(layout): 优化TUI布局管理系统`
7. 点击选择建议 1（带星标的推荐）
8. 点击 "Use Selected"
9. 确认提示信息
10. 模态框关闭

### 7. 边界情况测试

#### 7.1 取消操作
1. 打开 AI Commit
2. 点击 "Generate Suggestions"
3. 在生成过程中点击 "Cancel"
4. 模态框应该关闭

#### 7.2 重新生成
1. 生成建议后
2. 不选择任何建议
3. 再次点击 "Generate Suggestions"
4. 应该重新显示进度条并生成新建议

#### 7.3 切换 Provider 后生成
1. 切换到不同的 Provider
2. 点击 "Generate Suggestions"
3. 应该正常生成（模拟）

## 预期结果

✅ **成功标准**：
- AI Commit 模态框能正常打开/关闭
- 进度条动画流畅
- 建议能够被选择和使用
- 所有按钮和下拉框响应正常
- 键盘快捷键工作正常

## 视觉效果检查

1. **模态框样式**：
   - 深色背景遮罩
   - 模态框居中显示
   - 蓝色标题栏

2. **进度条动画**：
   - 蓝色填充
   - 平滑过渡

3. **建议卡片**：
   - 鼠标悬停效果
   - 选中高亮（蓝色背景）
   - 星标推荐显示

4. **按钮状态**：
   - "Use Selected" 初始隐藏
   - 选择建议后显示

## 常见问题

**Q: AI Commit 模态框打不开？**
A: 确保在浏览器中打开 HTML 文件，检查 JavaScript 控制台是否有错误。

**Q: 进度条不动？**
A: 这是模拟的进度条，会在 2 秒内自动完成。

**Q: 选择建议后没有反应？**
A: 确保点击整个建议卡片区域，查看是否有蓝色高亮。

## 测试记录

| 测试项 | 状态 | 备注 |
|-------|------|------|
| 打开 AI Commit | ✅ | Ctrl+A 和按钮都可以 |
| Provider 切换 | ✅ | 4 个 provider 选项 |
| Model 切换 | ✅ | 3 个 model 选项 |
| 生成建议 | ✅ | 进度条动画正常 |
| 选择建议 | ✅ | 高亮显示正常 |
| 使用建议 | ✅ | 弹出提示并关闭 |
| 取消操作 | ✅ | ESC 键和 Cancel 按钮 |
| 文件显示 | ✅ | 显示 3 个文件变更 |