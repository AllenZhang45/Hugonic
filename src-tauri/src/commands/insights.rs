use std::collections::HashMap;
use std::path::Path;
use std::time::UNIX_EPOCH;

use serde::Serialize;
use tauri::State;

use crate::state::AppState;

const IMAGE_EXTENSIONS: [&str; 8] = ["jpg", "jpeg", "png", "webp", "gif", "svg", "avif", "bmp"];
const MAX_CONTENT_BYTES: u64 = 2 * 1024 * 1024;
const RECENT_LIMIT: usize = 5;
const FACET_LIMIT: usize = 10;

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Insights {
    pub config_file: String,
    pub title: Option<String>,
    pub base_url: Option<String>,
    pub theme: Option<String>,
    pub language: Option<String>,
    pub counts: Counts,
    pub words: u64,
    pub sections: Vec<SectionStat>,
    pub tags: Vec<FacetStat>,
    pub categories: Vec<FacetStat>,
    pub recent: Vec<RecentFile>,
    pub missing_title: u64,
    pub missing_date: u64,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Counts {
    pub pages: u64,
    pub posts: u64,
    pub drafts: u64,
    pub media: u64,
    pub themes: u64,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectionStat {
    pub name: String,
    pub pages: u64,
    pub drafts: u64,
    pub words: u64,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct FacetStat {
    pub name: String,
    pub count: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentFile {
    pub path: String,
    pub name: String,
    pub modified_ms: u64,
}

#[derive(Debug, Default)]
struct ConfigSummary {
    title: Option<String>,
    base_url: Option<String>,
    theme: Option<String>,
    language: Option<String>,
}

#[derive(Debug, Default)]
struct FrontMatter {
    title: Option<String>,
    has_date: bool,
    draft: bool,
    tags: Vec<String>,
    categories: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FrontMatterFormat {
    Toml,
    Yaml,
}

#[tauri::command]
pub fn project_insights(state: State<AppState>) -> Result<Insights, String> {
    let root = state
        .root
        .lock()
        .map_err(|_| "项目状态不可用".to_string())?
        .clone()
        .ok_or_else(|| "尚未打开项目".to_string())?;

    build_insights(&root)
}

pub fn build_insights(root: &Path) -> Result<Insights, String> {
    let (config_file, config) = read_config(root);
    let content_dir = root.join("content");
    let mut insights = Insights {
        config_file,
        title: config.title,
        base_url: config.base_url,
        theme: config.theme,
        language: config.language,
        ..Insights::default()
    };

    let mut sections: HashMap<String, SectionStat> = HashMap::new();
    let mut tags: HashMap<String, u64> = HashMap::new();
    let mut categories: HashMap<String, u64> = HashMap::new();
    let mut recent: Vec<(u64, RecentFile)> = Vec::new();

    if content_dir.is_dir() {
        visit_content(
            root,
            &content_dir,
            &mut insights,
            &mut sections,
            &mut tags,
            &mut categories,
            &mut recent,
        )?;
    }

    insights.counts.media = count_media(root);
    insights.counts.themes = count_themes(root);

    let mut section_list: Vec<SectionStat> = sections.into_values().collect();
    section_list.sort_by(|a, b| a.name.cmp(&b.name));
    insights.sections = section_list;

    insights.tags = top_facets(tags);
    insights.categories = top_facets(categories);

    recent.sort_by_key(|item| std::cmp::Reverse(item.0));
    insights.recent = recent
        .into_iter()
        .take(RECENT_LIMIT)
        .map(|(_, file)| file)
        .collect();

    Ok(insights)
}

fn visit_content(
    root: &Path,
    dir: &Path,
    insights: &mut Insights,
    sections: &mut HashMap<String, SectionStat>,
    tags: &mut HashMap<String, u64>,
    categories: &mut HashMap<String, u64>,
    recent: &mut Vec<(u64, RecentFile)>,
) -> Result<(), String> {
    let entries = std::fs::read_dir(dir).map_err(|error| format!("读取 content 失败：{error}"))?;

    for entry in entries {
        let entry = entry.map_err(|error| format!("读取目录项失败：{error}"))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|error| format!("读取文件类型失败：{error}"))?;

        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            visit_content(root, &path, insights, sections, tags, categories, recent)?;
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }

        let metadata = entry
            .metadata()
            .map_err(|error| format!("读取文件信息失败：{error}"))?;
        let modified_ms = metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_millis() as u64)
            .unwrap_or(0);

        let relative = path
            .strip_prefix(root)
            .map_err(|_| "文件不在项目内".to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| relative.clone());
        let section = section_of(root, &path);

        insights.counts.pages += 1;
        if section == "posts" {
            insights.counts.posts += 1;
        }

        if metadata.len() <= MAX_CONTENT_BYTES
            && let Ok(text) = std::fs::read_to_string(&path)
        {
            let (front_matter, body) = parse_document(&text);
            if front_matter.draft {
                insights.counts.drafts += 1;
            }
            if front_matter.title.is_none() {
                insights.missing_title += 1;
            }
            if !front_matter.has_date {
                insights.missing_date += 1;
            }

            let words = count_words(&strip_code_blocks(body));
            insights.words += words;

            let stat = sections
                .entry(section.clone())
                .or_insert_with(|| SectionStat {
                    name: section.clone(),
                    ..SectionStat::default()
                });
            stat.pages += 1;
            stat.words += words;
            if front_matter.draft {
                stat.drafts += 1;
            }

            for tag in &front_matter.tags {
                *tags.entry(tag.clone()).or_insert(0) += 1;
            }
            for category in &front_matter.categories {
                *categories.entry(category.clone()).or_insert(0) += 1;
            }
        }

        recent.push((
            modified_ms,
            RecentFile {
                path: relative,
                name,
                modified_ms,
            },
        ));
    }

    Ok(())
}

fn section_of(root: &Path, path: &Path) -> String {
    let content = root.join("content");
    let Ok(relative) = path.strip_prefix(&content) else {
        return "root".to_string();
    };

    match relative
        .parent()
        .and_then(|parent| parent.components().next())
    {
        Some(component) => component.as_os_str().to_string_lossy().into_owned(),
        None => "root".to_string(),
    }
}

fn read_config(root: &Path) -> (String, ConfigSummary) {
    for name in ["hugo.toml", "config.toml", "config.yaml", "config.yml"] {
        let path = root.join(name);
        if !path.is_file() {
            continue;
        }

        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };

        let summary = if name.ends_with(".yaml") || name.ends_with(".yml") {
            parse_yaml_config(&text)
        } else {
            parse_toml_config(&text)
        };

        return (name.to_string(), summary);
    }

    (String::new(), ConfigSummary::default())
}

fn parse_toml_config(text: &str) -> ConfigSummary {
    let Ok(doc) = text.parse::<toml_edit::DocumentMut>() else {
        return ConfigSummary::default();
    };

    ConfigSummary {
        title: doc
            .get("title")
            .and_then(|item| item.as_str())
            .map(str::to_string),
        base_url: doc
            .get("baseURL")
            .and_then(|item| item.as_str())
            .map(str::to_string),
        theme: doc.get("theme").and_then(toml_theme),
        language: doc
            .get("languageCode")
            .or_else(|| doc.get("language"))
            .and_then(|item| item.as_str())
            .map(str::to_string),
    }
}

fn toml_theme(item: &toml_edit::Item) -> Option<String> {
    if let Some(theme) = item.as_str() {
        return Some(theme.to_string());
    }

    item.as_array()
        .and_then(|array| array.iter().find_map(|value| value.as_str()))
        .map(str::to_string)
}

fn parse_yaml_config(text: &str) -> ConfigSummary {
    let Ok(value) = serde_yaml::from_str::<serde_yaml::Value>(text) else {
        return ConfigSummary::default();
    };

    let string_at = |key: &str| {
        value
            .get(key)
            .and_then(|item| item.as_str())
            .map(str::to_string)
    };

    ConfigSummary {
        title: string_at("title"),
        base_url: string_at("baseURL"),
        theme: value.get("theme").and_then(|item| {
            item.as_str().map(str::to_string).or_else(|| {
                item.as_sequence()
                    .and_then(|sequence| sequence.first())
                    .and_then(|first| first.as_str())
                    .map(str::to_string)
            })
        }),
        language: string_at("languageCode").or_else(|| string_at("language")),
    }
}

fn parse_document(text: &str) -> (FrontMatter, &str) {
    if let Some((format, front_matter, body)) = split_front_matter(text) {
        let data = match format {
            FrontMatterFormat::Toml => parse_toml_front_matter(front_matter),
            FrontMatterFormat::Yaml => parse_yaml_front_matter(front_matter),
        };
        (data, body)
    } else {
        (FrontMatter::default(), text)
    }
}

fn split_front_matter(text: &str) -> Option<(FrontMatterFormat, &str, &str)> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);

    for (marker, format) in [
        ("+++", FrontMatterFormat::Toml),
        ("---", FrontMatterFormat::Yaml),
    ] {
        let Some(rest) = text.strip_prefix(marker) else {
            continue;
        };
        let rest = rest
            .strip_prefix("\r\n")
            .or_else(|| rest.strip_prefix('\n'))?;
        let end = rest.find(&format!("\n{marker}"))?;
        let front_matter = &rest[..end];
        let after = &rest[end + marker.len() + 1..];
        let body = after
            .strip_prefix("\r\n")
            .or_else(|| after.strip_prefix('\n'))
            .unwrap_or(after);
        return Some((format, front_matter, body));
    }

    None
}

fn parse_toml_front_matter(text: &str) -> FrontMatter {
    let Ok(doc) = text.parse::<toml_edit::DocumentMut>() else {
        return FrontMatter::default();
    };

    FrontMatter {
        title: doc
            .get("title")
            .and_then(|item| item.as_str())
            .map(str::to_string),
        has_date: doc.contains_key("date"),
        draft: doc
            .get("draft")
            .and_then(|item| item.as_bool())
            .unwrap_or(false),
        tags: doc.get("tags").map(toml_string_list).unwrap_or_default(),
        categories: doc
            .get("categories")
            .map(toml_string_list)
            .unwrap_or_default(),
    }
}

fn toml_string_list(item: &toml_edit::Item) -> Vec<String> {
    if let Some(value) = item.as_str() {
        return vec![value.to_string()];
    }

    item.as_array()
        .map(|array| {
            array
                .iter()
                .filter_map(|value| value.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn parse_yaml_front_matter(text: &str) -> FrontMatter {
    let Ok(value) = serde_yaml::from_str::<serde_yaml::Value>(text) else {
        return FrontMatter::default();
    };

    FrontMatter {
        title: value
            .get("title")
            .and_then(|item| item.as_str())
            .map(str::to_string),
        has_date: value.get("date").is_some(),
        draft: value
            .get("draft")
            .and_then(|item| item.as_bool())
            .unwrap_or(false),
        tags: yaml_string_list(value.get("tags")),
        categories: yaml_string_list(value.get("categories")),
    }
}

fn yaml_string_list(value: Option<&serde_yaml::Value>) -> Vec<String> {
    let Some(value) = value else {
        return Vec::new();
    };

    if let Some(item) = value.as_str() {
        return vec![item.to_string()];
    }

    value
        .as_sequence()
        .map(|sequence| {
            sequence
                .iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn strip_code_blocks(body: &str) -> String {
    let mut output = String::new();
    let mut in_code = false;

    for line in body.lines() {
        if line.trim_start().starts_with("```") {
            in_code = !in_code;
            continue;
        }
        if !in_code {
            output.push_str(line);
            output.push('\n');
        }
    }

    output
}

fn count_words(text: &str) -> u64 {
    let mut words = 0;
    let mut in_latin_word = false;

    for character in text.chars() {
        if is_cjk(character) {
            words += 1;
            in_latin_word = false;
        } else if character.is_alphanumeric() {
            if !in_latin_word {
                words += 1;
                in_latin_word = true;
            }
        } else {
            in_latin_word = false;
        }
    }

    words
}

fn is_cjk(character: char) -> bool {
    matches!(
        character as u32,
        0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF
    )
}

fn top_facets(counts: HashMap<String, u64>) -> Vec<FacetStat> {
    let mut facets: Vec<FacetStat> = counts
        .into_iter()
        .map(|(name, count)| FacetStat { name, count })
        .collect();
    facets.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.name.cmp(&b.name)));
    facets.truncate(FACET_LIMIT);
    facets
}

fn count_media(root: &Path) -> u64 {
    let mut count = 0;
    for dir in [root.join("static/images"), root.join("assets")] {
        count_media_dir(&dir, &mut count);
    }
    count
}

fn count_media_dir(dir: &Path, count: &mut u64) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            count_media_dir(&path, count);
            continue;
        }
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str())
            && IMAGE_EXTENSIONS.contains(&extension.to_lowercase().as_str())
        {
            *count += 1;
        }
    }
}

fn count_themes(root: &Path) -> u64 {
    let themes = root.join("themes");
    let Ok(entries) = std::fs::read_dir(themes) else {
        return 0;
    };

    entries
        .flatten()
        .filter(|entry| entry.path().is_dir() && entry.path().join("theme.toml").is_file())
        .count() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn project() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        fs::create_dir_all(root.join("content/posts")).unwrap();
        fs::create_dir_all(root.join("static/images")).unwrap();
        fs::create_dir_all(root.join("themes/ananke")).unwrap();
        fs::write(root.join("static/images/cover.png"), b"png").unwrap();
        fs::write(root.join("themes/ananke/theme.toml"), "").unwrap();
        fs::write(
            root.join("hugo.toml"),
            "baseURL = 'https://example.org/'\ntitle = 'Demo'\ntheme = 'ananke'\nlanguageCode = 'zh-cn'\n",
        )
        .unwrap();
        fs::write(
            root.join("content/posts/first.md"),
            "+++\ntitle = '第一篇'\ndate = 2026-10-01\ndraft = true\ntags = ['hugo', 'test']\ncategories = ['技术']\n+++\n\n你好 world\n",
        )
        .unwrap();
        fs::write(
            root.join("content/about.md"),
            "---\ntitle: About\ndate: 2026-10-02\ntags:\n  - hugo\n---\n\nAbout **me**\n",
        )
        .unwrap();
        fs::write(root.join("content/empty.md"), "no front matter\n").unwrap();
        dir
    }

    #[test]
    fn builds_counts_and_metadata() {
        let dir = project();
        let insights = build_insights(dir.path()).unwrap();

        assert_eq!(insights.config_file, "hugo.toml");
        assert_eq!(insights.base_url.as_deref(), Some("https://example.org/"));
        assert_eq!(insights.title.as_deref(), Some("Demo"));
        assert_eq!(insights.theme.as_deref(), Some("ananke"));
        assert_eq!(insights.language.as_deref(), Some("zh-cn"));
        assert_eq!(insights.counts.pages, 3);
        assert_eq!(insights.counts.posts, 1);
        assert_eq!(insights.counts.drafts, 1);
        assert_eq!(insights.counts.media, 1);
        assert_eq!(insights.counts.themes, 1);
        assert_eq!(insights.missing_title, 1);
        assert_eq!(insights.missing_date, 1);
        assert_eq!(insights.words, 3 + 2 + 3);
    }

    #[test]
    fn groups_sections_and_facets() {
        let dir = project();
        let insights = build_insights(dir.path()).unwrap();

        let sections: Vec<(&str, u64, u64)> = insights
            .sections
            .iter()
            .map(|section| (section.name.as_str(), section.pages, section.drafts))
            .collect();
        assert_eq!(sections, vec![("posts", 1, 1), ("root", 2, 0)]);

        let tags: Vec<(&str, u64)> = insights
            .tags
            .iter()
            .map(|tag| (tag.name.as_str(), tag.count))
            .collect();
        assert_eq!(tags, vec![("hugo", 2), ("test", 1)]);
        assert_eq!(insights.categories[0].name, "技术");
    }

    #[test]
    fn parses_yaml_config_and_theme_array() {
        let yaml = parse_yaml_config(
            "title: Demo\nbaseURL: https://x.test/\ntheme:\n  - ananke\nlanguage: zh\n",
        );
        assert_eq!(yaml.title.as_deref(), Some("Demo"));
        assert_eq!(yaml.theme.as_deref(), Some("ananke"));
        assert_eq!(yaml.language.as_deref(), Some("zh"));
    }

    #[test]
    fn counts_latin_words_and_cjk_characters() {
        assert_eq!(count_words("你好 world 123"), 4);
        assert_eq!(count_words("hello-world"), 2);
    }

    #[test]
    fn strips_fenced_code_blocks_before_counting() {
        let text = "before\n```rust\nlet x = 1;\n```\nafter";
        assert_eq!(count_words(&strip_code_blocks(text)), 2);
    }
}
