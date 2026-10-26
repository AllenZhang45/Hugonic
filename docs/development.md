# 开发指南

项目处于设计阶段。本文描述**目标工程结构与协作方式**；脚手架落地后，命令与目录会同步更新。
各阶段的交付物与验收标准见 [开发计划](roadmap.md)。

## 环境要求

| 依赖 | 版本 | 说明 |
|---|---|---|
| Node.js | 20 LTS+ | 前端构建（自带 npm） |
| npm | 随 Node 安装 | 包管理，仓库提交 `package-lock.json` |
| Rust | stable | Tauri 核心 |
| Tauri CLI | 2.x | 作为项目依赖安装：`npm i -D @tauri-apps/cli@^2` |
| Hugo | 仅开发期 | 调试与生成 sidecar 二进制；最终由安装包内置 |

Linux（Debian / Ubuntu 系）系统依赖：

```bash
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

当前只面向 Linux（Pop!_OS 24.04 / Ubuntu 24.04 系）开发；macOS / Windows 的构建依赖暂时不需要。

## 常用命令（规划）

| 命令 | 作用 |
|---|---|
| `npm install` | 安装前端依赖 |
| `npm ci` | CI / 干净安装（严格按 lockfile） |
| `npm run tauri dev` | 启动桌面应用（前端 HMR + Rust 热重载） |
| `npm run dev` | 只启动 Vite 前端（无 Tauri API，需 mock） |
| `npm run tauri build` | 打包当前平台安装包 |
| `npm run lint` | ESLint + Prettier |
| `npm test` | Vitest 单测 |
| `cd src-tauri && cargo test` | Rust 单测 |
| `node scripts/check-docs.mjs` | 校验 Markdown 相对链接（文档改动必跑） |
| `scripts/fetch-hugo.sh` | 下载并校验本平台的 Hugo 二进制到 `src-tauri/binaries/` |

`tauri` 是 `package.json` 里的脚本，指向项目依赖的 `@tauri-apps/cli`；不需要 `cargo install tauri-cli`。

## 目录结构（规划）

```
.
├─ AGENTS.md               协作约束
├─ README.md
├─ docs/                   设计文档
├─ src/                    Vue 前端
│  ├─ main.ts
│  ├─ App.vue
│  ├─ components/
│  ├─ stores/
│  ├─ composables/
│  └─ styles/
├─ src-tauri/              Rust 核心
│  ├─ Cargo.toml
│  ├─ tauri.conf.json
│  ├─ capabilities/
│  ├─ binaries/            hugo-<target-triple>，不入库
│  └─ src/
├─ scripts/
└─ package.json
```

## 测试策略

依据 [AGENTS.md](../AGENTS.md)：**每次改动都必须编写或更新测试**，交付前所有测试与验证必须通过。

| 层 | 工具 | 覆盖内容 |
|---|---|---|
| 前端逻辑 | Vitest | Pinia store 状态迁移、路径工具、排序与过滤 |
| 前端组件 | Vitest + @vue/test-utils | 文件树折叠与选中、标签开关、脏标记提示 |
| Rust | `cargo test` | 路径校验（`..` 穿越）、构建输出解析、端口分配 |
| 集成冒烟 | tauri-driver（可选） | 打开示例站点 → 编辑保存 → 预览刷新 |

规则：

- 修 bug 先写能复现的失败用例，再改代码。
- 测试不依赖真实网络与用户目录；Hugo 相关逻辑用 fixture 目录或输出样本。
- 文档改动至少运行 `node scripts/check-docs.mjs`，保证链接不失效。
- 交付前执行：

  ```bash
  npm run lint && npm test && (cd src-tauri && cargo test)
  ```

  并在 commit 正文或 PR 中写明验证方式。

## 提交规范

- **一次改动一个 commit**：完成一个可独立回滚的变更后立即提交，不把无关修改混在一起。
- 消息遵循 Conventional Commits：`<type>(<scope>): <subject>`，type 取 `feat` / `fix` / `refactor` / `test` / `docs` / `chore` / `perf`。
- subject 用中文或英文并保持一致，不超过 50 字；正文说明“为什么”，而不是复述“改了什么”。
- 示例：
  - `feat(explorer): 按需加载目录树`
  - `fix(preview): 端口占用时自动重试`
  - `test(build): 覆盖 hugo 输出解析`
- 不提交生成物（`node_modules`、`target`、`dist`、`public`）与本地配置。
- 包管理器统一用 npm，提交 `package-lock.json`，CI 用 `npm ci`；不混用 pnpm / yarn。

## 代码风格

- TypeScript 严格模式；Vue 统一 `<script setup lang="ts">`，组件文件名 PascalCase。
- 状态只在 Pinia store 中变更，组件通过 action 调用。
- 样式只走 UnoCSS utility，颜色等设计 token 从 CSS 变量取；不写全局选择器。
- Rust：`cargo fmt` + `cargo clippy -- -D warnings`，命令层与业务层分离。
- 新增依赖需说明理由；能用标准库或已有依赖解决时不加新包。

## 排障

| 现象 | 处理 |
|---|---|
| `npm run tauri dev` 报 WebKitGTK 缺失 | 安装上面的 Linux 系统依赖 |
| 预览空白且 1313 被占用 | 检查端口分配逻辑，确认 sidecar 的实际端口 |
| 打包时找不到 `hugo-<triple>` | 先运行拉取脚本，核对 target triple 命名 |
| iframe 被 CSP 拦截 | 检查 `frame-src` 是否包含 `http://127.0.0.1:*` |
