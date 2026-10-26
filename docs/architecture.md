# 架构设计

## 进程模型

```
┌────────────────────────────── Tauri 应用 ──────────────────────────────┐
│ Rust 核心（src-tauri）                                                 │
│  ├─ 项目服务      打开 / 校验 Hugo 站点、最近项目                       │
│  ├─ Sidecar 管理  启动 / 停止 hugo server、端口分配、退出清理            │
│  ├─ 构建服务      执行 hugo --minify、流式转发日志                       │
│  └─ 文件服务      目录树读取、文件读写、notify 监听                      │
│           ▲ commands（invoke） / events（emit）                        │
│ WebView 前端（Vue 3 + Pinia）                                         │
│  ├─ 资源管理器 / Monaco 编辑器 / iframe 预览                            │
│  └─ 状态栏 / 构建弹窗 / 部署菜单 / Toast                               │
└───────────────────────────────────────────────────────────────────────┘
              │ spawn（sidecar）                │ HTTP
        hugo server ───────────────▶ http://127.0.0.1:1313 ──▶ iframe
```

**边界原则**：所有触碰文件系统、进程、网络的逻辑都在 Rust；前端只负责展示与输入。前端不拼装绝对路径，路径一律由 Rust 返回并校验。

## 规划中的目录结构

```
.
├─ AGENTS.md               协作约束
├─ README.md
├─ docs/                   设计文档
├─ src/                    Vue 前端
│  ├─ main.ts
│  ├─ App.vue
│  ├─ components/          ExplorerPanel、MonacoEditor、PreviewPane…
│  ├─ stores/              Pinia
│  ├─ composables/         invoke / listen 封装
│  └─ styles/              tokens.css、uno.config.ts 入口
├─ src-tauri/              Rust 核心
│  ├─ Cargo.toml
│  ├─ tauri.conf.json
│  ├─ capabilities/        权限声明
│  ├─ binaries/            hugo-<target-triple>
│  └─ src/
│     ├─ main.rs
│     ├─ commands/         open_project、save_file、start_build…
│     ├─ hugo/             sidecar 生命周期、输出解析
│     └─ fs/               目录遍历、路径校验、watch
├─ scripts/                拉取并校验 hugo 二进制
└─ package.json
```

## 启动流程

1. 用户选择项目目录（或从“最近项目”打开）。
2. Rust 校验：目录下存在 `hugo.toml`、`config.toml`、`config.yaml` 之一，否则提示“不是 Hugo 站点”。
3. 分配端口：优先 1313，被占用则递增（1313–1323），把实际端口返回前端。
4. 启动 sidecar：`hugo server --bind 127.0.0.1 --port <port>`，限定工作目录为项目根。
5. 从子进程输出读到 `Web Server is available at http://127.0.0.1:<port>/` 后，通知前端加载 iframe。
6. 加载文件树，恢复上次打开的标签。

**完成判据**：iframe 显示站点首页，状态栏显示 `hugo server 运行中` 和实际端口。

## 预览链路

保存一次文件会发生什么：

1. 编辑器 `Ctrl/Cmd+S` → `invoke("save_file", { path, content })`。
2. Rust 原子写文件（写临时文件后 rename），返回成功。
3. 前端清除脏标记；若新建了文件，刷新对应目录节点。
4. Hugo 自身的文件监听感知变更 → 重建 → 通过注入的 LiveReload 脚本通知浏览器刷新。
5. iframe 自动刷新，前端不参与。

关键点：

- **不要为了刷新 iframe 再监听一遍文件**：重复刷新会造成闪烁与竞态。Rust 的 notify 监听只服务于 UI 状态（外部修改提示、文件树同步）。
- iframe 与宿主不同源，不能读 `contentDocument`、不能注入脚本。刷新只能依赖 Hugo 的 LiveReload，这也是选型时把预览交给 Hugo 的原因。
- CSP 需要放行 `frame-src http://127.0.0.1:*`。
- 设备切换只改 iframe 容器宽度（如 390px / 100%），不重新加载页面。

## 文件树

- 每次只读一层目录：`read_dir(path)` → 前端存 `Map<path, Node[]>`，展开时才请求子目录。
- 排序：目录在前，按名称不区分大小写排序。
- 忽略：`.git`、`public`、`resources`、`node_modules`、`.hugo_build.lock`。
- 与磁盘同步：Rust 监听到项目目录变化后 emit `fs://changed`，前端只刷新受影响的已展开节点。

## 编辑器

- Monaco 通过 Vite worker 打包；语言按扩展名映射：`.md` → markdown、`.toml` → toml、`.yaml/.yml` → yaml、`.html` → html。
- 文件内容由 Rust 读取后写入 model；保存走 `save_file`，不依赖浏览器的下载能力。
- 脏状态：内容变更后与已保存版本比较；关闭有未保存修改的标签时弹确认。
- 快捷键：`Ctrl/Cmd+S` 保存、`Ctrl/Cmd+K` 文件搜索、`Ctrl/Cmd+Shift+P` 命令面板、`Ctrl/Cmd+B` / `I` 粗体 / 斜体。

## 状态管理（Pinia）

| Store | 职责 | 关键状态 |
|---|---|---|
| `project` | 当前项目 | `rootPath`、`hugoVersion`、`recentProjects` |
| `files` | 文件树 | `tree`、`expanded`、`selected` |
| `editor` | 标签与编辑器 | `openTabs`、`activeTab`、`dirty` |
| `preview` | 预览 | `serverUrl`、`status`、`device` |
| `build` | 构建 | `running`、`progress`、`log[]`、`lastResult` |
| `deploy` | 部署 | `target`、`running` |
| `ui` | 界面 | `activeView`、`toasts` |

## 构建流水线

1. 前端 `invoke("start_build")`。
2. Rust 以项目根为工作目录执行 sidecar：`hugo --minify --logLevel info`。
3. stdout / stderr 逐行 `emit("build://log", line)`；结束时 `emit("build://done", { code, durationMs, summary })`。
4. 前端在弹窗中渲染日志与进度。进度按阶段估算；**不要伪造随机百分比**（原型里的随机进度只是示意），拿不到确定进度就改用不确定进度条。
5. 结果摘要从输出解析：Pages / Static files / Processed images / 耗时；解析失败时原样展示日志。
6. 支持取消：kill 子进程并标记为已取消。

## Sidecar 打包

- `tauri.conf.json` 中 `bundle.externalBin: ["binaries/hugo"]`，二进制按 `<name>-<target-triple>` 命名。当前只打包 Linux：`hugo-x86_64-unknown-linux-gnu`。
- 由 `scripts/` 下的脚本在打包前下载 Linux x86_64 版 Hugo、校验 SHA256、放到 `src-tauri/binaries/`；CI 缓存该目录。
- 运行期用 `tauri-plugin-shell` 的 sidecar API 启动，避免误用用户 `PATH` 里的同名程序。
- 体积：单个 Hugo 二进制约 20–30 MB，可接受。

## 安全

- CSP 只允许本地资源与 `http://127.0.0.1:*` 的 frame；禁止远程脚本；`eval` 按 Monaco worker 需求单独放开。
- fs 权限收窄到当前打开的项目根目录；路径规范化后校验前缀，拒绝 `..` 穿越与符号链接逃逸。
- sidecar 路径由应用从资源目录解析，不接受用户可控路径。
- 若后续启用自动更新，必须验签（`tauri-plugin-updater` 的 `pubkey`），更新包经 HTTPS 获取。
- 部署凭据存系统钥匙串（keyring），不落盘明文。

## 错误与日志

- Rust 用 `tracing` 写滚动日志（`app_log_dir`），关键操作记录项目路径与耗时。
- 面向用户的错误统一走 Toast；构建 / 部署失败展示完整日志并提供“复制日志”。
- 子进程退出码非 0、端口冲突、目录无权限这三类错误需要明确文案与恢复建议。
