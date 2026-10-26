use std::fs::Metadata;
use std::path::{Component, Path, PathBuf};
use std::time::UNIX_EPOCH;

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

pub const MAX_EDITABLE_BYTES: u64 = 2 * 1024 * 1024;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileContent {
    pub path: String,
    pub content: String,
    pub size: u64,
    pub mtime_ms: u64,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileMeta {
    pub size: u64,
    pub mtime_ms: u64,
}

#[tauri::command]
pub fn read_file(path: String, state: State<AppState>) -> Result<FileContent, String> {
    let root = current_root(&state)?;
    read_file_content(&root, &path)
}

#[tauri::command]
pub fn save_file(
    path: String,
    content: String,
    state: State<AppState>,
) -> Result<FileMeta, String> {
    let root = current_root(&state)?;
    write_file_atomic(&root, &path, &content)
}

#[tauri::command]
pub fn file_meta(path: String, state: State<AppState>) -> Result<FileMeta, String> {
    let root = current_root(&state)?;
    read_file_meta(&root, &path)
}

fn current_root(state: &State<AppState>) -> Result<PathBuf, String> {
    state
        .root
        .lock()
        .map_err(|_| "项目状态不可用".to_string())?
        .clone()
        .ok_or_else(|| "尚未打开项目".to_string())
}

pub fn read_file_content(root: &Path, relative: &str) -> Result<FileContent, String> {
    let target = resolve_within(root, relative)?;
    let metadata = std::fs::metadata(&target).map_err(|error| format!("无法读取文件：{error}"))?;

    if !metadata.is_file() {
        return Err("只能打开文件".to_string());
    }

    if metadata.len() > MAX_EDITABLE_BYTES {
        return Err(format!(
            "文件大小 {:.1} MB，超过 {} MB 上限，暂不支持编辑",
            metadata.len() as f64 / 1024.0 / 1024.0,
            MAX_EDITABLE_BYTES / 1024 / 1024
        ));
    }

    let bytes = std::fs::read(&target).map_err(|error| format!("无法读取文件：{error}"))?;
    let content =
        String::from_utf8(bytes).map_err(|_| "文件不是 UTF-8 编码，暂不支持编辑".to_string())?;

    Ok(FileContent {
        path: relative.to_string(),
        content,
        size: metadata.len(),
        mtime_ms: modified_ms(&metadata)?,
    })
}

/// 临时文件 + rename：写到同目录的临时文件，保留原权限后再原子替换。
pub fn write_file_atomic(root: &Path, relative: &str, content: &str) -> Result<FileMeta, String> {
    let target = resolve_within(root, relative)?;
    let metadata = std::fs::metadata(&target).map_err(|error| format!("无法访问文件：{error}"))?;

    if !metadata.is_file() {
        return Err("只能保存文件".to_string());
    }

    if metadata.permissions().readonly() {
        return Err("文件是只读的，无法保存".to_string());
    }

    let parent = target
        .parent()
        .ok_or_else(|| "无效的文件路径".to_string())?;
    let file_name = target
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .ok_or_else(|| "无效的文件名".to_string())?;
    let temp = parent.join(format!(".{file_name}.hugonic-{}.tmp", std::process::id()));

    let write_result = (|| -> Result<(), String> {
        std::fs::write(&temp, content.as_bytes())
            .map_err(|error| format!("写入临时文件失败：{error}"))?;
        std::fs::set_permissions(&temp, metadata.permissions())
            .map_err(|error| format!("设置文件权限失败：{error}"))?;
        std::fs::rename(&temp, &target).map_err(|error| format!("保存失败：{error}"))?;
        Ok(())
    })();

    if let Err(error) = write_result {
        let _ = std::fs::remove_file(&temp);
        return Err(error);
    }

    let saved = std::fs::metadata(&target).map_err(|error| format!("无法读取保存结果：{error}"))?;

    Ok(FileMeta {
        size: saved.len(),
        mtime_ms: modified_ms(&saved)?,
    })
}

pub fn read_file_meta(root: &Path, relative: &str) -> Result<FileMeta, String> {
    let target = resolve_within(root, relative)?;
    let metadata = std::fs::metadata(&target).map_err(|error| format!("无法访问文件：{error}"))?;

    if !metadata.is_file() {
        return Err("只能查询文件".to_string());
    }

    Ok(FileMeta {
        size: metadata.len(),
        mtime_ms: modified_ms(&metadata)?,
    })
}

fn modified_ms(metadata: &Metadata) -> Result<u64, String> {
    let modified = metadata
        .modified()
        .map_err(|error| format!("无法读取修改时间：{error}"))?;
    let duration = modified
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("修改时间异常：{error}"))?;

    Ok(duration.as_millis() as u64)
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

#[cfg(test)]
mod file_tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    fn project_with_file(name: &str, content: &str) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().expect("temp dir");
        fs::write(dir.path().join("hugo.toml"), "").expect("config");
        fs::write(dir.path().join(name), content).expect("file");
        let root = dir.path().canonicalize().expect("canonical");
        (dir, root)
    }

    #[test]
    fn reads_utf8_file_and_reports_meta() {
        let (_dir, root) = project_with_file("post.md", "# 标题\n");

        let file = read_file_content(&root, "post.md").expect("read");

        assert_eq!(file.content, "# 标题\n");
        assert_eq!(file.size, "# 标题\n".len() as u64);
        assert!(file.mtime_ms > 0);
    }

    #[test]
    fn saves_atomically_and_leaves_no_temp_file() {
        let (dir, root) = project_with_file("post.md", "old");

        let meta = write_file_atomic(&root, "post.md", "new").expect("save");

        assert_eq!(
            fs::read_to_string(root.join("post.md")).expect("read back"),
            "new"
        );
        assert_eq!(meta.size, 3);

        let leftovers: Vec<String> = fs::read_dir(dir.path())
            .expect("read dir")
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| name.contains(".hugonic-"))
            .collect();
        assert!(leftovers.is_empty(), "leftover temp files: {leftovers:?}");
    }

    #[test]
    fn keeps_file_permissions_on_save() {
        let (_dir, root) = project_with_file("post.md", "old");
        let path = root.join("post.md");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).expect("chmod");

        write_file_atomic(&root, "post.md", "new").expect("save");

        let mode = fs::metadata(&path).expect("meta").permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }

    #[test]
    fn refuses_to_save_readonly_file() {
        let (_dir, root) = project_with_file("post.md", "old");
        fs::set_permissions(root.join("post.md"), fs::Permissions::from_mode(0o444))
            .expect("chmod");

        let error = write_file_atomic(&root, "post.md", "new").expect_err("should fail");

        assert!(error.contains("只读"), "unexpected error: {error}");
    }

    #[test]
    fn rejects_file_larger_than_limit() {
        let (_dir, root) = project_with_file("big.md", "");
        let file = fs::File::create(root.join("big.md")).expect("create");
        file.set_len(MAX_EDITABLE_BYTES + 1).expect("sparse");

        let error = read_file_content(&root, "big.md").expect_err("should reject");

        assert!(error.contains("超过"), "unexpected error: {error}");
    }

    #[test]
    fn rejects_non_utf8_file() {
        let dir = tempfile::tempdir().expect("temp dir");
        fs::write(dir.path().join("hugo.toml"), "").expect("config");
        fs::write(dir.path().join("bin.dat"), [0xff, 0xfe, 0x00]).expect("binary");
        let root = dir.path().canonicalize().expect("canonical");

        let error = read_file_content(&root, "bin.dat").expect_err("should reject");

        assert!(error.contains("UTF-8"), "unexpected error: {error}");
    }

    #[test]
    fn rejects_traversal_when_reading() {
        let (_dir, root) = project_with_file("post.md", "x");

        let error = read_file_content(&root, "../post.md").expect_err("should reject");

        assert_eq!(error, "路径不能包含 ..");
    }
}
