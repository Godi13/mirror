use chrono::{DateTime, Utc};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VersionInfo {
    current: String,
    latest: String,
    has_update: bool,
    download_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppVersion {
    version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GitHubRelease {
    tag_name: String,
    html_url: String,
    name: String,
    body: String,
}

// 缓存结构
#[derive(Debug, Clone)]
struct CachedRelease {
    release: GitHubRelease,
    cached_at: DateTime<Utc>,
}

// 全局缓存
static CACHE: LazyLock<Mutex<HashMap<String, CachedRelease>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_app_version() -> AppVersion {
    AppVersion {
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

#[tauri::command]
async fn check_for_updates() -> Result<VersionInfo, String> {
    let current_version = env!("CARGO_PKG_VERSION");

    // 调用 GitHub API 获取最新版本
    match fetch_latest_release().await {
        Ok(release) => {
            let latest_version = release.tag_name.trim_start_matches('v'); // 移除 'v' 前缀
            let has_update = compare_versions(current_version, latest_version);

            Ok(VersionInfo {
                current: current_version.to_string(),
                latest: latest_version.to_string(),
                has_update,
                download_url: if has_update {
                    Some(release.html_url)
                } else {
                    None
                },
            })
        }
        Err(e) => {
            // 如果 GitHub API 调用失败，返回错误而不是使用硬编码版本
            eprintln!("Failed to fetch latest release: {}", e);
            Err(format!("Failed to fetch latest release: {}", e))
        }
    }
}

async fn fetch_latest_release() -> Result<GitHubRelease, Box<dyn std::error::Error>> {
    let cache_key = "latest_release".to_string();

    // 检查缓存
    if let Ok(cache) = CACHE.lock() {
        if let Some(cached) = cache.get(&cache_key) {
            // 缓存有效期：10分钟
            let ten_minutes_ago = Utc::now() - chrono::Duration::minutes(10);
            if cached.cached_at > ten_minutes_ago {
                println!("Using cached release info");
                return Ok(cached.release.clone());
            }
        }
    }

    // 方案1: 尝试GitHub API
    match try_github_api().await {
        Ok(release) => {
            // 更新缓存
            if let Ok(mut cache) = CACHE.lock() {
                cache.insert(
                    cache_key,
                    CachedRelease {
                        release: release.clone(),
                        cached_at: Utc::now(),
                    },
                );
            }
            return Ok(release);
        }
        Err(e) => {
            eprintln!("GitHub API failed: {}", e);
        }
    }

    // 方案2: 使用GitHub Pages或其他CDN（如果有的话）
    match try_alternative_sources().await {
        Ok(release) => return Ok(release),
        Err(e) => eprintln!("Alternative sources failed: {}", e),
    }

    // 方案3: 如果所有方案都失败，返回一个有意义的错误
    Err("All update check methods failed. This could be due to network issues or GitHub API rate limits. Please try again later.".into())
}

// GitHub API方法
async fn try_github_api() -> Result<GitHubRelease, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.github.com/repos/Godi13/mirror/releases/latest")
        .header("User-Agent", "mirror-app")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?;

    if response.status() == 403 {
        // 检查是否是速率限制
        let headers = response.headers();
        if let Some(remaining) = headers.get("x-ratelimit-remaining") {
            if remaining == "0" {
                return Err("GitHub API rate limit exceeded".into());
            }
        }
    }

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await?;
        return Err(format!("GitHub API error: {} - {}", status, text).into());
    }

    let release: GitHubRelease = response.json().await?;
    Ok(release)
}

// 后备方案：爬取GitHub releases页面
async fn try_alternative_sources() -> Result<GitHubRelease, Box<dyn std::error::Error>> {
    // 方案A: 直接访问releases页面，解析HTML
    match try_scrape_releases_page().await {
        Ok(release) => return Ok(release),
        Err(e) => eprintln!("Scraping failed: {}", e),
    }

    Err("All alternative sources failed".into())
}

// 爬取GitHub releases页面
async fn try_scrape_releases_page() -> Result<GitHubRelease, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://github.com/Godi13/mirror/releases/latest")
        .header("User-Agent", "mirror-app")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("Failed to fetch releases page: {}", response.status()).into());
    }

    let html = response.text().await?;

    // 简单的HTML解析，提取版本号
    if let Some(version) = extract_version_from_html(&html) {
        return Ok(GitHubRelease {
            tag_name: version.clone(),
            html_url: format!("https://github.com/Godi13/mirror/releases/tag/{}", version),
            name: format!("Mirror {}", version),
            body: "Retrieved from releases page".to_string(),
        });
    }

    Err("Could not extract version from releases page".into())
}

// 从HTML中提取版本号
fn extract_version_from_html(html: &str) -> Option<String> {
    // 查找类似 "/Godi13/mirror/releases/tag/v0.1.5" 的模式
    if let Some(start) = html.find("/Godi13/mirror/releases/tag/") {
        let substr = &html[start + 29..]; // 跳过 "/Godi13/mirror/releases/tag/"
        if let Some(end) = substr.find('"') {
            let version = &substr[..end];
            if !version.is_empty() {
                return Some(version.to_string());
            }
        }
    }
    None
}

fn compare_versions(current: &str, latest: &str) -> bool {
    // 使用 semver 进行更可靠的版本比较
    match (Version::parse(current), Version::parse(latest)) {
        (Ok(current_ver), Ok(latest_ver)) => latest_ver > current_ver,
        _ => {
            // 如果版本解析失败，退回到简单字符串比较
            eprintln!(
                "Failed to parse versions: current={}, latest={}",
                current, latest
            );
            current != latest && latest > current
        }
    }
}

#[tauri::command]
async fn trigger_update_check(_app: tauri::AppHandle) -> Result<String, String> {
    println!("trigger_update_check 被调用");

    // 使用我们自己的更新检测逻辑
    match check_for_updates().await {
        Ok(version_info) => {
            if version_info.has_update {
                let result = format!(
                    "发现新版本！当前版本: {}, 最新版本: {}",
                    version_info.current, version_info.latest
                );
                println!("发现更新: {}", result);

                // 开始增量下载和安装过程
                match download_and_install_update(version_info.clone()).await {
                    Ok(install_result) => Ok(format!(
                        "{}
{}",
                        result, install_result
                    )),
                    Err(e) => {
                        // 如果自动安装失败，提供手动下载链接
                        if let Some(download_url) = version_info.download_url {
                            Ok(format!(
                                "{}
自动安装失败: {}
请手动下载安装: {}",
                                result, e, download_url
                            ))
                        } else {
                            Err(format!("更新失败: {}", e))
                        }
                    }
                }
            } else {
                let result = "当前已是最新版本".to_string();
                println!("没有可用更新");
                Ok(result)
            }
        }
        Err(e) => {
            let error_msg = format!("检查更新失败: {}", e);
            println!("检查更新失败: {}", error_msg);
            Err(error_msg)
        }
    }
}

// 下载和安装更新的核心函数
async fn download_and_install_update(
    version_info: VersionInfo,
) -> Result<String, Box<dyn std::error::Error>> {
    println!("开始下载更新...");

    // 1. 确定下载URL
    let download_url = match version_info.download_url {
        Some(url) => {
            // 将GitHub releases页面URL转换为实际的下载URL
            if url.contains("/releases/tag/") {
                // 根据平台选择合适的文件
                let platform = detect_platform();
                get_download_url_for_platform(&version_info.latest, &platform).await?
            } else {
                url
            }
        }
        None => return Err("没有找到下载URL".into()),
    };

    println!("下载URL: {}", download_url);

    // 2. 创建临时目录
    let temp_dir = std::env::temp_dir().join(format!("mirror_update_{}", version_info.latest));
    std::fs::create_dir_all(&temp_dir)?;

    // 3. 下载文件
    let file_name = download_url.split('/').last().unwrap_or("update.dmg");
    let file_path = temp_dir.join(file_name);

    println!("正在下载到: {:?}", file_path);
    download_file(&download_url, &file_path).await?;

    // 4. 验证下载文件
    if !file_path.exists() {
        return Err("下载文件不存在".into());
    }

    // 5. 安装更新（这里我们启动安装程序，而不是直接替换）
    install_update(&file_path).await?;

    Ok(format!(
        "更新下载完成！安装程序已启动，请按照提示完成安装。"
    ))
}

// 检测当前平台
fn detect_platform() -> String {
    #[cfg(target_arch = "aarch64")]
    {
        "aarch64".to_string()
    }
    #[cfg(target_arch = "x86_64")]
    {
        "x64".to_string()
    }
    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    {
        "x64".to_string() // 默认
    }
}

// 根据平台获取下载URL
async fn get_download_url_for_platform(
    version: &str,
    platform: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // 根据你的发布文件名规则构造URL
    let file_name = if platform == "aarch64" {
        format!("mirror_0.1.0_aarch64.dmg")
    } else {
        format!("mirror_0.1.0_x64.dmg")
    };

    let url = format!(
        "https://github.com/Godi13/mirror/releases/download/{}/{}",
        version, file_name
    );

    // 验证URL是否有效
    let client = reqwest::Client::new();
    let response = client.head(&url).send().await?;

    if response.status().is_success() {
        Ok(url)
    } else {
        Err(format!("找不到平台 {} 的安装包", platform).into())
    }
}

// 下载文件的函数
async fn download_file(
    url: &str,
    file_path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Err(format!("下载失败: HTTP {}", response.status()).into());
    }

    let content = response.bytes().await?;
    std::fs::write(file_path, content)?;

    println!(
        "文件下载完成: {:?} ({} 字节)",
        file_path,
        file_path.metadata()?.len()
    );
    Ok(())
}

// 安装更新的函数
async fn install_update(file_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("开始安装更新: {:?}", file_path);

    #[cfg(target_os = "macos")]
    {
        // 在macOS上，我们可以使用open命令来启动.dmg文件
        let output = std::process::Command::new("open").arg(file_path).output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(format!("启动安装程序失败: {}", error).into());
        }

        println!("安装程序已启动");
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    {
        // 在其他平台上，可以添加相应的安装逻辑
        Err("当前平台不支持自动安装".into())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_app_version,
            check_for_updates,
            trigger_update_check
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
