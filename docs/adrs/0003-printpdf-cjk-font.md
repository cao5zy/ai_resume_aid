# ADR 0003: PDF 生成——printpdf + 嵌入式 CJK 字体的选择

## 状态

已采纳（2026-07-01）

## 上下文

工具需要为用户提供将优化后的简历导出为 PDF 的功能。PDF 内容为中文，需要正确渲染 CJK 字符和常用符号（如子弹字符 `•`）。

## 决策

- PDF 生成库：Rust 的 `printpdf` crate
- CJK 字体：Noto Sans CJK SC Regular 子集字体（嵌入式 OTF，≈11MB）

## 理由

### printpdf（vs 其他 Rust PDF crate）
- `printpdf` 是少数纯 Rust PDF 库之一，无需链接 C 库或系统依赖
- API 底层直接操作 PDF 图形原语，控制力强
- 支持嵌入外部字体（对 CJK 渲染至关重要）
- 相比 `genpdf`（高层封装），printpdf 的底层 API 更适合精确控制排版
- 相比 `wkhtmltopdf` 或 headless Chrome 方案，printpdf 无额外运行时依赖

### 嵌入式 CJK 字体（vs 系统字体，vs 不嵌入）
- 系统字体渲染：不同 OS 字体不同（macOS=STSong/NISC18030，Linux=Noto，Windows=SimSun），行为不一致
- 不嵌入字体：PDF 阅读器可能无法找到 CJK 字体 → 中文显示为方框或空白
- 嵌入式子集字体：文件体积适中（11MB），一次嵌入即保证在所有阅读器中正确显示
- 子集（subset）而非全量：完整 Noto Sans CJK SC ≈ 16MB，子集缩小至 11MB

### Noto Sans CJK SC（vs 其他中文字体）
- 开源（SIL OFL），允许嵌入和分发
- 字形覆盖完整（CJK Unified Ideographs + 常用符号 + ASCII）
- 无版权问题

## 后果

- 生成 PDF 文件较大（≈11MB），主要来自嵌入字体
- 字体更新需重新运行子集生成脚本（见 `scripts/subset-font.sh`）
- `printpdf` 对 OTF CFF 字体的支持需要依赖 C 编译的 `printpdf` 后端，已在 Cargo.toml 中配置
- 目前无法用纯 Rust 测试 PDF 文本提取（需要外部工具如 pdftotext 或 pypdf）
