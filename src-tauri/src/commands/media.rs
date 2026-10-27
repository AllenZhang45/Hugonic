use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Component, Path, PathBuf};
use std::time::UNIX_EPOCH;

use base64::Engine as _;
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::commands::files::resolve_within;
use crate::state::AppState;

const IMAGE_EXTENSIONS: [&str; 8] = ["jpg", "jpeg", "png", "webp", "gif", "svg", "avif", "bmp"];
const MAX_ITEMS: usize = 1000;
const MAX_IMPORT_BYTES: u64 = 20 * 1024 * 1024;
const MAX_SVG_BYTES: u64 = 256 * 1024;
const THUMB_SIZE: u32 = 360;
const THUMB_QUALITY: u8 = 75;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaItem {
    pub path: String,
    pub name: String,
    pub dir: String,
    pub size: u64,
    pub modified_ms: u64,
    pub extension: String,
}

#[tauri::command]
pub fn list_media(state: State<AppState>) -> Result<Vec<MediaItem>, String> {
    let root = project_root(&state)?;
    Ok(list_media_impl(&root))
}

#[tauri::command]
pub fn media_thumbnail(
    app: AppHandle,
    state: State<AppState>,
    path: String,
) -> Result<String, String> {
    let root = project_root(&state)?;
    let cache = app
        .path()
        .app_cache_dir()
        .map_err(|error| format!("无法定位应用缓存目录：{error}"))?
        .join("media-thumbs");

    thumbnail_data_url(&root, &cache, &path)
}

#[tauri::command]
pub fn import_media(
    state: State<AppState>,
    source: String,
    target_dir: String,
) -> Result<MediaItem, String> {
    let root = project_root(&state)?;
    import_media_impl(&root, Path::new(&source), &target_dir)
}

#[tauri::command]
pub fn open_media(state: State<AppState>, path: String) -> Result<(), String> {
    let root = project_root(&state)?;
    let resolved = resolve_within(&root, &path)?;
    if !resolved.is_file() {
        return Err("文件不存在".to_string());
    }
    tauri_plugin_opener::open_path(resolved, None::<&str>)
        .map_err(|error| format!("打开文件失败：{error}"))
}

fn project_root(state: &State<AppState>) -> Result<PathBuf, String> {
    state
        .root
        .lock()
        .map_err(|_| "项目状态不可用".to_string())?
        .clone()
        .ok_or_else(|| "尚未打开项目".to_string())
}

pub(crate) fn list_media_impl(root: &Path) -> Vec<MediaItem> {
    let mut items = Vec::new();

    for dir in ["static", "assets", "content"] {
        collect_media(root, &root.join(dir), &mut items);
        if items.len() >= MAX_ITEMS {
            break;
        }
    }

    items.sort_by(|a, b| {
        b.modified_ms
            .cmp(&a.modified_ms)
            .then_with(|| a.path.cmp(&b.path))
    });
    items.truncate(MAX_ITEMS);
    items
}

fn collect_media(root: &Path, dir: &Path, items: &mut Vec<MediaItem>) {
    if items.len() >= MAX_ITEMS {
        return;
    }

    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        if items.len() >= MAX_ITEMS {
            return;
        }

        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            collect_media(root, &path, items);
            continue;
        }

        if let Some(item) = media_item(root, &path) {
            items.push(item);
        }
    }
}

fn media_item(root: &Path, path: &Path) -> Option<MediaItem> {
    let extension = path.extension()?.to_str()?.to_lowercase();
    if !IMAGE_EXTENSIONS.contains(&extension.as_str()) {
        return None;
    }

    let metadata = std::fs::metadata(path).ok()?;
    let relative = path
        .strip_prefix(root)
        .ok()?
        .to_string_lossy()
        .replace('\\', "/");
    let name = path.file_name()?.to_string_lossy().into_owned();
    let dir = Path::new(&relative)
        .parent()
        .map(|parent| parent.to_string_lossy().replace('\\', "/"))
        .unwrap_or_default();
    let modified_ms = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0);

    Some(MediaItem {
        path: relative,
        name,
        dir,
        size: metadata.len(),
        modified_ms,
        extension,
    })
}

pub(crate) fn import_media_impl(
    root: &Path,
    source: &Path,
    target_dir: &str,
) -> Result<MediaItem, String> {
    validate_target_dir(target_dir)?;

    if !source.is_file() {
        return Err("源文件不存在".to_string());
    }

    let extension = source
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_lowercase)
        .ok_or_else(|| "文件没有扩展名".to_string())?;
    if !IMAGE_EXTENSIONS.contains(&extension.as_str()) {
        return Err(format!("不支持的图片格式：{extension}"));
    }

    let metadata = std::fs::metadata(source).map_err(|error| format!("读取源文件失败：{error}"))?;
    if metadata.len() > MAX_IMPORT_BYTES {
        return Err("图片超过 20 MB，暂不支持导入".to_string());
    }

    let target = root.join(target_dir);
    std::fs::create_dir_all(&target).map_err(|error| format!("创建目录失败：{error}"))?;

    let stem = source
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty())
        .unwrap_or("image");
    let mut destination = target.join(format!("{stem}.{extension}"));
    let mut index = 1;
    while destination.exists() {
        destination = target.join(format!("{stem}-{index}.{extension}"));
        index += 1;
    }

    std::fs::copy(source, &destination).map_err(|error| format!("复制图片失败：{error}"))?;

    media_item(root, &destination).ok_or_else(|| "导入后读取文件信息失败".to_string())
}

fn validate_target_dir(target_dir: &str) -> Result<(), String> {
    let path = Path::new(target_dir);
    if path.is_absolute() {
        return Err("目标目录必须是项目内相对路径".to_string());
    }

    let mut components = path.components();
    let first = components.next().and_then(|component| match component {
        Component::Normal(name) => name.to_str(),
        _ => None,
    });

    if !matches!(first, Some("static") | Some("assets")) {
        return Err("图片只能导入到 static/ 或 assets/ 下".to_string());
    }

    if path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err("目标目录不能包含 ..".to_string());
    }

    Ok(())
}

pub(crate) fn thumbnail_data_url(
    root: &Path,
    cache_dir: &Path,
    relative: &str,
) -> Result<String, String> {
    let path = resolve_within(root, relative)?;
    if !path.is_file() {
        return Err("文件不存在".to_string());
    }

    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_lowercase)
        .ok_or_else(|| "文件没有扩展名".to_string())?;
    if !IMAGE_EXTENSIONS.contains(&extension.as_str()) {
        return Err("不是图片文件".to_string());
    }

    let metadata =
        std::fs::metadata(&path).map_err(|error| format!("读取文件信息失败：{error}"))?;
    let modified_ms = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0);

    if extension == "svg" {
        if metadata.len() > MAX_SVG_BYTES {
            return Err("SVG 文件过大".to_string());
        }
        let bytes = std::fs::read(&path).map_err(|error| format!("读取文件失败：{error}"))?;
        let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
        return Ok(format!("data:image/svg+xml;base64,{encoded}"));
    }

    let mut hasher = DefaultHasher::new();
    relative.hash(&mut hasher);
    modified_ms.hash(&mut hasher);
    metadata.len().hash(&mut hasher);
    let cache_file = cache_dir.join(format!("{:016x}.jpg", hasher.finish()));

    if !cache_file.is_file() {
        let image = image::ImageReader::open(&path)
            .map_err(|error| format!("打开图片失败：{error}"))?
            .with_guessed_format()
            .map_err(|error| format!("识别图片格式失败：{error}"))?
            .decode()
            .map_err(|error| format!("解码图片失败：{error}"))?;
        let thumbnail = image.thumbnail(THUMB_SIZE, THUMB_SIZE);
        let mut buffer = Vec::new();
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buffer, THUMB_QUALITY)
            .encode_image(&thumbnail)
            .map_err(|error| format!("生成缩略图失败：{error}"))?;

        std::fs::create_dir_all(cache_dir).map_err(|error| format!("创建缓存目录失败：{error}"))?;
        std::fs::write(&cache_file, &buffer)
            .map_err(|error| format!("写入缩略图缓存失败：{error}"))?;

        let encoded = base64::engine::general_purpose::STANDARD.encode(&buffer);
        return Ok(format!("data:image/jpeg;base64,{encoded}"));
    }

    let bytes =
        std::fs::read(&cache_file).map_err(|error| format!("读取缩略图缓存失败：{error}"))?;
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    Ok(format!("data:image/jpeg;base64,{encoded}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn project() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("static/images")).unwrap();
        fs::create_dir_all(root.join("assets")).unwrap();
        fs::create_dir_all(root.join("content/posts/hello")).unwrap();
        fs::write(root.join("static/images/a.png"), b"png").unwrap();
        fs::write(root.join("static/style.css"), b"body{}").unwrap();
        fs::write(root.join("assets/logo.svg"), "<svg/>").unwrap();
        fs::write(root.join("content/posts/hello/cover.jpg"), b"jpg").unwrap();
        fs::write(root.join("content/readme.md"), "# readme").unwrap();
        dir
    }

    #[test]
    fn lists_images_from_static_assets_and_bundles() {
        let dir = project();

        let items = list_media_impl(dir.path());

        let paths: Vec<&str> = items.iter().map(|item| item.path.as_str()).collect();
        assert!(paths.contains(&"static/images/a.png"));
        assert!(paths.contains(&"assets/logo.svg"));
        assert!(paths.contains(&"content/posts/hello/cover.jpg"));
        assert!(!paths.iter().any(|path| path.ends_with(".css")));
        assert!(!paths.iter().any(|path| path.ends_with(".md")));
    }

    #[test]
    fn imports_with_unique_names() {
        let dir = project();
        let source = dir.path().join("incoming.png");
        fs::write(&source, b"png").unwrap();

        let first = import_media_impl(dir.path(), &source, "static/images").unwrap();
        let second = import_media_impl(dir.path(), &source, "static/images").unwrap();

        assert_eq!(first.name, "incoming.png");
        assert_eq!(second.name, "incoming-1.png");
        assert!(dir.path().join("static/images/incoming-1.png").is_file());
    }

    #[test]
    fn rejects_escape_and_bad_formats() {
        let dir = project();
        let source = dir.path().join("note.txt");
        fs::write(&source, b"text").unwrap();

        assert!(import_media_impl(dir.path(), &source, "static/images").is_err());
        assert!(import_media_impl(dir.path(), &source, "../outside").is_err());
        assert!(import_media_impl(dir.path(), &source, "content").is_err());
    }

    #[test]
    fn generates_and_caches_thumbnail() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("site");
        let cache = dir.path().join("cache");
        fs::create_dir_all(root.join("static/images")).unwrap();

        let image = image::RgbaImage::from_pixel(800, 600, image::Rgba([99, 102, 241, 255]));
        image.save(root.join("static/images/cover.png")).unwrap();

        let url = thumbnail_data_url(&root, &cache, "static/images/cover.png").unwrap();
        assert!(url.starts_with("data:image/jpeg;base64,"));
        assert_eq!(fs::read_dir(&cache).unwrap().count(), 1);

        let cached = thumbnail_data_url(&root, &cache, "static/images/cover.png").unwrap();
        assert_eq!(fs::read_dir(&cache).unwrap().count(), 1);
        assert_eq!(url, cached);
    }

    #[test]
    fn returns_svg_data_url() {
        let dir = project();

        let url =
            thumbnail_data_url(dir.path(), &dir.path().join("cache"), "assets/logo.svg").unwrap();

        assert!(url.starts_with("data:image/svg+xml;base64,"));
    }
}
