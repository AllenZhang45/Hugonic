# 技术选型

本文记录桌面端技术栈的决策与取舍。总目标：一个**零外部依赖**的 Hugo 桌面客户端 —— 用户安装后无需另装 Hugo，即可浏览、编辑、预览、构建并部署站点。

## 决策总览

| 层 | 选择 | 版本基线 | 一句话理由 |
|---|---|---|---|
| 桌面框架 | Tauri 2.x | 2.x | Rust 壳 + 系统 WebView，可把 Hugo 二进制作为 sidecar 打包 |
| 前端 | Vue 3 + Composition API | 3.4+ | 工具型应用，模板语法适合多面板布局 |
| 包管理 | npm | 随 Node 安装 | 零额外安装，单包仓库够用 |
| 状态管理 | Pinia | 2.x | 够用且无聊 |
| 编辑器 | Monaco Editor | 最新稳定版 | VS Code 同款，Markdown / TOML / YAML 开箱即用 |
| 文件树 | 自研组件 | 约 200 行 | 第三方文件树库都比手写复杂 |
| 预览 | iframe + `hugo server` | Hugo 0.139 | 热重建与刷新由 Hugo 自己完成 |
| 样式 | UnoCSS | 最新稳定版 | Tailwind 语法的按需版本 |
| 构建分发 | Tauri 内置 | — | Linux 本机打包，只产出 `.deb` |

## 桌面框架：Tauri 2.x

- **体积与内存**：链接系统 WebView（Linux WebKitGTK / macOS WKWebView / Windows WebView2），不打包 Chromium，相对 Electron 有数量级的优势。
- **内置 Hugo**：Tauri 2 的 `bundle.externalBin` 可以把 `hugo` 作为 sidecar 打进安装包，运行期由 Rust 侧启动。Hugo 本身是单个静态链接的 Go 二进制，天然适合这种分发方式 —— 这是“零外部依赖”的关键。
- **Rust 侧能力**：进程管理、文件监听、路径校验放在 Rust，比前端更可靠，也避免把危险能力暴露给 WebView。

**被否决：Electron。** 体积大、内存高，还要自己跟进 Chromium 的安全更新；对本地工具型应用没有收益。

## 前端：Vue 3 + Composition API + Pinia

- 界面本质是“图标栏 + 文件树 + 编辑器 + 预览 + 状态栏”的固定面板布局，状态集中在“当前项目 / 打开的文件 / 预览 / 构建”，Vue 的响应式与模板足够，不需要更重的架构。
- 相对 React：本项目没有复杂的状态组合或生态依赖；单文件组件把模板、样式、逻辑放在一起，对面板型 UI 更直观；中文生态与 Hugo 中文用户群也更匹配。
- Pinia 按领域拆 store（`project` / `files` / `editor` / `preview` / `build` / `deploy` / `ui`），避免一个巨型 store。

## 包管理器：npm

- 直接用 Node 自带的 npm，不额外引入 pnpm / yarn。这是单包仓库，pnpm 的核心收益（workspace 共享、严格依赖树）用不上，却要多维护一层符号链接与 hoisting 兼容问题。
- 提交 `package-lock.json`，CI 用 `npm ci` 保证可复现安装。
- Tauri CLI 作为 devDependency 安装（`npm i -D @tauri-apps/cli`），用 `npm run tauri` 调用，不全局安装。
- **曾考虑 pnpm**：磁盘只剩 1.2 GB 时，硬链接存储的省空间能力是实打实的优势。空间问题解决后，少一个工具、少一类坑更重要。

## 编辑器：Monaco Editor

- 与 VS Code 同一套编辑器内核：Markdown / TOML / YAML / HTML 高亮、行号、折叠、minimap、查找替换、多光标全部自带，不需要自己拼装。
- 体积大是已知代价。桌面应用从本地加载，不占网络与首屏预算。
- **被否决：CodeMirror 6。** 更轻、模块化更好，但语言支持、diff、worker 集成等周边不如 Monaco 成熟。编辑器不是本项目的差异化点，不值得为体积付出集成成本。

## 文件树：自研，控制在 200 行内

- 数据来源是 Tauri 的 `fs` 命令：按需读取一层目录，节点懒加载。
- 只做必要的事：目录优先排序、展开 / 折叠、点击打开、忽略 `.git`、`public`、`resources`、`node_modules`。
- **被否决：任何第三方文件树组件。** 它们都要适配虚拟滚动、拖拽、多选等本项目不需要的能力，接入成本高于实现成本。

## 实时预览：iframe + `hugo server`

- Rust 侧启动 `hugo server` 子进程，前端用 iframe 指向 `http://127.0.0.1:<port>`。
- 保存文件后由 **Hugo 自身**的文件监听触发重建，Hugo 注入的 LiveReload 脚本负责刷新 iframe。应用不需要实现“重建 + 刷新”的编排。
- 约束：iframe 与 Tauri 应用不同源，父页面读不到 `contentDocument`、也不能向预览页注入脚本，刷新只能交给 Hugo 的 LiveReload；同时 CSP 要放行 `frame-src http://127.0.0.1:*`。
- 备选：自己渲染 Markdown 做预览。**被否决** —— 主题、shortcode、模板都无法还原，预览会与真实站点不一致。

## 样式：UnoCSS

- Tailwind 语法的按需生成器，构建快、产物小；桌面应用不需要设计系统，utility class 足够。
- 颜色、间距、圆角只保留一份设计 token（CSS 变量），其余全部走 utility。
- **被否决**：完整 Tailwind（需要 PostCSS 配置与内容扫描）、手写全局 CSS（面板多、重复样式多）。

## 构建与分发：Tauri 内置

- 只面向本机开发环境（Linux x86_64）：`bundle.targets` 固定为 `["deb"]`，`tauri build` 只产出 `.deb`，不生成 AppImage。
- 不做 macOS / Windows 打包，也不做代码签名与公证 —— 这些只在需要分发给他人时才成为问题。
- 自动更新（`tauri-plugin-updater`）暂不启用：它需要签名密钥、更新端点与发布流水线，本机自用没有收益。方案记在 [开发计划](roadmap.md) 的 v1.1。

## 待决问题

| 问题 | 说明 |
|---|---|
| Hugo 版本策略 | 固定一个版本随应用发布，还是允许用户切换自定义 Hugo？前者稳定，后者灵活 |
| 部署实现 | 调用各平台 CLI，还是走 API？CLI 依赖用户环境，API 需要凭据管理 |
| 跨平台打包 | 当前只做 Linux；macOS / Windows 需要签名与公证，留到有分发需求时 |
| 主题市场 | 是否内置主题浏览与安装，会显著扩大范围 |
