# Hugonic

Hugo 静态站点的桌面管理器：左侧资源管理器 + Monaco 编辑器 + 实时预览，右上角一键构建与部署。
内置 Hugo 二进制，用户装完即可用，不需要自己安装 Hugo。

> **命名待定**：UI 原型中的产品名为 *Hugo Studio*，仓库目录为 `Hugonic`。本文档统一用 Hugonic 指代本项目。

## 当前状态

**M6 已实现。** 完整链路可用：编辑、实时预览、构建、Netlify 部署，并可用 `tauri build` 产出 Linux `.deb` 安装包（内置 Hugo）。安装验收进度见 [开发计划](docs/roadmap.md)。

| 内容 | 位置 |
|---|---|
| 技术选型 / 架构 / UI / 开发指南 | [`docs/`](docs/) |
| 前端（Vue 3 + UnoCSS + Pinia） | [`src/`](src/) |
| Tauri 壳与配置 | [`src-tauri/`](src-tauri/) |
| 可在浏览器打开的静态原型 | [docs/prototype/prototype-hugo-desktop-v1.html](docs/prototype/prototype-hugo-desktop-v1.html) |

## 快速开始

```bash
npm install
npm run tauri dev
```

或安装构建好的 deb：

```bash
sudo apt install ./Hugonic_*.deb
```

完整的环境依赖、命令与测试说明见 [开发指南](docs/development.md)。

## 文档

| 文档 | 内容 |
|---|---|
| [技术选型](docs/tech-stack.md) | Tauri 2 / Vue 3 / Monaco / UnoCSS 的选择与否决理由 |
| [架构设计](docs/architecture.md) | 进程模型、预览链路、构建流水线、sidecar 打包、安全 |
| [UI 设计](docs/ui-design.md) | 布局、组件拆分、交互规则、设计 token |
| [开发指南](docs/development.md) | 环境准备、常用命令、测试策略、提交规范 |
| [开发计划](docs/roadmap.md) | M0–M6 阶段拆解、验收标准、风险与待决事项 |
| [更新日志](CHANGELOG.md) | 各版本的变更记录 |
| [第三方许可](THIRD_PARTY_LICENSES.md) | Material Symbols 等第三方资源许可 |
| [UI 原型](docs/prototype/prototype-hugo-desktop-v1.html) | 单文件 HTML 原型，浏览器直接打开 |

## 技术栈

| 层 | 选择 | 理由 |
|---|---|---|
| 桌面壳 | Tauri 2.x | Rust 壳 + 系统 WebView，比 Electron 轻一个数量级；`externalBin` 可内置 Hugo |
| 前端 | Vue 3 + Composition API + Pinia | 工具型应用状态复杂度不高，模板语法贴合“编辑器 + 文件树 + 预览”布局 |
| 编辑器 | Monaco Editor | VS Code 同款，Markdown / TOML / YAML 高亮开箱即用 |
| 文件树 | 自研（约 200 行） | Tauri `fs` 按需读目录；第三方文件树库都比手写复杂 |
| 预览 | iframe + `hugo server` | 复用 Hugo 自带的热重建与 LiveReload，不造轮子 |
| 样式 | UnoCSS | Tailwind 语法的按需版本，构建快、无需设计系统 |
| 分发 | `tauri build` | 只做 Linux 本机打包（`.deb`）；跨平台与自动更新暂缓 |

取舍细节见 [技术选型](docs/tech-stack.md)。

## 目标功能

- 打开本地 Hugo 站点，在资源管理器中浏览 `content` / `pages` / `config` / `layouts` / `static`
- 用 Monaco 编辑 Markdown、TOML、YAML、HTML，`Ctrl/Cmd+S` 保存
- 内嵌 iframe 实时预览，支持移动 / 桌面两种视图
- 一键构建，弹窗中展示进度、页面数与构建日志
- 部署到 Netlify（GitHub Pages 等其余平台后续接入），或导出静态文件到本地
- 打包为 Linux 本机安装包（`.deb`）

## 开发计划

按 M0–M6 推进：脚手架 → 打开项目与文件树 → 编辑器与保存 → 实时预览 → 构建 → 部署 → 打包（Linux）。
每阶段的交付物、验收标准、依赖与风险见 [开发计划](docs/roadmap.md)。

## 协作约定

每次改动完成后创建一个对应的 commit；每次改动附带测试，交付前所有测试与验证通过。详见 [开发指南](docs/development.md)。
