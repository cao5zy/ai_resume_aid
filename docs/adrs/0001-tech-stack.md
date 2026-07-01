# ADR 0001: 技术栈选择

## 状态

已采纳（2026-07-01）

## 上下文

需要为公益简历优化工具选择技术栈。项目主要面向残障和大龄求职者，需要具备良好的无障碍适配能力、部署简便、开发和维护成本低。

## 决策

- **后端**: Rust + Axum
- **前端**: Vue 3 + TypeScript + Vite + Element Plus
- **数据库**: 无（见 ADR 0002）

## 理由

### Axum（vs Actix-web，vs 其他后端框架）
- 团队已有 Rust 后端经验
- Axum 基于 Tokio + Tower 生态，类型安全的路由和中间件链
- 相比 Actix-web，Axum 的 IntoResponse trait 设计更简洁，错误处理更直观
- 编译期保证请求/响应类型匹配

### Vue 3 + Element Plus（vs React，vs 其他 UI 框架）
- 团队已有 Vue 3 经验
- Element Plus 提供了完整的无障碍组件（ARIA 标签、键盘导航）
- 开箱即用的表单、上传、消息提示组件，减少开发量
- TypeScript 支持完善

### Vite（vs Webpack）
- 开发服务器秒启，HMR 极快
- 配置简洁，原生 ESM 支持
- 本项目的单页 SPA 场景不需要复杂打包配置

## 后果

- 前端开发人员需要熟悉 Vue 3 Composition API
- 无 SSR，SEO 不在本工具诉求范围内（登录即用的交互工具）
- 后端用 Rust 意味着编译时间较长，但运行时性能和安全性优势显著
