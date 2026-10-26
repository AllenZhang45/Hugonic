use std::path::PathBuf;
use std::sync::Mutex;

/// 当前打开的项目根目录。文件相关命令只允许访问这个目录下的路径。
#[derive(Default)]
pub struct AppState {
    pub root: Mutex<Option<PathBuf>>,
}
