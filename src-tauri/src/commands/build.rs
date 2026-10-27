use std::io::{BufRead, BufReader};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::state::{AppState, BuildState};

const EVENT_LOG: &str = "build://log";
const EVENT_DONE: &str = "build://done";
const MAX_CAPTURE_LINES: usize = 5_000;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BuildLogPayload {
    line: String,
    stream: &'static str,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BuildDonePayload {
    code: Option<i32>,
    duration_ms: u64,
    cancelled: bool,
    summary: BuildSummary,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildSummary {
    pub pages: Option<u64>,
    pub static_files: Option<u64>,
    pub processed_images: Option<u64>,
    pub duration_ms: Option<u64>,
}

#[tauri::command]
pub fn start_build(
    app: AppHandle,
    state: State<AppState>,
    build: State<BuildState>,
) -> Result<(), String> {
    let root = state
        .root
        .lock()
        .map_err(|_| "项目状态不可用".to_string())?
        .clone()
        .ok_or_else(|| "尚未打开项目".to_string())?;

    if build.running.swap(true, Ordering::SeqCst) {
        return Err("已有构建正在进行".to_string());
    }

    build.cancelled.store(false, Ordering::SeqCst);

    let binary = crate::commands::preview::sidecar_path()?;
    let mut command = std::process::Command::new(binary);
    command
        .current_dir(&root)
        .arg("--minify")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    crate::process::isolate(&mut command);

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            build.running.store(false, Ordering::SeqCst);
            return Err(format!("启动 hugo 构建失败：{error}"));
        }
    };

    if let Ok(mut guard) = build.pid.lock() {
        *guard = Some(child.id());
    }

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "无法读取构建输出".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "无法读取构建错误输出".to_string())?;

    let started = Instant::now();
    let captured = Arc::new(Mutex::new(Vec::<String>::new()));
    spawn_stream(app.clone(), stdout, "stdout", captured.clone());
    spawn_stream(app.clone(), stderr, "stderr", captured.clone());

    std::thread::spawn(move || {
        let status = child.wait();
        let duration_ms = started.elapsed().as_millis() as u64;
        let cancelled = app
            .try_state::<BuildState>()
            .map(|state| state.cancelled.load(Ordering::SeqCst))
            .unwrap_or(false);

        let text = captured
            .lock()
            .map(|lines| lines.join("\n"))
            .unwrap_or_default();

        let payload = BuildDonePayload {
            code: status.ok().and_then(|status| status.code()),
            duration_ms,
            cancelled,
            summary: parse_summary(&text),
        };

        let _ = app.emit(EVENT_DONE, payload);

        if let Some(state) = app.try_state::<BuildState>() {
            if let Ok(mut guard) = state.pid.lock() {
                *guard = None;
            }
            state.running.store(false, Ordering::SeqCst);
        }
    });

    Ok(())
}

/// 打开当前项目的 `public/` 目录。路径由后端从项目根拼接，前端无法指定任意路径。
#[tauri::command]
pub fn open_build_output(state: State<AppState>) -> Result<(), String> {
    let root = state
        .root
        .lock()
        .map_err(|_| "项目状态不可用".to_string())?
        .clone()
        .ok_or_else(|| "尚未打开项目".to_string())?;
    let output = root.join("public");

    if !output.is_dir() {
        return Err("public 目录不存在，请先构建".to_string());
    }

    tauri_plugin_opener::open_path(output, None::<&str>)
        .map_err(|error| format!("打开输出目录失败：{error}"))
}

#[tauri::command]
pub fn cancel_build(app: AppHandle) {
    cancel_build_impl(&app);
}

pub fn cancel_build_impl(app: &AppHandle) {
    let Some(state) = app.try_state::<BuildState>() else {
        return;
    };
    state.cancelled.store(true, Ordering::SeqCst);

    if let Ok(guard) = state.pid.lock()
        && let Some(pid) = *guard
    {
        #[cfg(unix)]
        unsafe {
            libc::kill(pid as i32, libc::SIGTERM);
        }
    }
}

fn spawn_stream<R: std::io::Read + Send + 'static>(
    app: AppHandle,
    reader: R,
    stream: &'static str,
    captured: Arc<Mutex<Vec<String>>>,
) {
    std::thread::spawn(move || {
        for line in BufReader::new(reader).lines().map_while(Result::ok) {
            if let Ok(mut lines) = captured.lock()
                && lines.len() < MAX_CAPTURE_LINES
            {
                lines.push(line.clone());
            }
            let _ = app.emit(EVENT_LOG, BuildLogPayload { line, stream });
        }
    });
}

/// 解析 hugo 构建输出末尾的统计表与 `Total in <n> ms`。
pub fn parse_summary(output: &str) -> BuildSummary {
    let mut summary = BuildSummary::default();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if line.starts_with("Pages") && !line.starts_with("Paginator") {
            summary.pages = last_number(line);
        } else if line.starts_with("Static files") {
            summary.static_files = last_number(line);
        } else if line.starts_with("Processed images") {
            summary.processed_images = last_number(line);
        } else if let Some(rest) = line.strip_prefix("Total in ") {
            summary.duration_ms = parse_duration_ms(rest);
        }
    }

    summary
}

fn last_number(line: &str) -> Option<u64> {
    line.split_whitespace().last()?.parse().ok()
}

fn parse_duration_ms(text: &str) -> Option<u64> {
    let mut parts = text.split_whitespace();
    let value: f64 = parts.next()?.parse().ok()?;

    match parts.next()? {
        "ms" => Some(value.round() as u64),
        "s" => Some((value * 1000.0).round() as u64),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SUCCESS_OUTPUT: &str = "Start building sites …\n\
hugo v0.168.0 linux/amd64\n\
\n\
 Pages            │ 11 \n\
 Paginator pages  │  0 \n\
 Non-page files   │  0 \n\
 Static files     │  1 \n\
 Processed images │  0 \n\
 Aliases          │  1 \n\
 Cleaned          │  0 \n\
\n\
Total in 41 ms\n";

    #[test]
    fn parses_hugo_summary_table() {
        let summary = parse_summary(SUCCESS_OUTPUT);

        assert_eq!(summary.pages, Some(11));
        assert_eq!(summary.static_files, Some(1));
        assert_eq!(summary.processed_images, Some(0));
        assert_eq!(summary.duration_ms, Some(41));
    }

    #[test]
    fn parses_seconds_duration() {
        assert_eq!(parse_duration_ms("1.5 s"), Some(1500));
        assert_eq!(parse_duration_ms("287 ms"), Some(287));
        assert_eq!(parse_duration_ms("n/a"), None);
    }

    #[test]
    fn returns_defaults_for_failure_output() {
        let output = "Error: failed to load config: open hugo.toml: no such file or directory\n";
        let summary = parse_summary(output);

        assert_eq!(summary, BuildSummary::default());
    }

    #[test]
    fn ignores_paginator_pages_count() {
        let output = "Paginator pages  │ 3 \n Pages            │ 12 \n";

        let summary = parse_summary(output);

        assert_eq!(summary.pages, Some(12));
    }
}
