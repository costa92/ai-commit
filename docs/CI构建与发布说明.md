# ai-commit CI/CD 构建与发布说明

本项目使用 GitHub Actions 自动构建并发布多平台二进制文件到 Release 页面，支持 Linux（musl 静态编译）、macOS（Intel/ARM）、Windows。

---

## 一、CI 配置核心流程

CI 配置文件为 `.github/workflows/release.yml`，主要流程如下：

- 触发方式：
  - 推送符合 `v*.*.*` 格式的 tag 时自动触发
  - 支持手动 workflow_dispatch
- 权限：
  - `contents: write`，用于发布 Release 资产
- 平台矩阵：
  - x86_64-pc-windows-gnu（zip，ubuntu-latest）
  - x86_64-unknown-linux-musl（tar.gz/tar.xz/tar.zst，ubuntu-latest）
  - x86_64-apple-darwin（zip，macos-latest）
  - aarch64-apple-darwin（zip，macos-latest）

### 主要步骤说明

1. **依赖缓存**
   - 使用 actions/cache 缓存 cargo 依赖和 target 目录，加速构建
2. **依赖安装**
   - Windows 交叉编译：安装 mingw-w64
   - Linux musl：安装 musl-tools 和 cross
3. **Rust 工具链安装**
   - actions-rs/toolchain 安装 stable 工具链和目标平台 target
4. **构建**
   - musl 目标用 cross build，其它平台用 cargo build
   - Linux 构建时设置 OPENSSL_STATIC=1 以兼容 ring/openssl
5. **测试与格式检查**
   - 除 Windows 外均运行 cargo test
   - 所有平台运行 cargo fmt -- --check
6. **产物准备与归档**
   - 产物名格式：ai-commit-<版本>-<平台>，包含可执行文件、README.md、CHANGELOG.md
   - Windows 为 .exe，其它平台为无扩展名
   - 支持 zip、tar.gz、tar.xz、tar.zst 多种归档格式
7. **校验和生成**
   - 自动生成 SHA256 校验和，兼容 macOS（shasum -a 256）和 Linux（sha256sum）
8. **上传 Release 资产**
   - 使用 ncipollo/release-action@v1 上传所有归档和校验和，支持多次上传（allowUpdates: true）

---

## 二、关键代码说明

### 1. 产物命名与路径
- 产物基础名：`ai-commit-${{ github.ref_name }}-${{ matrix.target }}`
- 可执行文件路径：`target/${{ matrix.target }}/release/ai-commit`（Windows 为 ai-commit.exe）
- 归档内容：主程序、README.md、CHANGELOG.md

### 2. 归档脚本
- 支持 zip、tar.gz、tar.xz、tar.zst 多种格式：
  ```bash
  zip -r "${ARTIFACT_BASE_NAME}.zip" $FILES_TO_ARCHIVE
  tar -czf "${ARTIFACT_BASE_NAME}.tar.gz" $FILES_TO_ARCHIVE
  tar -cJf "${ARTIFACT_BASE_NAME}.tar.xz" $FILES_TO_ARCHIVE
  tar --zstd -cf "${ARTIFACT_BASE_NAME}.tar.zst" $FILES_TO_ARCHIVE
  ```
- 归档文件统一输出到 `${{ github.workspace }}` 目录

### 3. 校验和生成
- 自动检测 sha256sum 或 shasum -a 256：
  ```bash
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum ai-commit-*.* > ai-commit-SHA256SUMS.txt
  else
    shasum -a 256 ai-commit-*.* > ai-commit-SHA256SUMS.txt
  fi
  ```
- 校验和文件会自动上传到 Release

### 4. 关键变量与输出
- 通过 `echo "key=value" >> $GITHUB_OUTPUT` 传递产物名、路径等变量
- 归档产物路径通过 `steps.create_archive.outputs.paths` 传递给上传步骤

### 5. 失败处理与调试
- 构建失败时自动列出 release 目录下所有文件，便于排查
- 若找不到可执行文件会主动报错并输出目录内容

---

## 三、不同平台构建方式说明

- **Linux (musl 静态编译)**：
  - 需用 cross build --release --target x86_64-unknown-linux-musl
  - 需先安装 musl-tools 和 cross
- **macOS (Intel/M1/M2)**：
  - 直接用 cargo build --release --target x86_64-apple-darwin 或 aarch64-apple-darwin
- **Windows**：
  - 需先安装 mingw-w64，再用 cargo build --release --target x86_64-pc-windows-gnu

---

## 四、常见错误与解决方法

1. **ring/openssl 相关 musl 报错**
   - 错误：failed to find tool "x86_64-linux-musl-gcc" 或 ring build failed
   - 解决：musl 目标必须用 cross，不要用 cargo build，并确保安装 musl-tools

2. **Release 资产上传 422 already_exists**
   - 错误：Validation Failed: already_exists 或 release failed with status: 422
   - 解决：用 ncipollo/release-action@v1 并加 allowUpdates: true

3. **找不到产物文件**
   - 错误：does not match any files
   - 解决：确认归档步骤产物路径与上传参数一致，上传前用 ls 检查

4. **Windows 交叉编译失败**
   - 错误：找不到 mingw 相关工具
   - 解决：确保安装 mingw-w64，runner 用 ubuntu-latest

5. **macOS 校验和命令不兼容**
   - 解决：自动检测 sha256sum 或 shasum -a 256，已在 workflow 兼容

---

## 五、注意事项

- 不同平台的构建方式不要混用，musl 目标只用 cross，其它平台用 cargo build
- 上传 Release 资产时 artifacts 路径要与实际生成文件一致
- 每次发布 tag 前建议清理旧 Release，避免历史产物干扰
- 产物归档内务必包含 README.md、CHANGELOG.md 和主程序
- 校验和文件（ai-commit-SHA256SUMS.txt）会自动上传到 Release

---

如需更多 CI/CD 优化建议或遇到特殊平台问题，欢迎提 issue 交流！ 