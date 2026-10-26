# Hugonic

Hugo 静态站点的桌面管理器：左侧资源管理器 + Monaco 编辑器 + 实时预览，右上角一键构建与部署。
内置 Hugo 二进制，用户装完即可用，不需要自己安装 Hugo。

> **命名待定**：UI 原型中的产品名为 *Hugo Studio*，仓库目录为 `Hugonic`。本文档统一用 Hugonic 指代本项目。

## 当前状态

**设计阶段。** 仓库目前只有文档与 UI 原型，还没有应用代码：

| 内容 | 位置 |
|---|---|
| 协作约束 | [AGENTS.md](AGENTS.md) |
| 技术选型 / 架构 / UI / 开发指南 | [`docs/`](docs/) |
| 可在浏览器打开的静态原型 | [docs/prototype/prototype-hugo-desktop-v1.html](docs/prototype/prototype-hugo-desktop-v1.html) |

## 文档

| 文档 | 内容 |
|---|---|
| [技术选型](docs/tech-stack.md) | Tauri 2 / Vue 3 / Monaco / UnoCSS 的选择与否决理由 |
| [架构设计](docs/architecture.md) | 进程模型、预览链路、构建流水线、sidecar 打包、安全 |
| [UI 设计](docs/ui-design.md) | 布局、组件拆分、交互规则、设计 token |
| [开发指南](docs/development.md) | 环境准备、常用命令、测试策略、提交规范 |
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
| 分发 | `tauri build` + updater | 一条命令产出 dmg / msi / AppImage |

取舍细节见 [技术选型](docs/tech-stack.md)。

## 目标功能

- 打开本地 Hugo 站点，在资源管理器中浏览 `content` / `pages` / `config` / `layouts` / `static`
- 用 Monaco 编辑 Markdown、TOML、YAML、HTML，`Ctrl/Cmd+S` 保存
- 内嵌 iframe 实时预览，支持移动 / 桌面两种视图
- 一键构建，弹窗中展示进度、页面数与构建日志
- 部署到 Vercel / Netlify / GitHub Pages / Cloudflare Pages，或导出静态文件到本地
- 自动更新

## 里程碑

| 阶段 | 内容 | 完成标准 |
|---|---|---|
| M0 | 脚手架 | `pnpm tauri dev` 能启动窗口，前端 HMR 正常 |
| M1 | 打开项目 + 文件树 | 选择目录后正确渲染目录树，忽略 `.git` / `public` |
| M2 | 编辑器 + 保存 | 打开、编辑、保存文件，标签与脏标记正确 |
| M3 | 实时预览 | 保存后预览内容更新，端口冲突可自愈 |
| M4 | 构建 | 弹窗展示真实日志与耗时，可取消 |
| M5 | 部署 | 至少一个平台可用（GitHub Pages 或 Netlify） |
| M6 | 打包发布 | 三平台安装包 + 签名的自动更新 |

## 协作约定

见 [AGENTS.md](AGENTS.md)：每次改动完成后必须创建一个对应的 commit；每次改动必须附带测试，交付前所有测试与验证通过。
