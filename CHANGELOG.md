# 更新日志

本项目遵循[语义化版本](https://semver.org/lang/zh-CN/)，格式参考 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)。

## [未发布]

### 新增

- 视图骨架：侧栏可切换仪表盘 / 分析 / 媒体库 / 主题，内容视图保留编辑器状态
- 仪表盘：站点信息、内容统计、最近修改、快捷操作与运行状态
- 分析：section 分布、字数、标签 / 分类 Top、元数据缺失检查

> 媒体库与主题浏览仍为占位，随 0.2.0 后续阶段接入。

## [0.1.1] - 2026-10-28

### 变更

- 应用内功能图标改用 Material Symbols（UnoCSS preset-icons，mask 模式），图标映射集中在 `src/components/icons.ts`
- 补充 Material Symbols 的 Apache-2.0 许可与第三方许可说明

### 开发者

- 新增 `scripts/dev.sh`：在 tty / SSH 中自动探测 Wayland / X11 后启动 `tauri dev`

## [0.1.0] - 2026-10-27

首个可用版本：面向 Linux 的 Hugo 桌面客户端。

### 新增

- 打开本地 Hugo 站点：校验 `hugo.toml` / `config.*`，文件树按需加载，忽略 `.git`、`public` 等目录
- Monaco 编辑器：Markdown / TOML / YAML 语法高亮、多标签、脏标记、`Ctrl/Cmd+S` 原子保存、外部修改冲突提示、Markdown 工具栏
- 实时预览：内置 `hugo server` sidecar，自动选择 1313+ 空闲端口，iframe 展示；保存后经 LiveReload 自动刷新，支持移动 / 桌面视图
- 构建：`hugo --minify`、流式日志、Pages / Static / Images 摘要、取消、打开输出目录
- 部署：Netlify 一键部署，token 存入系统钥匙串，按站点地址重新构建 baseURL，zip 上传并轮询结果
- 打包：单个 `.deb`，内置 Hugo 二进制，用户无需另装 Hugo
- 桌面集成：应用图标、单窗口布局、底部状态栏

### 说明

- 仅支持 Linux x86_64，产物为 `.deb`
- 不含自动更新、代码签名与 AppImage
- 部署目前仅支持 Netlify
- 已知依赖告警：`monaco-editor` 依赖的 `dompurify` 有 npm audit 告警，当前不渲染不可信 HTML，暂时接受

[0.1.1]: https://github.com/AllenZhang45/Hugonic/releases/tag/v0.1.1
[0.1.0]: https://github.com/AllenZhang45/Hugonic/releases/tag/v0.1.0
