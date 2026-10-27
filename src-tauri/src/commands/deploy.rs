use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::state::{AppState, DeployState};

const SERVICE: &str = "hugonic";
const ACCOUNT: &str = "netlify-token";
const API_BASE: &str = "https://api.netlify.com/api/v1";
const EVENT_LOG: &str = "deploy://log";
const EVENT_DONE: &str = "deploy://done";
const POLL_INTERVAL: Duration = Duration::from_secs(2);
const POLL_ATTEMPTS: u32 = 90;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialStatus {
    pub available: bool,
    pub has_token: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetlifyAccount {
    pub name: String,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct Site {
    id: String,
    name: String,
    ssl_url: String,
}

#[derive(Debug, Clone, Deserialize)]
struct Deploy {
    id: String,
    state: String,
    #[serde(default)]
    ssl_url: Option<String>,
    #[serde(default)]
    deploy_ssl_url: Option<String>,
    #[serde(default)]
    error_message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UserResponse {
    #[serde(default)]
    full_name: Option<String>,
    #[serde(default)]
    email: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DeployLog {
    line: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DeployDone {
    ok: bool,
    url: Option<String>,
    site_name: String,
    deploy_id: Option<String>,
    error: Option<String>,
}

#[derive(Debug)]
struct DeployOutcome {
    url: String,
    deploy_id: String,
}

// ---------------------------------------------------------------------------
// 凭据
// ---------------------------------------------------------------------------

fn entry() -> Result<keyring::Entry, String> {
    keyring::Entry::new(SERVICE, ACCOUNT).map_err(|error| format!("无法访问系统钥匙串：{error}"))
}

fn load_token() -> Result<String, String> {
    if !keyring::Entry::store_status().is_ok() {
        return Err("系统钥匙串不可用，无法读取 Netlify token".to_string());
    }

    match entry()?.get_password() {
        Ok(token) if !token.trim().is_empty() => Ok(token),
        Ok(_) => Err("尚未配置 Netlify token".to_string()),
        Err(keyring::Error::NoEntry) => Err("尚未配置 Netlify token".to_string()),
        Err(error) => Err(format!("读取 Netlify token 失败：{error}")),
    }
}

fn save_token(token: &str) -> Result<(), String> {
    let token = token.trim();
    if token.is_empty() {
        return Err("token 不能为空".to_string());
    }
    entry()?
        .set_password(token)
        .map_err(|error| format!("写入系统钥匙串失败：{error}"))
}

fn clear_token() -> Result<(), String> {
    match entry()?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(error) => Err(format!("删除 token 失败：{error}")),
    }
}

#[tauri::command]
pub fn netlify_credential_status() -> CredentialStatus {
    let available = keyring::Entry::store_status().is_ok();
    let has_token = if available {
        match entry().and_then(|entry| entry.get_password().map_err(|error| error.to_string())) {
            Ok(token) => !token.trim().is_empty(),
            Err(_) => false,
        }
    } else {
        false
    };

    CredentialStatus {
        available,
        has_token,
    }
}

#[tauri::command]
pub async fn save_netlify_token(token: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || save_token(&token))
        .await
        .map_err(|error| format!("保存 token 任务失败：{error}"))?
}

#[tauri::command]
pub async fn clear_netlify_token() -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(clear_token)
        .await
        .map_err(|error| format!("删除 token 任务失败：{error}"))?
}

#[tauri::command]
pub async fn test_netlify_connection() -> Result<NetlifyAccount, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let token = load_token()?;
        Netlify::new(token)?.whoami()
    })
    .await
    .map_err(|error| format!("测试连接任务失败：{error}"))?
}

// ---------------------------------------------------------------------------
// Netlify 客户端
// ---------------------------------------------------------------------------

struct Netlify {
    client: reqwest::blocking::Client,
    token: String,
}

impl Netlify {
    fn new(token: String) -> Result<Self, String> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .map_err(|error| format!("创建 HTTP 客户端失败：{error}"))?;
        Ok(Self { client, token })
    }

    fn get_json<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T, String> {
        let response = self
            .client
            .get(format!("{API_BASE}{path}"))
            .bearer_auth(&self.token)
            .header("Accept", "application/json")
            .send()
            .map_err(|error| format!("连接 Netlify 失败：{error}"))?;

        if !response.status().is_success() {
            return Err(map_response_error("请求 Netlify", response));
        }

        response
            .json::<T>()
            .map_err(|error| format!("解析 Netlify 响应失败：{error}"))
    }

    fn whoami(&self) -> Result<NetlifyAccount, String> {
        let user: UserResponse = self.get_json("/user")?;
        let name = user
            .full_name
            .filter(|name| !name.trim().is_empty())
            .or_else(|| user.email.clone())
            .unwrap_or_else(|| "Netlify 账号".to_string());

        Ok(NetlifyAccount {
            name,
            email: user.email,
        })
    }

    fn find_site(&self, name: &str) -> Result<Option<Site>, String> {
        let sites: Vec<Site> = self.get_json(&format!("/sites?name={name}"))?;
        Ok(sites.into_iter().find(|site| site.name == name))
    }

    fn create_site(&self, name: &str) -> Result<Site, String> {
        let response = self
            .client
            .post(format!("{API_BASE}/sites"))
            .bearer_auth(&self.token)
            .json(&serde_json::json!({ "name": name }))
            .send()
            .map_err(|error| format!("创建 Netlify 站点失败：{error}"))?;

        if !response.status().is_success() {
            return Err(map_response_error("创建站点", response));
        }

        response
            .json::<Site>()
            .map_err(|error| format!("解析站点响应失败：{error}"))
    }

    fn upload_deploy(&self, site_id: &str, zip: Vec<u8>) -> Result<Deploy, String> {
        let response = self
            .client
            .post(format!("{API_BASE}/sites/{site_id}/deploys"))
            .bearer_auth(&self.token)
            .header("Content-Type", "application/zip")
            .body(zip)
            .send()
            .map_err(|error| format!("上传部署包失败：{error}"))?;

        if !response.status().is_success() {
            return Err(map_response_error("上传部署包", response));
        }

        response
            .json::<Deploy>()
            .map_err(|error| format!("解析部署响应失败：{error}"))
    }

    fn get_deploy(&self, deploy_id: &str) -> Result<Deploy, String> {
        self.get_json(&format!("/deploys/{deploy_id}"))
    }
}

fn map_response_error(context: &str, response: reqwest::blocking::Response) -> String {
    let status = response.status();
    let body = response.text().unwrap_or_default();
    let detail = body.trim();
    let detail: String = detail.chars().take(300).collect();

    match status.as_u16() {
        401 | 403 => "Netlify token 无效或权限不足，请重新配置".to_string(),
        404 => format!("{context}失败：资源不存在（HTTP 404）"),
        422 => format!("{context}失败：请求被 Netlify 拒绝，站点名可能已被占用"),
        429 => format!("{context}失败：请求过于频繁或超出配额，请稍后重试"),
        _ if detail.is_empty() => format!("{context}失败（HTTP {status}）"),
        _ => format!("{context}失败（HTTP {status}）：{detail}"),
    }
}

// ---------------------------------------------------------------------------
// 部署流程
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn start_deploy(
    app: AppHandle,
    state: State<AppState>,
    deploy: State<DeployState>,
    site_name: String,
) -> Result<(), String> {
    let root = state
        .root
        .lock()
        .map_err(|_| "项目状态不可用".to_string())?
        .clone()
        .ok_or_else(|| "尚未打开项目".to_string())?;

    let name = normalize_site_name(&site_name)?;

    if deploy.running.swap(true, Ordering::SeqCst) {
        return Err("已有部署正在进行".to_string());
    }

    let handle = app.clone();
    std::thread::spawn(move || {
        let outcome = run_deploy(&handle, &root, &name);

        let payload = match &outcome {
            Ok(result) => DeployDone {
                ok: true,
                url: Some(result.url.clone()),
                site_name: name.clone(),
                deploy_id: Some(result.deploy_id.clone()),
                error: None,
            },
            Err(error) => DeployDone {
                ok: false,
                url: None,
                site_name: name.clone(),
                deploy_id: None,
                error: Some(error.clone()),
            },
        };

        let _ = handle.emit(EVENT_DONE, payload);

        if let Some(state) = handle.try_state::<DeployState>() {
            state.running.store(false, Ordering::SeqCst);
        }
    });

    Ok(())
}

fn run_deploy(app: &AppHandle, root: &Path, site_name: &str) -> Result<DeployOutcome, String> {
    emit_log(app, "读取 Netlify token…");
    let token = load_token()?;
    let netlify = Netlify::new(token)?;

    emit_log(app, "校验 Netlify 凭据…");
    let account = netlify.whoami()?;
    emit_log(app, format!("已登录：{}", account.name));

    emit_log(app, format!("查找站点 {site_name}…"));
    let site = match netlify.find_site(site_name)? {
        Some(site) => {
            emit_log(app, format!("复用已有站点：{}", site.ssl_url));
            site
        }
        None => {
            emit_log(app, "站点不存在，正在创建…");
            let site = netlify.create_site(site_name)?;
            emit_log(app, format!("站点已创建：{}", site.ssl_url));
            site
        }
    };

    let base_url = ensure_trailing_slash(&site.ssl_url);
    let output = TempOutput::new()?;

    emit_log(app, format!("构建站点（baseURL = {base_url}）…"));
    build_for_deploy(app, root, &output.0, &base_url)?;

    emit_log(app, "打包 public/…");
    let zip = zip_directory(&output.0.join("public"))?;
    emit_log(
        app,
        format!("部署包大小：{:.1} MB", zip.len() as f64 / 1024.0 / 1024.0),
    );

    emit_log(app, "上传到 Netlify…");
    let mut deploy = netlify.upload_deploy(&site.id, zip)?;
    emit_log(app, format!("部署已提交：{}", deploy.id));

    let mut last_state = deploy.state.clone();
    for _ in 0..POLL_ATTEMPTS {
        if deploy.state == "ready" {
            break;
        }
        if deploy.state == "error" || deploy.state == "rejected" {
            let detail = deploy
                .error_message
                .clone()
                .unwrap_or_else(|| "Netlify 未返回具体原因".to_string());
            return Err(format!("部署失败：{detail}"));
        }

        std::thread::sleep(POLL_INTERVAL);
        deploy = netlify.get_deploy(&deploy.id)?;

        if deploy.state != last_state {
            emit_log(app, format!("部署状态：{}", deploy.state));
            last_state = deploy.state.clone();
        }
    }

    if deploy.state != "ready" {
        return Err(format!("等待部署完成超时（当前状态：{}）", deploy.state));
    }

    let url = deploy
        .ssl_url
        .or(deploy.deploy_ssl_url)
        .unwrap_or_else(|| site.ssl_url.clone());

    emit_log(app, format!("部署完成：{url}"));

    Ok(DeployOutcome {
        url,
        deploy_id: deploy.id,
    })
}

fn build_for_deploy(
    app: &AppHandle,
    root: &Path,
    destination: &Path,
    base_url: &str,
) -> Result<(), String> {
    let binary = crate::commands::preview::sidecar_path()?;
    let output = std::process::Command::new(binary)
        .current_dir(root)
        .arg("--minify")
        .arg("--baseURL")
        .arg(base_url)
        .arg("--destination")
        .arg(destination)
        .output()
        .map_err(|error| format!("启动 hugo 构建失败：{error}"))?;

    emit_stream(app, &output.stdout);
    emit_stream(app, &output.stderr);

    if !output.status.success() {
        return Err(format!(
            "构建失败（退出码 {}），请查看日志",
            output.status.code().unwrap_or(-1)
        ));
    }

    Ok(())
}

fn emit_stream(app: &AppHandle, bytes: &[u8]) {
    for line in String::from_utf8_lossy(bytes).lines() {
        if !line.trim().is_empty() {
            emit_log(app, line.to_string());
        }
    }
}

fn emit_log(app: &AppHandle, line: impl Into<String>) {
    let _ = app.emit(EVENT_LOG, DeployLog { line: line.into() });
}

fn ensure_trailing_slash(url: &str) -> String {
    if url.ends_with('/') {
        url.to_string()
    } else {
        format!("{url}/")
    }
}

/// 把用户输入规范成 Netlify 可用的站点名：小写、连字符、最长 50。
pub fn normalize_site_name(input: &str) -> Result<String, String> {
    let mut name = String::new();
    let mut last_dash = false;

    for character in input.trim().to_lowercase().chars() {
        if character.is_ascii_alphanumeric() {
            name.push(character);
            last_dash = false;
        } else if !last_dash && !name.is_empty() {
            name.push('-');
            last_dash = true;
        }
        if name.len() >= 50 {
            break;
        }
    }

    let name = name.trim_end_matches('-').to_string();

    if name.is_empty() {
        return Err("站点名不能为空，只能包含字母、数字和连字符".to_string());
    }

    Ok(name)
}

struct TempOutput(PathBuf);

impl TempOutput {
    fn new() -> Result<Self, String> {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis())
            .unwrap_or(0);
        let path =
            std::env::temp_dir().join(format!("hugonic-deploy-{}-{stamp}", std::process::id()));
        std::fs::create_dir_all(&path).map_err(|error| format!("创建临时目录失败：{error}"))?;
        Ok(Self(path))
    }
}

impl Drop for TempOutput {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

pub fn zip_directory(root: &Path) -> Result<Vec<u8>, String> {
    if !root.is_dir() {
        return Err("public 目录不存在，构建可能失败".to_string());
    }

    let mut buffer = Cursor::new(Vec::<u8>::new());
    {
        let mut writer = zip::ZipWriter::new(&mut buffer);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        add_directory(&mut writer, root, root, options)?;
        writer
            .finish()
            .map_err(|error| format!("打包失败：{error}"))?;
    }

    Ok(buffer.into_inner())
}

fn add_directory(
    writer: &mut zip::ZipWriter<&mut Cursor<Vec<u8>>>,
    base: &Path,
    directory: &Path,
    options: zip::write::SimpleFileOptions,
) -> Result<(), String> {
    let entries = std::fs::read_dir(directory).map_err(|error| format!("读取目录失败：{error}"))?;

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
            add_directory(writer, base, &path, options)?;
            continue;
        }

        let relative = path
            .strip_prefix(base)
            .map_err(|_| "无法计算相对路径".to_string())?
            .to_string_lossy()
            .replace('\\', "/");

        writer
            .start_file(relative, options)
            .map_err(|error| format!("写入 zip 条目失败：{error}"))?;

        let mut file =
            std::fs::File::open(&path).map_err(|error| format!("读取文件失败：{error}"))?;
        let mut content = Vec::new();
        file.read_to_end(&mut content)
            .map_err(|error| format!("读取文件失败：{error}"))?;
        writer
            .write_all(&content)
            .map_err(|error| format!("写入 zip 失败：{error}"))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_site_names() {
        assert_eq!(normalize_site_name("My Blog").unwrap(), "my-blog");
        assert_eq!(normalize_site_name("  demo-site  ").unwrap(), "demo-site");
        assert_eq!(normalize_site_name("a__b--c").unwrap(), "a-b-c");
        assert!(normalize_site_name("   ").is_err());
    }

    #[test]
    fn ensures_trailing_slash() {
        assert_eq!(
            ensure_trailing_slash("https://demo.netlify.app"),
            "https://demo.netlify.app/"
        );
        assert_eq!(
            ensure_trailing_slash("https://demo.netlify.app/"),
            "https://demo.netlify.app/"
        );
    }

    #[test]
    fn parses_site_and_deploy_json() {
        let site: Site = serde_json::from_str(
            r#"{"id":"abc","name":"demo","ssl_url":"https://demo.netlify.app","url":"https://demo.netlify.app"}"#,
        )
        .unwrap();
        assert_eq!(site.id, "abc");
        assert_eq!(site.ssl_url, "https://demo.netlify.app");

        let deploy: Deploy = serde_json::from_str(
            r#"{"id":"d1","state":"ready","ssl_url":"https://demo.netlify.app","deploy_ssl_url":"https://d1--demo.netlify.app","error_message":null}"#,
        )
        .unwrap();
        assert_eq!(deploy.state, "ready");
        assert_eq!(
            deploy.deploy_ssl_url.as_deref(),
            Some("https://d1--demo.netlify.app")
        );
    }

    #[test]
    fn zips_directory_contents() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("index.html"), "<h1>hi</h1>").unwrap();
        std::fs::create_dir_all(dir.path().join("css")).unwrap();
        std::fs::write(dir.path().join("css/site.css"), "body{}").unwrap();

        let bytes = zip_directory(dir.path()).unwrap();

        let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
        let mut names: Vec<String> = (0..archive.len())
            .map(|index| archive.by_index(index).unwrap().name().unwrap().to_string())
            .collect();
        names.sort();
        assert_eq!(names, vec!["css/site.css", "index.html"]);

        let mut content = String::new();
        archive
            .by_name("index.html")
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();
        assert_eq!(content, "<h1>hi</h1>");
    }
}
