# ai-commit CI/CD 构建与发布说明

本项目使用 GitHub Actions 自动构建并发布多平台二进制文件到 Release 页面。

---

## 一、CI 配置核心片段

```yaml
env:
  CARGO_TERM_COLOR: always

jobs:
  release:
    name: release ${{ matrix.target }}
    strategy:
      fail-fast: false
      matrix:
        include:
          - target: x86_64-pc-windows-gnu
            archive: zip
            os: ubuntu-latest
          - target: x86_64-unknown-linux-musl
            archive: tar.gz tar.xz tar.zst
            os: ubuntu-latest
          - target: x86_64-apple-darwin
            archive: zip
            os: macos-latest
          - target: aarch64-apple-darwin
            archive: zip
            os: macos-latest
    runs-on: ${{ matrix.os }}
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      # Windows 交叉编译依赖
      - name: Install MinGW for Windows cross-compilation
        if: matrix.target == 'x86_64-pc-windows-gnu' && runner.os == 'Linux'
        run: |
          sudo apt-get update
          sudo apt-get install -y mingw-w64

      # Linux musl 静态编译依赖
      - name: Install cross
        if: matrix.target == 'x86_64-unknown-linux-musl' && runner.os == 'Linux'
        run: cargo install cross --force

      # 构建步骤
      - name: Build release binary
        if: matrix.target != 'x86_64-unknown-linux-musl'
        run: |
          cargo build --release --target ${{ matrix.target }}

      - name: Build with cross (musl)
        if: matrix.target == 'x86_64-unknown-linux-musl' && runner.os == 'Linux'
        run: |
          cross build --release --target x86_64-unknown-linux-musl

      # 归档产物
      - name: Create release archive
        id: create_archive
        run: |
          # 归档逻辑略，见实际 workflow

      - name: List workspace files for debug
        run: ls -lh ${{ github.workspace }}

      # 上传 Release 资产
      - name: Upload Release Assets
        uses: ncipollo/release-action@v1
        if: startsWith(github.ref, 'refs/tags/')
        with:
          artifacts: ${{ steps.create_archive.outputs.paths }}
          allowUpdates: true
          token: ${{ secrets.GITHUB_TOKEN }}
```

---

## 二、不同平台构建方式说明

- **Linux (musl 静态编译)**：
  - 需用 `cross build --release --target x86_64-unknown-linux-musl`
  - 不要用 `cargo build`，否则会因缺少 musl 工具链或 ring 等依赖报错
- **macOS (Intel/M1/M2)**：
  - 直接用 `cargo build --release --target x86_64-apple-darwin` 或 `aarch64-apple-darwin`
- **Windows**：
  - 需先安装 `mingw-w64`，再用 `cargo build --release --target x86_64-pc-windows-gnu`

---

## 三、常见错误与解决方法

1. **ring/openssl 相关 musl 报错**
   - 错误：`failed to find tool "x86_64-linux-musl-gcc"` 或 `ring build failed`
   - 解决：musl 目标必须用 cross，不要用 cargo build

2. **Release 资产上传 422 already_exists**
   - 错误：`Validation Failed: already_exists` 或 `release failed with status: 422`
   - 解决：用 ncipollo/release-action@v1 并加 `allowUpdates: true`，不要用 softprops/action-gh-release

3. **找不到产物文件**
   - 错误：`does not match any files`
   - 解决：确认归档步骤产物路径与上传参数一致，上传前用 `ls` 检查

4. **Windows 交叉编译失败**
   - 错误：找不到 mingw 相关工具
   - 解决：确保安装 `mingw-w64`，并 runner 用 ubuntu-latest

---

## 四、注意事项

- 不同平台的构建方式不要混用，musl 目标只用 cross，其他平台用 cargo build
- 上传 Release 资产时 artifacts 路径要与实际生成文件一致
- 推荐每次发布 tag 前先清理旧 Release，避免历史产物干扰

---

如需更多 CI/CD 优化建议或遇到特殊平台问题，欢迎提 issue 交流！ 