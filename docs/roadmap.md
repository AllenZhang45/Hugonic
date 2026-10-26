# 开发计划

目标：以 M0–M6 七个阶段推进到 v1.0 —— 一个能在本机（Linux）安装运行的 Hugo 桌面客户端。
每个阶段都以**可运行的产物**结束，且必须满足[质量门禁](#质量门禁)。

不写日历日期：阶段按依赖顺序排队，实际节奏由验证结果决定。

## 范围

**v1.0 必须包含**：打开 Hugo 站点、文件树、编辑器与保存、实时预览、构建、至少一个部署目标、Linux 本机安装包（`.deb`）。

**v1.0 不做**（原型里出现但先隐藏或置灰）：

- 仪表盘、分析、媒体库、主题市场
- 命令面板、多窗口、拖拽排序
- 浅色主题
- 内置主题安装
- macOS / Windows 打包与签名、自动更新（需要分发给他人时再做）

## 阶段总览

| 阶段 | 主题 | 依赖 | 出口标准 |
|---|---|---|---|
| M0 | 脚手架与技术验证 | — | 窗口能起、HMR 正常、CI 跑通 |
| M1 | 打开项目 + 文件树 | M0 | 真实站点目录正确渲染 |
| M2 | 编辑器 + 保存 | M1 | 编辑保存闭环、脏标记正确 |
| M3 | 实时预览 | M2 | 保存后预览自动更新、无孤儿进程 |
| M4 | 构建 | M3 | 真实构建、日志流式展示、可取消 |
| M5 | 部署 | M4 | 一个平台端到端可用 |
| M6 | 打包（Linux） | M5 | `.deb` 可安装运行 |

## M0 脚手架与技术验证

交付物：可运行的空应用骨架 + CI。

任务：

- [x] Vite + Vue 3 + TypeScript + UnoCSS + Pinia 初始化，`src/styles/tokens.css` 落地原型 token
- [x] Tauri 2 初始化，窗口默认 1280×800、最小 1024×640，`capabilities` 只开必要权限
- [x] ESLint + Prettier + Vitest + @vue/test-utils
- [x] 六个空壳组件：`AppTitleBar` / `ActivityBar` / `ExplorerPanel` / `MonacoEditor` / `PreviewPane` / `StatusBar`
- [ ] CI：`npm run lint && npm test && npm run build`（Linux）

技术验证 spike：

- [ ] Monaco 在 Tauri WebView 中能加载、worker 正常、字体不塌（体积与 worker 配置一次定清楚）
- [ ] iframe 加载 `http://127.0.0.1:任意端口` 不被 CSP 拦截，WebKitGTK 下表现一致

验收：

- `npm run tauri dev` 起窗口，改前端代码 HMR 生效
- 未打开项目时显示空状态，状态栏显示“未打开项目”
- `npm test` 至少有一个真实断言（不是占位）

## M1 打开项目 + 文件树

交付物：能打开真实 Hugo 站点并在左侧看到目录树。

任务：

- [x] `tauri-plugin-dialog` 选择目录；`tauri-plugin-store` 记录最近项目
- [x] Rust `open_project`：校验 `hugo.toml` / `config.toml` / `config.yaml`，返回规范化根路径与 Hugo 版本
- [x] Rust `read_dir`：只读一层，返回 `{ name, kind, path }`，按忽略清单过滤
- [x] 路径校验：拒绝 `..` 穿越与符号链接逃逸
- [x] `FileTree` 组件 + `files` store：懒加载、目录优先排序、折叠、选中（打开文件随 M2）
- [x] 空状态与“不是 Hugo 站点”的错误提示

验收：

- 打开示例站点，树结构与磁盘一致；展开子目录才发请求
- `.git`、`public`、`resources`、`node_modules` 不出现在树里
- Rust 单测覆盖：忽略规则、路径穿越、排序
- 组件测试覆盖：折叠状态、选中态、懒加载触发

## M2 编辑器 + 保存

交付物：能打开、编辑、保存文件。

任务：

- [x] Monaco 集成，按扩展名映射语言；`Ctrl/Cmd+S` 保存
- [x] Rust `read_file` / `save_file`（临时文件 + rename 原子写），超过 2 MB 或非 UTF-8 拒绝打开
- [x] `editor` store：多标签、激活标签、脏状态、关闭确认
- [x] 检测磁盘上的外部修改，提示重新加载
- [x] Markdown 工具栏按钮接入真实命令（粗体、斜体、代码块、链接、图片、表格）

验收：

- 编辑 → 保存 → 重新打开，内容一致，无编码问题
- 未保存关闭标签会确认；放弃修改后磁盘内容不变
- 保存失败（只读文件）有明确错误，不丢编辑内容

## M3 实时预览

交付物：左侧编辑、右侧预览，保存即刷新。

任务：

- [ ] `scripts/fetch-hugo.sh`：下载 Linux x86_64 版 Hugo、校验 SHA256、写入 `src-tauri/binaries/hugo-x86_64-unknown-linux-gnu`
- [ ] Sidecar 生命周期：启动、就绪探测（匹配 `Web Server is available`）、停止、异常退出处理
- [ ] 端口分配：1313 起递增，占用自动跳过
- [ ] iframe 预览 + CSP `frame-src http://127.0.0.1:*`；设备切换只改容器宽度
- [ ] 应用退出 / 切换项目时结束子进程（含子进程组，避免孤儿）

验收：

- 保存文件后 1–2 秒内预览更新（由 Hugo LiveReload 驱动，前端不参与刷新）
- 1313 被占用时自动换端口且预览正常
- 退出应用后 `pgrep hugo` 无残留

## M4 构建

交付物：真实构建与可观测的日志。

任务：

- [ ] Rust `start_build` / `cancel_build`：执行 sidecar `hugo --minify`，工作目录为项目根
- [ ] 逐行 emit 日志事件，结束时 emit 结果（退出码、耗时、摘要）
- [ ] `BuildModal`：进度（确定或不确定）、日志滚动、复制日志、打开输出目录
- [ ] 摘要解析（Pages / Static files / Processed images / 耗时），解析失败时降级为原始日志
- [ ] 并发保护：同一项目同时只允许一个构建

验收：

- 示例站点构建成功，日志不丢行、顺序正确
- 取消后子进程确实结束，UI 回到可重试状态
- 解析器单测使用真实输出样本（含失败输出）

## M5 部署

交付物：至少一个平台的端到端部署。

任务：

- [ ] 先定平台：GitHub Pages（复用 Git 身份）或 Netlify（API token）
- [ ] 凭据存系统钥匙串，不落盘明文；提供“测试连接”
- [ ] 部署流程可观测：进度、日志、失败原因（权限 / 分支保护 / 构建失败）
- [ ] 部署成功后给出可访问 URL，可复制、可在浏览器打开

验收：

- 用示例站点完成一次真实部署并在公网访问
- 错误凭据、网络失败、目标冲突三类错误都有可读提示
- 部署逻辑的关键分支有单测（凭据读取、结果解析），网络调用用 mock

## M6 打包（Linux 本机）

交付物：可在本机安装运行的 `.deb`，内置 Hugo。

任务：

- [ ] Linux sidecar 二进制（`hugo-x86_64-unknown-linux-gnu`）与 `externalBin` 配置；打包脚本自动拉取并校验
- [ ] 应用图标、bundle identifier、版本号策略
- [ ] `bundle.targets` 固定为 `deb`，`tauri build` 只产出 `.deb`；确认包内依赖声明正确（WebKitGTK 4.1 等）
- [ ] 本机安装一次，走通打开站点 / 预览 / 构建
- [ ] CI 单 Linux job 构建产物并归档

验收：

- `.deb` 在 Pop!_OS 24.04 上 `sudo apt install ./hugonic_*.deb` 安装并启动，功能与 dev 模式一致
- 安装包体积在预期范围（含 Hugo 约 20–40 MB）
- 明确不含：AppImage、代码签名、公证、自动更新（见 v1.1）

## 阶段依赖

```
M0 ──▶ M1 ──▶ M2 ──▶ M3 ──▶ M4 ──▶ M5 ──▶ M6
        │              │
        └─ 文档同步 ────┘        （每阶段结束时更新 docs/）
```

M5 与 M6 可以在 M4 之后并行；只做本机打包，不涉及签名与公证流程。

## 质量门禁

每个阶段结束前，以下全部满足才算完成：

- 代码与测试在同一个 commit 里；修 bug 先有失败用例
- `npm run lint`、`npm test`、`cd src-tauri && cargo test` 全绿
- 文档改动跑 `node scripts/check-docs.mjs`
- 该阶段的手动冒烟清单逐条通过，结果写进 commit 正文
- 相关文档（架构 / UI / 开发指南）同步更新

## 风险与缓解

| 风险 | 影响 | 缓解 |
|---|---|---|
| WebKitGTK 的 iframe / CSP 行为与 Chromium 不同 | M0、M3 返工 | M0 先做 spike，在最难的环境（Linux WebKitGTK）上验证 |
| Monaco 体积与 worker 配置复杂 | M0 延期 | 在脚手架上就固定 worker 方案，避免后期换打包方式 |
| Hugo 输出格式随版本变化，摘要解析失效 | M4 展示错误 | 解析失败降级为原始日志；固定 Hugo 版本并加样本测试 |
| 子进程残留（崩溃、强制退出） | 用户体验差 | 启动时清理上次的 PID、进程组 kill、退出钩子 |
| deb 的依赖声明不全，换台机器装上后缺 WebKitGTK 运行库 | M6 返工 | 用 `dpkg -I` 检查 Depends；在干净环境（容器或新用户）安装验证 |
| 部署平台 API 变更 | M5 返工 | 先做一个平台，把接口抽象成 `DeployTarget`，其余平台后续接 |

## 待决（阻塞后续阶段）

| 决策 | 需要的时间点 | 影响 |
|---|---|---|
| Hugo 版本固定策略 | M3 开工前 | sidecar 打包、输出解析、升级路径 |
| 首发部署平台 | M5 开工前 | 凭据模型、部署代码结构 |
| 是否需要跨平台分发 | 有分发需求时 | macOS / Windows 打包、签名与自动更新的范围 |

## v1.1 及以后

- 仪表盘 / 分析 / 媒体库 / 主题浏览（原型中的图标入口）
- 命令面板、多窗口、快捷键自定义
- 浅色主题与系统主题跟随
- 更多部署目标（Vercel、Cloudflare Pages）
- 跨平台打包（macOS 公证、Windows 签名）与自动更新（updater + 更新端点）
- 其他 Linux 包格式（AppImage、RPM）
- 站点新建向导（`hugo new site`）
