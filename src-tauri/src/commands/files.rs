use std::path::{Component, Path, PathBuf};

use serde::Serialize;
use tauri::State;

use crate::state::AppState;

/// 不做展示、也不参与读取的目录与文件。
const IGNORED: [&str; 6] = [
    ".git",
    "public",
    "resources",
    "node_modules",
    ".hugo_build.lock",
    ".DS_Store",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryKind {
    Dir,
    File,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Entry {
    pub name: String,
    /// 相对项目根目录的路径，使用 `/` 分隔。
    pub path: String,
    pub kind: EntryKind,
}

#[tauri::command]
pub fn read_dir(path: String, state: State<AppState>) -> Result<Vec<Entry>, String> {
    let root = state
        .root
        .lock()
        .map_err(|_| "项目状态不可用".to_string())?
        .clone()
        .ok_or_else(|| "尚未打开项目".to_string())?;

    read_entries(&root, &path)
}

/// 读取 `root/relative` 这一层目录，按「目录优先、名称不区分大小写」排序。
pub fn read_entries(root: &Path, relative: &str) -> Result<Vec<Entry>, String> {
    let directory = resolve_within(root, relative)?;
    let reader = std::fs::read_dir(&directory).map_err(|error| format!("无法读取目录：{error}"))?;

    let mut entries = Vec::new();

    for item in reader {
        let item = item.map_err(|error| format!("无法读取目录项：{error}"))?;
        let name = item.file_name().to_string_lossy().into_owned();

        if is_ignored(&name) {
            continue;
        }

        let file_type = item
            .file_type()
            .map_err(|error| format!("无法读取文件类型：{error}"))?;

        // 符号链接等特殊项不进入文件树，避免跳出项目根目录。
        let kind = if file_type.is_dir() {
            EntryKind::Dir
        } else if file_type.is_file() {
            EntryKind::File
        } else {
            continue;
        };

        entries.push(Entry {
            path: join_relative(relative, &name),
            name,
            kind,
        });
    }

    entries.sort_by(|left, right| {
        let left_is_dir = left.kind == EntryKind::Dir;
        let right_is_dir = right.kind == EntryKind::Dir;

        right_is_dir
            .cmp(&left_is_dir)
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
    });

    Ok(entries)
}

fn is_ignored(name: &str) -> bool {
    IGNORED
        .iter()
        .any(|ignored| ignored.eq_ignore_ascii_case(name))
}

fn join_relative(parent: &str, name: &str) -> String {
    if parent.is_empty() {
        name.to_string()
    } else {
        format!("{parent}/{name}")
    }
}

/// 把相对路径解析到项目根目录内，拒绝绝对路径、`..` 以及指向外部的符号链接。
pub fn resolve_within(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let candidate = if relative.is_empty() {
        root.to_path_buf()
    } else {
        let requested = Path::new(relative);

        if requested.is_absolute() {
            return Err("只接受相对路径".to_string());
        }

        if requested
            .components()
            .any(|component| matches!(component, Component::ParentDir))
        {
            return Err("路径不能包含 ..".to_string());
        }

        root.join(requested)
    };

    let resolved = candidate
        .canonicalize()
        .map_err(|error| format!("无法访问路径：{error}"))?;

    if !resolved.starts_with(root) {
        return Err("路径越界".to_string());
    }

    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::symlink;

    fn sample_project() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("temp dir");
        let root = dir.path();

        fs::create_dir_all(root.join("content/posts")).expect("content dir");
        fs::create_dir_all(root.join("assets")).expect("assets dir");
        fs::create_dir_all(root.join("public")).expect("public dir");
        fs::create_dir_all(root.join("node_modules")).expect("node_modules dir");
        fs::create_dir_all(root.join(".git")).expect(".git dir");
        fs::write(root.join("hugo.toml"), "baseURL = '/'\n").expect("config");
        fs::write(root.join("zeta.md"), "").expect("zeta");
        fs::write(root.join("Alpha.md"), "").expect("alpha");

        dir
    }

    #[test]
    fn lists_directories_first_then_files_alphabetically() {
        let project = sample_project();
        let root = project.path().canonicalize().expect("canonical");

        let entries = read_entries(&root, "").expect("read root");
        let names: Vec<&str> = entries.iter().map(|entry| entry.name.as_str()).collect();

        assert_eq!(
            names,
            vec!["assets", "content", "Alpha.md", "hugo.toml", "zeta.md"]
        );
        assert_eq!(entries[0].kind, EntryKind::Dir);
        assert_eq!(entries[2].kind, EntryKind::File);
        assert_eq!(entries[0].path, "assets");
    }

    #[test]
    fn hides_ignored_entries() {
        let project = sample_project();
        let root = project.path().canonicalize().expect("canonical");

        let entries = read_entries(&root, "").expect("read root");
        let names: Vec<&str> = entries.iter().map(|entry| entry.name.as_str()).collect();

        for ignored in [".git", "public", "node_modules"] {
            assert!(!names.contains(&ignored), "{ignored} should be hidden");
        }
    }

    #[test]
    fn reads_nested_directory_with_prefixed_paths() {
        let project = sample_project();
        let root = project.path().canonicalize().expect("canonical");

        let entries = read_entries(&root, "content").expect("read content");

        assert_eq!(
            entries,
            vec![Entry {
                name: "posts".to_string(),
                path: "content/posts".to_string(),
                kind: EntryKind::Dir,
            }]
        );
    }

    #[test]
    fn rejects_parent_directory_segments() {
        let project = sample_project();
        let root = project.path().canonicalize().expect("canonical");

        let error = resolve_within(&root, "../etc").expect_err("should reject");

        assert_eq!(error, "路径不能包含 ..");
    }

    #[test]
    fn rejects_absolute_paths() {
        let project = sample_project();
        let root = project.path().canonicalize().expect("canonical");

        let error = resolve_within(&root, "/etc").expect_err("should reject");

        assert_eq!(error, "只接受相对路径");
    }

    #[test]
    fn rejects_symlink_escaping_the_project() {
        let project = sample_project();
        let outside = tempfile::tempdir().expect("outside dir");
        let root = project.path().canonicalize().expect("canonical");

        symlink(outside.path(), root.join("escape")).expect("symlink");

        let error = resolve_within(&root, "escape").expect_err("should reject");

        assert_eq!(error, "路径越界");
    }

    #[test]
    fn skips_symlink_entries() {
        let project = sample_project();
        let outside = tempfile::tempdir().expect("outside dir");
        let root = project.path().canonicalize().expect("canonical");

        symlink(outside.path(), root.join("escape")).expect("symlink");

        let entries = read_entries(&root, "").expect("read root");

        assert!(!entries.iter().any(|entry| entry.name == "escape"));
    }
}
