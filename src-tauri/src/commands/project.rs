use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;
use tauri::{AppHandle, State};

use crate::state::AppState;

const CONFIG_FILES: [&str; 4] = ["hugo.toml", "config.toml", "config.yaml", "config.yml"];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInfo {
    pub root: String,
    pub name: String,
    pub config_file: String,
    pub hugo_version: Option<String>,
}

/// 校验目录是否是 Hugo 站点，返回规范化后的根路径、目录名与命中的配置文件。
pub fn inspect_project(path: &Path) -> Result<(PathBuf, String, String), String> {
    let root = path
        .canonicalize()
        .map_err(|error| format!("无法访问目录：{error}"))?;

    if !root.is_dir() {
        return Err("选择的路径不是目录".to_string());
    }

    let config_file = CONFIG_FILES
        .iter()
        .find(|name| root.join(name).is_file())
        .ok_or_else(|| "不是 Hugo 站点：缺少 hugo.toml 或 config.*".to_string())?;

    let name = root
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| root.to_string_lossy().into_owned());

    Ok((root, name, (*config_file).to_string()))
}

#[tauri::command]
pub fn open_project(
    path: String,
    app: AppHandle,
    state: State<AppState>,
) -> Result<ProjectInfo, String> {
    let (root, name, config_file) = inspect_project(Path::new(&path))?;

    // 切换项目时先结束旧的预览与构建进程。
    crate::commands::preview::stop_preview_impl(&app);
    crate::commands::build::cancel_build_impl(&app);

    let hugo_version = detect_hugo_version();

    let info = ProjectInfo {
        root: root.to_string_lossy().into_owned(),
        name,
        config_file,
        hugo_version,
    };

    let mut current = state
        .root
        .lock()
        .map_err(|_| "项目状态不可用".to_string())?;
    *current = Some(root);

    Ok(info)
}

/// 尽力从 PATH 中探测 Hugo 版本。M3 接入 sidecar 后改用打包的二进制。
fn detect_hugo_version() -> Option<String> {
    let output = Command::new("hugo").arg("version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    parse_hugo_version(&String::from_utf8_lossy(&output.stdout))
}

/// 从 `hugo version` 的首行解析版本号：`hugo v0.168.0-e27a6760 +extended linux/amd64` → `0.168.0`
pub fn parse_hugo_version(output: &str) -> Option<String> {
    let first_line = output.lines().next()?;
    let mut parts = first_line.split_whitespace();

    if parts.next()? != "hugo" {
        return None;
    }

    let token = parts.next()?;
    let version = token.strip_prefix('v')?.split('-').next()?;

    if version.starts_with(|character: char| character.is_ascii_digit()) {
        Some(version.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parses_extended_version_output() {
        let output = "hugo v0.168.0-e27a6760c554 +extended linux/amd64 BuildDate=2026-10-13\n";

        assert_eq!(parse_hugo_version(output).as_deref(), Some("0.168.0"));
    }

    #[test]
    fn returns_none_for_unexpected_output() {
        assert_eq!(parse_hugo_version(""), None);
        assert_eq!(parse_hugo_version("command not found"), None);
    }

    #[test]
    fn accepts_directory_with_hugo_config() {
        let dir = tempfile::tempdir().expect("temp dir");
        fs::write(dir.path().join("hugo.toml"), "baseURL = '/'\n").expect("write config");

        let (root, name, config) = inspect_project(dir.path()).expect("valid project");

        assert_eq!(root, dir.path().canonicalize().expect("canonical"));
        assert_eq!(config, "hugo.toml");
        assert!(!name.is_empty());
    }

    #[test]
    fn rejects_directory_without_config() {
        let dir = tempfile::tempdir().expect("temp dir");

        let error = inspect_project(dir.path()).expect_err("should reject");

        assert!(
            error.contains("不是 Hugo 站点"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn rejects_file_path() {
        let dir = tempfile::tempdir().expect("temp dir");
        let file = dir.path().join("hugo.toml");
        fs::write(&file, "").expect("write file");

        let error = inspect_project(&file).expect_err("should reject");

        assert!(error.contains("不是目录"), "unexpected error: {error}");
    }
}
