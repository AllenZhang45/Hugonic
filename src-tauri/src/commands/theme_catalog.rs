use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::Engine as _;
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

const SITE: &str = "https://themes.gohugo.io/";
const CATALOG_TTL_SECS: u64 = 7 * 24 * 60 * 60;
const SCHEMA: u32 = 2;
const BUILDER_CACHE: &str = "https://raw.githubusercontent.com/gohugoio/hugoThemesSiteBuilder/main/cmd/hugothemesitebuilder/build/cache/githubrepos.json";
const MAX_SCREENSHOT_BYTES: u64 = 5 * 1024 * 1024;
const MAX_COMMANDS: usize = 6;

#[derive(Default)]
pub struct CatalogState {
    running: AtomicBool,
    progress: Mutex<SyncProgress>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncProgress {
    pub phase: String,
    pub done: u32,
    pub total: u32,
    pub failed: u32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogTheme {
    pub slug: String,
    pub name: String,
    pub authors: Vec<String>,
    pub description: String,
    pub tags: Vec<String>,
    pub license: Option<String>,
    pub stars: Option<u64>,
    pub updated: Option<String>,
    pub repo: String,
    pub demo: Option<String>,
    pub screenshot_url: Option<String>,
    pub install_commands: Vec<String>,
    pub install_extracted: bool,
    pub install_source: Option<String>,
    #[serde(default)]
    pub repo_updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogCache {
    pub schema: u32,
    pub fetched_at: u64,
    pub checked_at: u64,
    pub complete: bool,
    pub source: String,
    pub themes: Vec<CatalogTheme>,
}

#[derive(Debug)]
struct CatalogCard {
    slug: String,
    name: String,
    screenshot_url: Option<String>,
}

#[derive(Debug, Default)]
struct ThemeDetail {
    description: String,
    authors: Vec<String>,
    tags: Vec<String>,
    license: Option<String>,
    stars: Option<u64>,
    updated: Option<String>,
    repo: Option<String>,
    demo: Option<String>,
    screenshot_url: Option<String>,
}

#[tauri::command]
pub fn load_theme_catalog(app: AppHandle) -> Result<Option<CatalogCache>, String> {
    read_cache(&app)
}

#[tauri::command]
pub fn theme_catalog_running(state: State<CatalogState>) -> bool {
    state.running.load(Ordering::SeqCst)
}

#[tauri::command]
pub fn theme_catalog_progress(state: State<CatalogState>) -> SyncProgress {
    state
        .progress
        .lock()
        .map(|progress| progress.clone())
        .unwrap_or_default()
}

#[tauri::command]
pub fn start_theme_sync(
    app: AppHandle,
    state: State<CatalogState>,
    force: bool,
) -> Result<bool, String> {
    if state.running.swap(true, Ordering::SeqCst) {
        return Ok(false);
    }

    let cached = read_cache(&app).ok().flatten();
    let mode = match &cached {
        _ if force => SyncMode::Full,
        None => SyncMode::Full,
        Some(cache) if !cache.complete => SyncMode::Full,
        Some(cache) if !is_fresh(cache.checked_at) => SyncMode::Incremental,
        Some(_) => {
            state.running.store(false, Ordering::SeqCst);
            return Ok(false);
        }
    };

    {
        let mut progress = state
            .progress
            .lock()
            .map_err(|_| "同步状态不可用".to_string())?;
        *progress = SyncProgress {
            phase: "starting".to_string(),
            message: "准备同步主题目录…".to_string(),
            ..SyncProgress::default()
        };
    }

    let handle = app.clone();
    let spawned = std::thread::Builder::new()
        .name("theme-catalog".to_string())
        .spawn(move || run_sync(handle, mode));

    if spawned.is_err() {
        state.running.store(false, Ordering::SeqCst);
        return Err("无法启动同步线程".to_string());
    }

    Ok(true)
}

#[tauri::command]
pub fn catalog_screenshot(app: AppHandle, slug: String) -> Result<Option<String>, String> {
    if !valid_slug(&slug) {
        return Err("主题标识不合法".to_string());
    }

    let cache = read_cache(&app)?.ok_or_else(|| "主题目录尚未同步".to_string())?;
    let theme = cache
        .themes
        .iter()
        .find(|theme| theme.slug == slug)
        .ok_or_else(|| "主题不存在".to_string())?;

    let dir = cache_dir(&app)?.join("screenshots");
    if let Some(data) = read_cached_screenshot(&dir, &slug) {
        return Ok(Some(data));
    }

    let Some(url) = theme.screenshot_url.as_deref() else {
        return Ok(None);
    };

    let client = http_client()?;
    let response = client
        .get(url)
        .send()
        .map_err(|error| format!("下载截图失败：{error}"))?;
    if !response.status().is_success() {
        return Ok(None);
    }
    let bytes = response
        .bytes()
        .map_err(|error| format!("读取截图失败：{error}"))?;
    if bytes.len() as u64 > MAX_SCREENSHOT_BYTES {
        return Ok(None);
    }

    let mime = screenshot_mime(url);
    std::fs::create_dir_all(&dir).map_err(|error| format!("创建缓存目录失败：{error}"))?;
    std::fs::write(dir.join(format!("{slug}.img")), &bytes)
        .map_err(|error| format!("写入截图缓存失败：{error}"))?;

    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    Ok(Some(format!("data:{mime};base64,{encoded}")))
}

#[derive(Clone, Copy)]
enum SyncMode {
    Full,
    Incremental,
}

struct SyncOutcome {
    themes: usize,
    updated: u32,
    failed: u32,
}

fn run_sync(app: AppHandle, mode: SyncMode) {
    let result = match mode {
        SyncMode::Full => full_sync(&app),
        SyncMode::Incremental => incremental_sync(&app),
    };
    let state = app.state::<CatalogState>();
    state.running.store(false, Ordering::SeqCst);

    match result {
        Ok(outcome) => {
            let message = match mode {
                SyncMode::Full => format!("目录已同步：{} 个 GitHub 主题", outcome.themes),
                SyncMode::Incremental => format!(
                    "检查完成：{} 个主题，本次更新 {} 个",
                    outcome.themes, outcome.updated
                ),
            };
            let progress = SyncProgress {
                phase: "done".to_string(),
                done: outcome.themes as u32,
                total: outcome.themes as u32,
                failed: outcome.failed,
                message,
            };
            if let Ok(mut slot) = state.progress.lock() {
                *slot = progress.clone();
            }
            let _ = app.emit("theme-catalog://progress", progress);
            let _ = app.emit(
                "theme-catalog://done",
                serde_json::json!({
                    "ok": true,
                    "themes": outcome.themes,
                    "updated": outcome.updated,
                    "failed": outcome.failed
                }),
            );
        }
        Err(error) => {
            let progress = SyncProgress {
                phase: "error".to_string(),
                message: error.clone(),
                ..SyncProgress::default()
            };
            if let Ok(mut slot) = state.progress.lock() {
                *slot = progress.clone();
            }
            let _ = app.emit("theme-catalog://progress", progress);
            let _ = app.emit(
                "theme-catalog://done",
                serde_json::json!({ "ok": false, "error": error }),
            );
        }
    }
}

fn full_sync(app: &AppHandle) -> Result<SyncOutcome, String> {
    let client = http_client()?;

    set_progress(app, "catalog", 0, 0, 0, "抓取主题目录…");
    let cards = fetch_catalog_cards(&client)?;
    let versions = fetch_repo_versions(&client).unwrap_or_default();
    let total = cards.len() as u32;
    let mut themes = Vec::new();
    let mut failed = 0u32;

    for (index, card) in cards.iter().enumerate() {
        std::thread::sleep(Duration::from_millis(120));

        match fetch_theme(&client, card, &versions) {
            Ok(theme) => themes.push(theme),
            Err(_) => failed += 1,
        }

        let done = index as u32 + 1;
        set_progress(
            app,
            "details",
            done,
            total,
            failed,
            &format!("抓取主题详情 {done}/{total}"),
        );

        if done.is_multiple_of(25) {
            let _ = write_cache(
                app,
                &CatalogCache {
                    schema: SCHEMA,
                    fetched_at: now_secs(),
                    checked_at: now_secs(),
                    complete: false,
                    source: SITE.to_string(),
                    themes: themes.clone(),
                },
            );
        }
    }

    if themes.is_empty() {
        return Err("没有可用的 GitHub 主题，目录解析可能失败".to_string());
    }

    themes.sort_by_key(|theme| theme.name.to_lowercase());

    write_cache(
        app,
        &CatalogCache {
            schema: SCHEMA,
            fetched_at: now_secs(),
            checked_at: now_secs(),
            complete: true,
            source: SITE.to_string(),
            themes: themes.clone(),
        },
    )?;

    Ok(SyncOutcome {
        themes: themes.len(),
        updated: themes.len() as u32,
        failed,
    })
}

fn incremental_sync(app: &AppHandle) -> Result<SyncOutcome, String> {
    let cache = read_cache(app)?.ok_or_else(|| "主题目录缓存不存在".to_string())?;
    let client = http_client()?;

    set_progress(app, "catalog", 0, 0, 0, "检查主题目录更新…");
    let cards = fetch_catalog_cards(&client)?;
    let versions = fetch_repo_versions(&client)?;

    let mut existing: HashMap<String, CatalogTheme> = cache
        .themes
        .into_iter()
        .map(|theme| (theme.slug.clone(), theme))
        .collect();

    let total = cards.len() as u32;
    let mut themes = Vec::new();
    let mut updated = 0u32;
    let mut failed = 0u32;

    for (index, card) in cards.iter().enumerate() {
        let previous = existing.remove(&card.slug);
        let dirty = previous
            .as_ref()
            .map(|theme| is_dirty(theme, &versions))
            .unwrap_or(true);

        if dirty {
            std::thread::sleep(Duration::from_millis(120));
            match fetch_theme(&client, card, &versions) {
                Ok(theme) => {
                    themes.push(theme);
                    updated += 1;
                }
                Err(_) => {
                    failed += 1;
                    if let Some(theme) = previous {
                        themes.push(theme);
                    }
                }
            }
        } else if let Some(theme) = previous {
            themes.push(theme);
        }

        let done = index as u32 + 1;
        set_progress(
            app,
            "update",
            done,
            total,
            failed,
            &format!("检查主题更新 {done}/{total}，已更新 {updated}"),
        );
    }

    themes.sort_by_key(|theme| theme.name.to_lowercase());

    write_cache(
        app,
        &CatalogCache {
            schema: SCHEMA,
            fetched_at: cache_fetched_at(app).unwrap_or_else(now_secs),
            checked_at: now_secs(),
            complete: true,
            source: SITE.to_string(),
            themes: themes.clone(),
        },
    )?;

    Ok(SyncOutcome {
        themes: themes.len(),
        updated,
        failed,
    })
}

fn cache_fetched_at(app: &AppHandle) -> Option<u64> {
    read_cache(app).ok().flatten().map(|cache| cache.fetched_at)
}

fn fetch_catalog_cards(client: &reqwest::blocking::Client) -> Result<Vec<CatalogCard>, String> {
    let home = client
        .get(SITE)
        .send()
        .map_err(|error| format!("访问 themes.gohugo.io 失败：{error}"))?
        .text()
        .map_err(|error| format!("读取主题目录失败：{error}"))?;

    let cards = parse_catalog(&home);
    if cards.is_empty() {
        return Err("没有从主题目录解析到任何条目，页面结构可能已变化".to_string());
    }
    Ok(cards)
}

fn fetch_theme(
    client: &reqwest::blocking::Client,
    card: &CatalogCard,
    versions: &HashMap<String, String>,
) -> Result<CatalogTheme, String> {
    let detail = fetch_detail(client, &card.slug)?;
    let (owner, repo) = github_repo(detail.repo.as_deref().unwrap_or_default())
        .ok_or_else(|| "主题不是 GitHub 仓库".to_string())?;
    let full_name = format!("{owner}/{repo}");
    let repo_updated_at = versions
        .get(&full_name.to_lowercase())
        .cloned()
        .or_else(|| detail.updated.clone());

    let fallback = format!(
        "git submodule add https://github.com/{owner}/{repo} themes/{}",
        card.slug
    );
    let (commands, extracted, source) = match fetch_readme(client, &owner, &repo) {
        Some((url, markdown)) => {
            let (commands, extracted) = extract_install_commands(&markdown, &fallback);
            (commands, extracted, Some(url))
        }
        None => (
            vec![fallback],
            false,
            Some(format!("https://github.com/{owner}/{repo}")),
        ),
    };

    Ok(CatalogTheme {
        slug: card.slug.clone(),
        name: card.name.clone(),
        authors: detail.authors,
        description: detail.description,
        tags: detail.tags,
        license: detail.license,
        stars: detail.stars,
        updated: detail.updated,
        repo: format!("https://github.com/{owner}/{repo}"),
        demo: detail.demo,
        screenshot_url: detail
            .screenshot_url
            .or_else(|| card.screenshot_url.clone()),
        install_commands: commands,
        install_extracted: extracted,
        install_source: source,
        repo_updated_at,
    })
}

fn fetch_repo_versions(
    client: &reqwest::blocking::Client,
) -> Result<HashMap<String, String>, String> {
    let text = client
        .get(BUILDER_CACHE)
        .send()
        .map_err(|error| format!("读取主题版本数据失败：{error}"))?
        .text()
        .map_err(|error| format!("读取主题版本数据失败：{error}"))?;
    let value = serde_json::from_str::<serde_json::Value>(&text)
        .map_err(|error| format!("解析主题版本数据失败：{error}"))?;
    let object = value
        .as_object()
        .ok_or_else(|| "主题版本数据格式不正确".to_string())?;

    let mut versions = HashMap::new();
    for (path, data) in object {
        let Some(full_name) = module_repo(path) else {
            continue;
        };
        let Some(updated_at) = data.get("updated_at").and_then(|value| value.as_str()) else {
            continue;
        };
        versions.insert(full_name, updated_at.to_string());
    }

    Ok(versions)
}

fn module_repo(path: &str) -> Option<String> {
    let rest = path.strip_prefix("github.com/")?;
    let mut parts = rest.split('/');
    let owner = parts.next()?.trim();
    let repo = parts.next()?.trim();
    if owner.is_empty() || repo.is_empty() {
        return None;
    }
    Some(format!("{owner}/{repo}").to_lowercase())
}

fn is_dirty(theme: &CatalogTheme, versions: &HashMap<String, String>) -> bool {
    let Some(full_name) =
        github_repo(&theme.repo).map(|(owner, repo)| format!("{owner}/{repo}").to_lowercase())
    else {
        return false;
    };

    match versions.get(&full_name) {
        Some(current) => theme.repo_updated_at.as_deref() != Some(current.as_str()),
        None => true,
    }
}

fn set_progress(app: &AppHandle, phase: &str, done: u32, total: u32, failed: u32, message: &str) {
    let progress = SyncProgress {
        phase: phase.to_string(),
        done,
        total,
        failed,
        message: message.to_string(),
    };
    let state = app.state::<CatalogState>();
    if let Ok(mut slot) = state.progress.lock() {
        *slot = progress.clone();
    }
    let _ = app.emit("theme-catalog://progress", progress);
}

fn http_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .user_agent("Hugonic/0.2 (+https://github.com/AllenZhang45/Hugonic)")
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|error| format!("创建 HTTP 客户端失败：{error}"))
}

fn fetch_detail(client: &reqwest::blocking::Client, slug: &str) -> Result<ThemeDetail, String> {
    let url = format!("{SITE}themes/{slug}/");
    let response = client
        .get(&url)
        .send()
        .map_err(|error| format!("请求主题页失败：{error}"))?;
    if !response.status().is_success() {
        return Err(format!("主题页返回 {}", response.status()));
    }
    let html = response
        .text()
        .map_err(|error| format!("读取主题页失败：{error}"))?;

    Ok(parse_detail(&html))
}

fn fetch_readme(
    client: &reqwest::blocking::Client,
    owner: &str,
    repo: &str,
) -> Option<(String, String)> {
    for file in ["README.md", "readme.md", "README.markdown", "Readme.md"] {
        let url = format!("https://raw.githubusercontent.com/{owner}/{repo}/HEAD/{file}");
        let response = client.get(&url).send().ok()?;
        if !response.status().is_success() {
            continue;
        }
        if let Ok(text) = response.text()
            && !text.trim().is_empty()
        {
            return Some((url, text));
        }
    }

    None
}

fn parse_catalog(html: &str) -> Vec<CatalogCard> {
    let document = Html::parse_document(html);
    let item_selector = Selector::parse("li.relative").unwrap();
    let link_selector = Selector::parse("a[href^='/themes/']").unwrap();
    let image_selector = Selector::parse("img").unwrap();
    let text_selector = Selector::parse("p").unwrap();
    let mut seen = HashSet::new();
    let mut cards = Vec::new();

    for item in document.select(&item_selector) {
        let Some(link) = item.select(&link_selector).next() else {
            continue;
        };
        let Some(href) = link.value().attr("href") else {
            continue;
        };
        let slug = href
            .trim_start_matches("/themes/")
            .trim_end_matches('/')
            .to_string();
        if slug.is_empty() || slug.contains('/') || !seen.insert(slug.clone()) {
            continue;
        }

        let name = item
            .select(&text_selector)
            .next()
            .map(|node| node.text().collect::<String>().trim().to_string())
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| slug.clone());

        let screenshot_url = item
            .select(&image_selector)
            .next()
            .and_then(|image| image.value().attr("src"))
            .map(absolute_url);

        cards.push(CatalogCard {
            slug,
            name,
            screenshot_url,
        });
    }

    cards.sort_by_key(|card| card.name.to_lowercase());
    cards
}

fn parse_detail(html: &str) -> ThemeDetail {
    let document = Html::parse_document(html);
    let meta_selector = Selector::parse("meta[name=description]").unwrap();
    let tag_selector = Selector::parse("a[href^='https://themes.gohugo.io/tags/']").unwrap();
    let image_selector = Selector::parse("meta[property='og:image']").unwrap();
    let anchor_selector = Selector::parse("a").unwrap();

    let authors = dd_values(html, "Author");

    let description = document
        .select(&meta_selector)
        .next()
        .and_then(|meta| meta.value().attr("content"))
        .unwrap_or_default()
        .trim()
        .to_string();

    let mut tags = Vec::new();
    for tag in document.select(&tag_selector) {
        let text = tag.text().collect::<String>().trim().to_string();
        if !text.is_empty() && !tags.contains(&text) {
            tags.push(text);
        }
    }

    let mut repo = None;
    let mut demo = None;
    for anchor in document.select(&anchor_selector) {
        let Some(href) = anchor.value().attr("href") else {
            continue;
        };
        let text = anchor.text().collect::<String>().trim().to_lowercase();
        if repo.is_none() && (text == "download" || text == "repository" || text == "source") {
            repo = Some(href.to_string());
        } else if demo.is_none() && text == "demo" {
            demo = Some(href.to_string());
        }
    }

    let screenshot_url = document
        .select(&image_selector)
        .next()
        .and_then(|meta| meta.value().attr("content"))
        .map(str::to_string);

    ThemeDetail {
        description,
        authors,
        tags,
        license: dd_text(html, "License"),
        stars: dd_text(html, "GitHub Stars")
            .and_then(|value| value.replace(',', "").trim().parse::<u64>().ok()),
        updated: dd_text(html, "Last Updated"),
        repo,
        demo,
        screenshot_url,
    }
}

fn dd_text(html: &str, label: &str) -> Option<String> {
    dd_values(html, label).into_iter().next()
}

fn dd_values(html: &str, label: &str) -> Vec<String> {
    let Ok(pattern) = Regex::new(&format!(
        r"(?s)<dt[^>]*>\s*{}\s*</dt>\s*<dd[^>]*>(.*?)</dd>",
        regex::escape(label)
    )) else {
        return Vec::new();
    };
    let Ok(tag_regex) = Regex::new(r"<[^>]+>") else {
        return Vec::new();
    };

    let mut values = Vec::new();
    for captures in pattern.captures_iter(html) {
        let text = tag_regex
            .replace_all(&captures[1], " ")
            .replace("&amp;", "&")
            .replace("&nbsp;", " ");
        let compact = text.split_whitespace().collect::<Vec<_>>().join(" ");
        if !compact.is_empty() && !values.contains(&compact) {
            values.push(compact);
        }
    }
    values
}

fn absolute_url(value: &str) -> String {
    if value.starts_with("http://") || value.starts_with("https://") {
        value.to_string()
    } else if value.starts_with('/') {
        format!("https://themes.gohugo.io{value}")
    } else {
        format!("https://themes.gohugo.io/{value}")
    }
}

pub(crate) fn github_repo(url: &str) -> Option<(String, String)> {
    let trimmed = url
        .trim()
        .trim_end_matches('/')
        .trim_end_matches(".git")
        .split(['?', '#'])
        .next()
        .unwrap_or(url);
    let without_scheme = trimmed
        .strip_prefix("https://")
        .or_else(|| trimmed.strip_prefix("http://"))
        .unwrap_or(trimmed);
    let mut parts = without_scheme.split('/');
    let host = parts.next()?;
    if host != "github.com" {
        return None;
    }
    let owner = parts.next()?.trim();
    let repo = parts.next()?.trim();
    if owner.is_empty() || repo.is_empty() || owner == "gohugoio" && repo == "hugo" {
        return None;
    }
    if parts.next().is_some() {
        return None;
    }

    Some((owner.to_string(), repo.to_string()))
}

pub(crate) fn extract_install_commands(markdown: &str, fallback: &str) -> (Vec<String>, bool) {
    let mut commands = Vec::new();

    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty()
            || trimmed.starts_with('#')
            || trimmed.starts_with("//")
            || trimmed.starts_with("<!--")
        {
            continue;
        }

        let command = trimmed.strip_prefix("$ ").unwrap_or(trimmed);
        let lower = command.to_lowercase();
        let relevant = lower.contains("git submodule add")
            || (lower.contains("git clone") && lower.contains("themes/"))
            || lower.contains("hugo mod get");

        if relevant && !commands.iter().any(|existing| existing == command) {
            commands.push(command.to_string());
        }
    }

    let extracted = !commands.is_empty();
    if !extracted {
        commands.push(fallback.to_string());
    }
    commands.truncate(MAX_COMMANDS);
    (commands, extracted)
}

fn cache_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_cache_dir()
        .map_err(|error| format!("无法定位应用缓存目录：{error}"))?
        .join("theme-catalog");
    std::fs::create_dir_all(&dir).map_err(|error| format!("创建缓存目录失败：{error}"))?;
    Ok(dir)
}

fn read_cache(app: &AppHandle) -> Result<Option<CatalogCache>, String> {
    let path = cache_dir(app)?.join("catalog.json");
    if !path.is_file() {
        return Ok(None);
    }
    let text =
        std::fs::read_to_string(&path).map_err(|error| format!("读取目录缓存失败：{error}"))?;
    let cache = serde_json::from_str::<CatalogCache>(&text)
        .map_err(|error| format!("解析目录缓存失败：{error}"))?;
    if cache.schema != SCHEMA {
        return Ok(None);
    }
    Ok(Some(cache))
}

fn write_cache(app: &AppHandle, cache: &CatalogCache) -> Result<(), String> {
    let path = cache_dir(app)?.join("catalog.json");
    let text = serde_json::to_string_pretty(cache)
        .map_err(|error| format!("序列化目录缓存失败：{error}"))?;
    std::fs::write(&path, text).map_err(|error| format!("写入目录缓存失败：{error}"))
}

fn read_cached_screenshot(dir: &std::path::Path, slug: &str) -> Option<String> {
    let path = dir.join(format!("{slug}.img"));
    let bytes = std::fs::read(&path).ok()?;
    let mime = detect_image_mime(&bytes);
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    Some(format!("data:{mime};base64,{encoded}"))
}

fn detect_image_mime(bytes: &[u8]) -> &'static str {
    if bytes.starts_with(b"\x89PNG") {
        "image/png"
    } else if bytes.starts_with(b"\xff\xd8") {
        "image/jpeg"
    } else if bytes.starts_with(b"GIF8") {
        "image/gif"
    } else if bytes.starts_with(b"RIFF") {
        "image/webp"
    } else {
        "image/png"
    }
}

fn screenshot_mime(url: &str) -> &'static str {
    let lower = url.to_lowercase();
    if lower.ends_with(".png") {
        "image/png"
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "image/jpeg"
    } else if lower.ends_with(".webp") {
        "image/webp"
    } else if lower.ends_with(".gif") {
        "image/gif"
    } else {
        "image/png"
    }
}

fn valid_slug(slug: &str) -> bool {
    !slug.is_empty()
        && slug.chars().all(|character| {
            character.is_ascii_alphanumeric()
                || character == '-'
                || character == '_'
                || character == '.'
        })
        && !slug.contains("..")
}

fn is_fresh(fetched_at: u64) -> bool {
    now_secs().saturating_sub(fetched_at) < CATALOG_TTL_SECS
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOME: &str = r#"
    <ul>
      <li class=relative><div><picture><img src=/themes/ananke/tn.png></picture>
        <a href=/themes/ananke/><span class=sr-only>View details for Ananke</span></a></div>
        <p>Ananke Theme</p></li>
      <li class=relative><div><picture><img src=/themes/paper/tn.png></picture>
        <a href=/themes/hugo-paper/></a></div><p>Paper</p></li>
    </ul>
    "#;

    const DETAIL: &str = r#"
    <html><head><meta name=description content="A base theme"></head><body>
      <dl>
        <dt>License</dt><dd>MIT</dd>
        <dt>Author</dt><dd>Patrick Kollitsch</dd>
        <dt>Author</dt><dd>Bud Parr</dd>
        <dt>GitHub Stars</dt><dd>1,397</dd>
        <dt>Last Updated</dt><dd>2026-09-26</dd>
        <dt>Tags</dt><dd>
          <a href=https://themes.gohugo.io/tags/blog/>blog</a>
          <a href=https://themes.gohugo.io/tags/multilingual/>multilingual</a>
        </dd>
      </dl>
      <a href=https://github.com/gohugo-ananke/ananke rel=nofollow>Download</a>
      <a href=https://ananke-theme.netlify.app/ rel=nofollow>Demo</a>
      <meta property="og:image" content="https://themes.gohugo.io/themes/ananke/tn-featured.png">
    </body></html>
    "#;

    #[test]
    fn parses_catalog_cards() {
        let cards = parse_catalog(HOME);

        assert_eq!(cards.len(), 2);
        assert_eq!(cards[0].slug, "ananke");
        assert_eq!(cards[0].name, "Ananke Theme");
        assert_eq!(
            cards[0].screenshot_url.as_deref(),
            Some("https://themes.gohugo.io/themes/ananke/tn.png")
        );
        assert_eq!(cards[1].slug, "hugo-paper");
    }

    #[test]
    fn parses_detail_fields() {
        let detail = parse_detail(DETAIL);

        assert_eq!(detail.description, "A base theme");
        assert_eq!(detail.authors, vec!["Patrick Kollitsch", "Bud Parr"]);
        assert_eq!(detail.license.as_deref(), Some("MIT"));
        assert_eq!(detail.stars, Some(1397));
        assert_eq!(detail.updated.as_deref(), Some("2026-09-26"));
        assert_eq!(detail.tags, vec!["blog", "multilingual"]);
        assert_eq!(
            detail.repo.as_deref(),
            Some("https://github.com/gohugo-ananke/ananke")
        );
        assert_eq!(
            detail.demo.as_deref(),
            Some("https://ananke-theme.netlify.app/")
        );
    }

    #[test]
    fn extracts_install_commands_from_readme() {
        let readme = "Install:\n\n```bash\n$ git submodule add https://github.com/acme/theme themes/acme\n```\n\n```bash\ngit clone https://github.com/acme/starter my-site\n```\n";

        let (commands, extracted) =
            extract_install_commands(readme, "git submodule add fallback themes/x");

        assert!(extracted);
        assert_eq!(
            commands,
            vec!["git submodule add https://github.com/acme/theme themes/acme"]
        );
    }

    #[test]
    fn falls_back_when_readme_has_no_commands() {
        let (commands, extracted) = extract_install_commands(
            "See the documentation for install instructions.",
            "git submodule add https://github.com/acme/theme themes/acme",
        );

        assert!(!extracted);
        assert_eq!(
            commands,
            vec!["git submodule add https://github.com/acme/theme themes/acme"]
        );
    }

    #[test]
    fn accepts_only_github_repositories() {
        assert_eq!(
            github_repo("https://github.com/acme/theme.git"),
            Some(("acme".to_string(), "theme".to_string()))
        );
        assert_eq!(github_repo("https://codeberg.org/acme/theme"), None);
        assert_eq!(github_repo("https://github.com/gohugoio/hugo"), None);
    }

    #[test]
    fn ttl_is_seven_days() {
        let now = now_secs();
        assert!(is_fresh(now - 100));
        assert!(!is_fresh(now - CATALOG_TTL_SECS - 10));
    }
    fn theme(repo_updated_at: Option<&str>) -> CatalogTheme {
        CatalogTheme {
            slug: "ananke".to_string(),
            name: "Ananke".to_string(),
            authors: Vec::new(),
            description: String::new(),
            tags: Vec::new(),
            license: None,
            stars: None,
            updated: None,
            repo: "https://github.com/acme/theme".to_string(),
            demo: None,
            screenshot_url: None,
            install_commands: Vec::new(),
            install_extracted: false,
            install_source: None,
            repo_updated_at: repo_updated_at.map(str::to_string),
        }
    }

    #[test]
    fn detects_theme_version_changes() {
        let mut versions = HashMap::new();
        versions.insert("acme/theme".to_string(), "2026-10-01T00:00:00Z".to_string());

        assert!(!is_dirty(&theme(Some("2026-10-01T00:00:00Z")), &versions));
        assert!(is_dirty(&theme(Some("2026-09-01T00:00:00Z")), &versions));
        assert!(is_dirty(&theme(None), &versions));
    }

    #[test]
    fn extracts_repo_names_from_builder_keys() {
        assert_eq!(
            module_repo("github.com/acme/theme/v2"),
            Some("acme/theme".to_string())
        );
        assert_eq!(module_repo("codeberg.org/acme/theme"), None);
    }
}
