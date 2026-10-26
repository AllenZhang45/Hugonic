use std::path::PathBuf;
use std::process::Child;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;

/// 当前打开的项目根目录。文件相关命令只允许访问这个目录下的路径。
#[derive(Default)]
pub struct AppState {
    pub root: Mutex<Option<PathBuf>>,
}

/// 正在运行的 `hugo server` 子进程。
#[derive(Default)]
pub struct PreviewState {
    pub child: Mutex<Option<Child>>,
}

/// 正在运行的 `hugo --minify` 构建进程。
#[derive(Default)]
pub struct BuildState {
    pub running: AtomicBool,
    pub cancelled: AtomicBool,
    pub pid: Mutex<Option<u32>>,
}
