use std::path::{Component, Path, PathBuf};

use base64::Engine as _;
use serde::Serialize;
use tauri::State;

use crate::commands::insights::{active_theme_name, find_config_file};
use crate::state::AppState;

const MAX_SCREENSHOT_BYTES: u64 = 2 * 1024 * 1024;
const SCREENSHOTS: [&str; 6] = [
    "images/screenshot.png",
    "images/screenshot.jpg",
    "images/screenshot.jpeg",
    "images/tn.png",
    "images/tn.jpg",
    "screenshot.png",
];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeInfo {
    pub dir: String,
    pub name: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub homepage: Option<String>,
    pub min_version: Option<String>,
    pub active: bool,
    pub has_screenshot: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeList {
    pub themes: Vec<ThemeInfo>,
    pub config_file: String,
    pub can_switch: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Default)]
struct ThemeMeta {
    name: Option<String>,
    description: Option<String>,
    author: Option<String>,
    license: Option<String>,
    homepage: Option<String>,
    min_version: Option<String>,
}

#[tauri::command]
pub fn list_themes(state: State<AppState>) -> Result<ThemeList, String> {
    let root = project_root(&state)?;
    Ok(list_themes_impl(&root))
}

#[tauri::command]
pub fn theme_screenshot(state: State<AppState>, dir: String) -> Result<Option<String>, String> {
    let root = project_root(&state)?;
    screenshot_data_url(&root, &dir)
}

#[tauri::command]
pub fn apply_theme(state: State<AppState>, dir: String) -> Result<(), String> {
    let root = project_root(&state)?;
    apply_theme_impl(&root, &dir)
}

#[tauri::command]
pub fn open_theme_folder(state: State<AppState>, dir: String) -> Result<(), String> {
    let root = project_root(&state)?;
    let path = theme_dir(&root, &dir)?;
    tauri_plugin_opener::open_path(path, None::<&str>)
        .map_err(|error| format!("打开主题目录失败：{error}"))
}

fn project_root(state: &State<AppState>) -> Result<PathBuf, String> {
    state
        .root
        .lock()
        .map_err(|_| "项目状态不可用".to_string())?
        .clone()
        .ok_or_else(|| "尚未打开项目".to_string())
}

fn theme_dir(root: &Path, dir: &str) -> Result<PathBuf, String> {
    let mut components = Path::new(dir).components();
    let valid =
        matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none();
    if !valid {
        return Err("主题目录名不合法".to_string());
    }

    let path = root.join("themes").join(dir);
    if !path.join("theme.toml").is_file() {
        return Err(format!("主题 {dir} 不存在或缺少 theme.toml"));
    }

    Ok(path)
}

pub(crate) fn list_themes_impl(root: &Path) -> ThemeList {
    let active = active_theme_name(root);
    let (config_file, can_switch, reason) = switch_capability(root);
    let mut themes = Vec::new();

    if let Ok(entries) = std::fs::read_dir(root.join("themes")) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() || !path.join("theme.toml").is_file() {
                continue;
            }

            let dir = entry.file_name().to_string_lossy().into_owned();
            let meta = std::fs::read_to_string(path.join("theme.toml"))
                .map(|text| parse_theme_meta(&text))
                .unwrap_or_default();

            themes.push(ThemeInfo {
                name: meta.name.clone().unwrap_or_else(|| dir.clone()),
                dir: dir.clone(),
                description: meta.description,
                author: meta.author,
                license: meta.license,
                homepage: meta.homepage,
                min_version: meta.min_version,
                active: active.as_deref() == Some(dir.as_str()),
                has_screenshot: screenshot_path(&path).is_some(),
            });
        }
    }

    themes.sort_by(|a, b| b.active.cmp(&a.active).then_with(|| a.name.cmp(&b.name)));

    ThemeList {
        themes,
        config_file,
        can_switch,
        reason,
    }
}

pub(crate) fn apply_theme_impl(root: &Path, dir: &str) -> Result<(), String> {
    theme_dir(root, dir)?;

    let config_path = find_config_file(root).ok_or_else(|| "未找到 Hugo 配置文件".to_string())?;
    if !is_toml(&config_path) {
        return Err(
            "当前站点使用 YAML 配置，暂不支持自动切换主题，请手动修改 theme 字段".to_string(),
        );
    }

    let text =
        std::fs::read_to_string(&config_path).map_err(|error| format!("读取配置失败：{error}"))?;
    let mut doc = text
        .parse::<toml_edit::DocumentMut>()
        .map_err(|error| format!("解析配置失败：{error}"))?;

    if has_module_imports(&doc) {
        return Err("该站点通过 Hugo Modules 管理主题，请手动修改配置".to_string());
    }

    doc["theme"] = toml_edit::value(dir);

    let mut content = doc.to_string();
    if !content.ends_with('\n') {
        content.push('\n');
    }

    let metadata =
        std::fs::metadata(&config_path).map_err(|error| format!("读取配置权限失败：{error}"))?;
    let file_name = config_path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .ok_or_else(|| "配置文件路径无效".to_string())?;
    let temp =
        config_path.with_file_name(format!(".{file_name}.hugonic-{}.tmp", std::process::id()));

    std::fs::write(&temp, content).map_err(|error| format!("写入临时配置失败：{error}"))?;
    std::fs::set_permissions(&temp, metadata.permissions())
        .map_err(|error| format!("设置配置权限失败：{error}"))?;

    if let Err(error) = std::fs::rename(&temp, &config_path) {
        let _ = std::fs::remove_file(&temp);
        return Err(format!("保存配置失败：{error}"));
    }

    Ok(())
}

fn switch_capability(root: &Path) -> (String, bool, Option<String>) {
    let Some(path) = find_config_file(root) else {
        return (
            String::new(),
            false,
            Some("未找到 Hugo 配置文件".to_string()),
        );
    };
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();

    if !is_toml(&path) {
        return (
            name,
            false,
            Some("当前站点使用 YAML 配置，暂不支持自动切换主题".to_string()),
        );
    }

    let has_modules = std::fs::read_to_string(&path)
        .ok()
        .and_then(|text| text.parse::<toml_edit::DocumentMut>().ok())
        .map(|doc| has_module_imports(&doc))
        .unwrap_or(false);

    if has_modules {
        return (
            name,
            false,
            Some("该站点通过 Hugo Modules 管理主题，请手动修改配置".to_string()),
        );
    }

    (name, true, None)
}

fn is_toml(path: &Path) -> bool {
    matches!(path.extension().and_then(|ext| ext.to_str()), Some("toml"))
}

fn has_module_imports(doc: &toml_edit::DocumentMut) -> bool {
    doc.get("module")
        .and_then(|item| item.as_table_like())
        .and_then(|table| table.get("imports"))
        .is_some()
}

fn parse_theme_meta(text: &str) -> ThemeMeta {
    let Ok(doc) = text.parse::<toml_edit::DocumentMut>() else {
        return ThemeMeta::default();
    };

    ThemeMeta {
        name: string_field(&doc, "name"),
        description: string_field(&doc, "description"),
        author: author_name(&doc),
        license: string_field(&doc, "license"),
        homepage: string_field(&doc, "homepage"),
        min_version: string_field(&doc, "min_version").or_else(|| string_field(&doc, "minVersion")),
    }
}

fn string_field(doc: &toml_edit::DocumentMut, key: &str) -> Option<String> {
    doc.get(key)
        .and_then(|item| item.as_str())
        .map(str::to_string)
}

fn author_name(doc: &toml_edit::DocumentMut) -> Option<String> {
    doc.get("author").and_then(|item| {
        item.as_str().map(str::to_string).or_else(|| {
            item.as_table_like()
                .and_then(|table| table.get("name"))
                .and_then(|name| name.as_str())
                .map(str::to_string)
        })
    })
}

fn screenshot_path(theme: &Path) -> Option<PathBuf> {
    SCREENSHOTS
        .iter()
        .map(|relative| theme.join(relative))
        .find(|path| path.is_file())
}

pub(crate) fn screenshot_data_url(root: &Path, dir: &str) -> Result<Option<String>, String> {
    let theme = theme_dir(root, dir)?;
    let Some(path) = screenshot_path(&theme) else {
        return Ok(None);
    };

    let metadata =
        std::fs::metadata(&path).map_err(|error| format!("读取截图信息失败：{error}"))?;
    if metadata.len() > MAX_SCREENSHOT_BYTES {
        return Ok(None);
    }

    let bytes = std::fs::read(&path).map_err(|error| format!("读取截图失败：{error}"))?;
    let mime = match path.extension().and_then(|ext| ext.to_str()) {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        _ => "image/png",
    };
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);

    Ok(Some(format!("data:{mime};base64,{encoded}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn project(with_yaml: bool) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("themes/ananke/images")).unwrap();
        fs::create_dir_all(root.join("themes/other")).unwrap();

        fs::write(
            root.join("hugo.toml"),
            "# 保留这行注释\nbaseURL = 'https://example.org/'\ntheme = 'ananke'\n",
        )
        .unwrap();
        if with_yaml {
            fs::remove_file(root.join("hugo.toml")).unwrap();
            fs::write(root.join("hugo.yaml"), "theme: ananke\n").unwrap();
        }

        fs::write(
            root.join("themes/ananke/theme.toml"),
            "name = 'Ananke'\ndescription = 'Demo theme'\nlicense = 'MIT'\nmin_version = '0.128.0'\n\n[author]\nname = 'Bud Parr'\nhomepage = 'https://example.com'\n",
        )
        .unwrap();
        fs::write(root.join("themes/other/theme.toml"), "name = 'Other'\n").unwrap();
        fs::write(
            root.join("themes/ananke/images/screenshot.png"),
            b"\x89PNG\r\n\x1a\n",
        )
        .unwrap();

        dir
    }

    #[test]
    fn lists_themes_with_metadata_and_active_flag() {
        let dir = project(false);
        let list = list_themes_impl(dir.path());

        assert!(list.can_switch);
        assert_eq!(list.themes.len(), 2);
        assert_eq!(list.themes[0].dir, "ananke");
        assert!(list.themes[0].active);
        assert_eq!(list.themes[0].name, "Ananke");
        assert_eq!(list.themes[0].author.as_deref(), Some("Bud Parr"));
        assert_eq!(list.themes[0].license.as_deref(), Some("MIT"));
        assert_eq!(list.themes[0].min_version.as_deref(), Some("0.128.0"));
        assert!(list.themes[0].has_screenshot);
    }

    #[test]
    fn switches_theme_and_keeps_comments() {
        let dir = project(false);

        apply_theme_impl(dir.path(), "other").unwrap();

        let config = fs::read_to_string(dir.path().join("hugo.toml")).unwrap();
        assert!(config.contains("# 保留这行注释"));
        assert!(config.contains("theme = \"other\""));
        assert!(!config.contains("theme = 'ananke'"));
    }

    #[test]
    fn rejects_unknown_theme() {
        let dir = project(false);

        let error = apply_theme_impl(dir.path(), "missing").unwrap_err();

        assert!(error.contains("不存在"));
    }

    #[test]
    fn yaml_config_is_read_only() {
        let dir = project(true);
        let list = list_themes_impl(dir.path());

        assert!(!list.can_switch);
        assert!(list.reason.is_some());
        assert!(apply_theme_impl(dir.path(), "other").is_err());
    }

    #[test]
    fn module_imports_are_read_only() {
        let dir = project(false);
        fs::write(
            dir.path().join("hugo.toml"),
            "[module]\n[[module.imports]]\npath = 'github.com/example/theme'\n",
        )
        .unwrap();

        let list = list_themes_impl(dir.path());

        assert!(!list.can_switch);
    }

    #[test]
    fn builds_screenshot_data_url() {
        let dir = project(false);

        let url = screenshot_data_url(dir.path(), "ananke").unwrap().unwrap();

        assert!(url.starts_with("data:image/png;base64,"));
    }
}
