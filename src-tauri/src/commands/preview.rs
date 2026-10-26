use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::state::{AppState, PreviewState};

const SIDECAR_NAME: &str = "hugonic-hugo";
const PORT_RANGE_START: u16 = 1313;
const PORT_ATTEMPTS: u16 = 20;
const READY_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewInfo {
    pub url: String,
    pub port: u16,
}

#[tauri::command]
pub async fn start_preview(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<PreviewInfo, String> {
    let root = state
        .root
        .lock()
        .map_err(|_| "项目状态不可用".to_string())?
        .clone()
        .ok_or_else(|| "尚未打开项目".to_string())?;

    // 启动和就绪探测最长会阻塞十几秒，放到阻塞线程池里，避免卡住 UI 线程。
    tauri::async_runtime::spawn_blocking(move || start_preview_blocking(app, root))
        .await
        .map_err(|error| format!("预览启动任务失败：{error}"))?
}

fn start_preview_blocking(app: AppHandle, root: PathBuf) -> Result<PreviewInfo, String> {
    stop_preview_impl(&app);

    let port = pick_port(PORT_RANGE_START, PORT_ATTEMPTS)?;
    let binary = sidecar_path()?;

    let mut command = Command::new(&binary);
    command
        .current_dir(&root)
        .args(["server", "--bind", "127.0.0.1", "--port", &port.to_string()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }

    let mut child = command
        .spawn()
        .map_err(|error| format!("启动 hugo server 失败：{error}"))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "无法读取 hugo 输出".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "无法读取 hugo 错误输出".to_string())?;

    let (sender, receiver) = mpsc::channel::<String>();
    spawn_reader(stdout, sender.clone());
    spawn_reader(stderr, sender);

    let deadline = Instant::now() + READY_TIMEOUT;
    let mut url: Option<String> = None;
    let mut tail: Vec<String> = Vec::new();

    while Instant::now() < deadline {
        match receiver.recv_timeout(Duration::from_millis(200)) {
            Ok(line) => {
                if is_ready_line(&line) {
                    // Hugo 就绪行里可能写的是 localhost，统一用绑定的 IPv4 地址，
                    // 避免 iframe 解析到 ::1 而后端只监听了 127.0.0.1。
                    url = Some(format!("http://127.0.0.1:{port}"));
                    break;
                }
                push_tail(&mut tail, line);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if let Ok(Some(status)) = child.try_wait() {
                    return Err(format!(
                        "hugo server 提前退出（{status}）：{}",
                        tail.join(" | ")
                    ));
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    let Some(url) = url else {
        let _ = child.kill();
        let _ = child.wait();
        return Err(format!("等待 hugo server 就绪超时：{}", tail.join(" | ")));
    };

    if let Some(preview) = app.try_state::<PreviewState>()
        && let Ok(mut guard) = preview.child.lock()
    {
        *guard = Some(child);
    }

    // 必须把接收端留在后台持续读走输出：接收端被丢弃后读线程会退出，
    // 管道关闭后 hugo 下一次写日志（例如保存触发的重建）就会收到 EPIPE 而退出。
    std::thread::spawn(move || while receiver.recv().is_ok() {});

    Ok(PreviewInfo { url, port })
}

/// 预览进程是否还活着；已退出时顺手回收子进程，避免留下僵尸。
#[tauri::command]
pub fn preview_alive(app: AppHandle) -> bool {
    let Some(state) = app.try_state::<PreviewState>() else {
        return false;
    };
    let Ok(mut guard) = state.child.lock() else {
        return false;
    };
    let Some(child) = guard.as_mut() else {
        return false;
    };

    match child.try_wait() {
        Ok(Some(_status)) => {
            *guard = None;
            false
        }
        Ok(None) => true,
        Err(_) => false,
    }
}

#[tauri::command]
pub fn stop_preview(app: AppHandle) {
    stop_preview_impl(&app);
}

pub fn stop_preview_impl(app: &AppHandle) {
    if let Some(preview) = app.try_state::<PreviewState>()
        && let Ok(mut guard) = preview.child.lock()
        && let Some(mut child) = guard.take()
    {
        let _ = child.kill();
        let _ = child.wait();
    }
}

fn spawn_reader<R: std::io::Read + Send + 'static>(reader: R, sender: mpsc::Sender<String>) {
    std::thread::spawn(move || {
        for line in BufReader::new(reader).lines().map_while(Result::ok) {
            if sender.send(line).is_err() {
                break;
            }
        }
    });
}

fn push_tail(tail: &mut Vec<String>, line: String) {
    if line.trim().is_empty() {
        return;
    }
    tail.push(line);
    if tail.len() > 12 {
        tail.remove(0);
    }
}

/// 从 1313 开始找一个可用端口。
pub fn pick_port(start: u16, attempts: u16) -> Result<u16, String> {
    for offset in 0..attempts {
        let port = start.saturating_add(offset);
        if TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return Ok(port);
        }
    }

    Err(format!(
        "端口 {start}-{} 都被占用",
        start.saturating_add(attempts.saturating_sub(1))
    ))
}

/// 判断是否是 hugo 的启动完成行：
/// `Web Server is available at http://localhost:1313/ (bind address 127.0.0.1)`。
pub fn is_ready_line(line: &str) -> bool {
    let Some(rest) = line.split("Web Server is available at ").nth(1) else {
        return false;
    };
    let Some(url) = rest.split_whitespace().next() else {
        return false;
    };

    url.starts_with("http://127.0.0.1:")
        || url.starts_with("http://localhost:")
        || url.starts_with("http://[::1]:")
}

fn sidecar_path() -> Result<PathBuf, String> {
    let file_name = if cfg!(windows) {
        format!("{SIDECAR_NAME}.exe")
    } else {
        SIDECAR_NAME.to_string()
    };

    if let Ok(executable) = std::env::current_exe()
        && let Some(dir) = executable.parent()
    {
        let candidate = dir.join(&file_name);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    // 开发期 tauri dev 可能没有把 sidecar 复制到 target 目录，退回 binaries/。
    let triple = "x86_64-unknown-linux-gnu";
    let dev = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("binaries")
        .join(format!("{SIDECAR_NAME}-{triple}"));

    if dev.is_file() {
        return Ok(dev);
    }

    Err("找不到内置的 Hugo 可执行文件，请先运行 scripts/fetch-hugo.sh".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_hugo_ready_lines() {
        assert!(is_ready_line(
            "Web Server is available at http://127.0.0.1:1313/ (bind address 127.0.0.1)"
        ));
        assert!(is_ready_line(
            "Web Server is available at http://localhost:1313/ (bind address 127.0.0.1)"
        ));
    }

    #[test]
    fn ignores_unrelated_or_unsafe_lines() {
        assert!(!is_ready_line("Building sites …"));
        assert!(!is_ready_line(
            "Web Server is available at https://example.com/"
        ));
        assert!(!is_ready_line(
            "Web Server is available at http://10.0.0.1:1313/"
        ));
    }

    #[test]
    fn skips_occupied_port() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind");
        let occupied = listener.local_addr().expect("addr").port();

        let picked = pick_port(occupied, 5).expect("pick port");

        assert_ne!(picked, occupied);
    }
}
