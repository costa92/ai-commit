# Git 提交规范模板（Conventional Commits）

本项目采用 [Conventional Commits](https://www.conventionalcommits.org/zh-hans/v1.0.0/) 规范，要求所有提交信息遵循如下格式：

```
<type>(<scope>): <subject>

<body>
```

- `type`：提交类型（必填），如：
  - feat：新功能
  - fix：修复 bug
  - docs：文档变更
  - style：代码格式（不影响功能，例如空格、分号等）
  - refactor：重构（即不是新增功能，也不是修复 bug）
  - test：增加测试
  - chore：构建过程或辅助工具的变动
- `scope`：影响范围（可选），如模块名、文件名等
- `subject`：简要描述（必填），建议不超过 50 字符
- `body`：详细描述（可选）

## 示例

```
feat(parser): add ability to parse arrays

Support parsing arrays in the new parser module.
```

```
fix(login): correct password validation logic
```

```
docs(readme): update usage instructions
```

---

更多详细规范请参考：[Conventional Commits 中文文档](https://www.conventionalcommits.org/zh-hans/v1.0.0/) 