# AI微光求职

> 免费简历优化 · 面向每一位劳动者

面向弱势就业群体的免费简历优化交互工具。帮助残障人士、大龄再就业人群等求职弱势群体，突破简历表达困境，让每一类劳动者的个人价值被完整看见。

**2026人文季 · 公益单元投稿** — #用AI重新看见人 #用AI回应真实困境

## 项目理念

市面上常规 AI 简历工具大多服务都市白领，门槛高、话术专业化，忽略弱势群体缺乏职场表达能力、存在求职自卑、信息获取闭塞等现实问题。

本项目聚焦四类求职弱势群体——残障人士、大龄再就业人群、乡村务工青年、偏远地区贫困应届生——依托 AI 技术搭建轻量化交互系统，以技术作为帮扶载体，消解就业不平等，践行 AI 向善的人文内核。

## 核心功能

### MVP 阶段（已实现）

- **分人群定制化简历优化** — 针对残障求职者和 45+ 大龄求职者，使用不同的 AI 优化策略
  - 残障人群：弱化身体局限，突出技能优势
  - 大龄人群：规避年龄歧视话术，凸显实操经验
- **PDF 简历上传解析** — 上传已有 PDF 简历，自动提取文本
- **全景式优化结果** — 优化后的简历 + 心理鼓励话术一次性生成
- **包容性岗位资讯** — 接入知乎搜索 API，匹配适配弱势群体的岗位/文章链接
- **PDF 导出** — 将优化后的简历导出为 PDF 文件
- **无障碍适配** — 大字体模式、高对比度模式、ARIA 标签、键盘导航
- **完全匿名** — 用户数据无需留存服务器，隐私安全

### 后续规划

- 更多人群覆盖：低学历务工者、贫困应届生
- 语音输入交互
- 方言语音支持
- 多轮心理疏导对话
- 数字档案板块

## 技术栈

| 层级 | 技术 |
|------|------|
| 前端 | Vue 3 + TypeScript + Vite + Element Plus |
| 后端 | Rust + Axum |
| 外部 API | 知乎数据开放平台（直答 Agent + 知乎搜索） |
| PDF 解析 | pdf-extract（Rust） |
| PDF 生成 | printpdf（Rust） |

## 项目结构

```
ai_resume_aid/
├── backend/                    # Rust Axum 后端服务
│   ├── Cargo.toml
│   ├── .env.example            # 环境变量配置模板
│   └── src/
│       ├── main.rs             # 服务入口
│       ├── lib.rs              # AppState
│       ├── config.rs           # 环境配置
│       ├── errors.rs           # 统一错误处理
│       ├── models.rs           # 请求/响应类型定义
│       ├── handlers/
│       │   ├── optimize.rs     # POST /api/optimize
│       │   ├── search_jobs.rs  # POST /api/search-jobs
│       │   └── export_pdf.rs   # POST /api/export-pdf
│       └── services/
│           ├── zhihu.rs        # 知乎直答 + 搜索 API
│           ├── pdf_parser.rs   # PDF 文本提取
│           └── pdf_generator.rs # PDF 文档生成
│
├── web/                        # Vue 3 前端 SPA
│   ├── index.html
│   ├── package.json
│   ├── vite.config.ts
│   ├── env.d.ts
│   ├── tsconfig.json
│   └── src/
│       ├── main.ts
│       ├── App.vue             # 单页应用（全部组件）
│       ├── api/client.ts       # Axios API 客户端
│       ├── types/index.ts      # TypeScript 类型定义
│       └── styles/main.css     # 设计系统 + 主题变量
│
└── docs/
    └── requirement.md          # 产品需求文档
```

## 快速开始

### 前置条件

- Rust 1.75+
- Node.js 18+
- npm 9+
- 知乎开放平台账号（[developer.zhihu.com](https://developer.zhihu.com)）及 Access Secret

### 1. 配置环境变量

```bash
cd ai_resume_aid/backend
cp .env.example .env
# 编辑 .env，填入你的知乎 Access Secret
```

`.env` 文件内容：

```
ZHIHU_API_TOKEN=你的知乎开放平台 API Token
SERVER_HOST=0.0.0.0
SERVER_PORT=3001
RUST_LOG=info
```

### 2. 启动后端

```bash
cd ai_resume_aid/backend
cargo run
```

后端服务启动在 `http://localhost:3001`，提供以下端点：

| 端点 | 方法 | 功能 |
|------|------|------|
| `/api/optimize` | POST | 简历优化（支持 JSON 文本 / multipart PDF 上传） |
| `/api/search-jobs` | POST | 搜索包容性岗位资讯 |
| `/api/export-pdf` | POST | 生成 PDF 下载 |

### 3. 启动前端

```bash
cd ai_resume_aid/web
npm install
npm run dev
```

前端开发服务器启动在 `http://localhost:5173`，已配置 Vite 代理将 `/api/*` 请求转发到后端。

### 4. 构建生产版本

```bash
cd ai_resume_aid/web
npm run build
# 构建产物在 web/dist/
```

## API 接口

### POST /api/optimize

接受 JSON 文本或 PDF 文件上传，返回优化后的简历和心理鼓励话术。

**JSON 请求：**
```json
{
  "text": "原始简历文本内容",
  "group": "disabled"
}
```

**multipart 请求：**
- `file`: PDF 文件
- `group`: `"disabled"` 或 `"elderly"`
- `text`: （可选）额外文本

**响应：**
```json
{
  "success": true,
  "data": {
    "optimized_text": "优化后的简历（Markdown）",
    "encouragement": "心理鼓励话术",
    "original_text": "原始简历文本"
  }
}
```

### POST /api/search-jobs

```json
{
  "group": "disabled",
  "query": "可选自定义搜索词"
}
```

### POST /api/export-pdf

```json
{
  "text": "要导出为 PDF 的文本内容",
  "title": "可选标题"
}
```

返回 `application/pdf` 二进制流。

## 无障碍设计

本项目的无障碍适配覆盖：

- **大字体模式** — CSS 变量驱动，一键切换，字号放大 25%
- **高对比度模式** — 满足 WCAG AA 标准，边框、文字对比度增强
- **ARIA 标签** — 所有交互元素附带语义化标签
- **键盘导航** — 完整的 Tab 顺序和焦点指示器
- **屏幕阅读器** — 实时区域通过 `aria-live` 宣告状态变化
- **响应式布局** — 375px 手机到 1200px+ 桌面自适应

## 知乎开放平台集成

本项目接入 [知乎数据开放平台](https://developer.zhihu.com) 的两个 API：

| API | 用途 | 免费配额 |
|-----|------|----------|
| 直答 Agent（`zhida-thinking-1p5`） | 简历优化 + 心理鼓励生成 | 1000 次/天 |
| 知乎搜索 | 包容性岗位资讯匹配 | 1000 次/天 |

认证方式：`Authorization: Bearer <Access Secret>` + `X-Request-Timestamp` 时间戳头。

## 人文价值

AI 可以快速润色文字、匹配岗位，但无法天然关照人的处境。本项目跳出"效率工具"的单一定位，把技术重心放在被就业市场忽视的人群身上：

许多残障、中老年、基层劳动者拥有扎实实操能力，却不懂包装自身经历，在求职阶段就被筛选淘汰。本工具用降低门槛的 AI 交互，弥补信息差与表达短板，让每一类劳动者的个人价值被完整看见。

> 算力只是载体，关怀人的需求、弥合社会公平缺口，才是 AI 创作真正的人文意义。

## License

公益项目，开放源代码。
