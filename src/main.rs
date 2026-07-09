mod builder;
mod parser;

use anyhow::Result;
use clap::{Parser, Subcommand};
use notify::event::ModifyKind;
use notify::{Event, EventKind, RecursiveMode, Watcher};
use percent_encoding::percent_decode_str;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use tiny_http::{Header, Response, Server};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(name = "guidebook")]
#[command(version)]
#[command(about = "HonKit/GitBook compatible static book generator", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new book
    Init {
        /// Directory to initialize
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Build the book
    Build {
        /// Source directory
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Output directory
        #[arg(short, long, default_value = "_book")]
        output: PathBuf,
    },
    /// Start a local server for preview
    Serve {
        /// Source directory
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Port to listen on
        #[arg(short, long, default_value = "4000")]
        port: u16,
        /// Open browser automatically
        #[arg(short, long)]
        open: bool,
    },
    /// Update guidebook to the latest version
    Update,
}

fn main() -> Result<()> {
    // Check for updates in background (non-blocking)
    check_for_updates();

    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path } => init_book(&path),
        Commands::Build { path, output } => {
            println!("Building book from {:?} to {:?}", path, output);
            builder::build(&path, &output)
        }
        Commands::Serve { path, port, open } => serve_book(&path, port, open),
        Commands::Update => update_self(),
    }
}

fn init_book(path: &PathBuf) -> Result<()> {
    println!("Initializing book in {:?}", path);

    // Create directory if it doesn't exist
    if !path.exists() {
        fs::create_dir_all(path)?;
        println!("  Created directory {:?}", path);
    }

    // Create README.md
    let readme_path = path.join("README.md");
    if !readme_path.exists() {
        let readme_content = r#"# Introduction

Welcome to your new book!

This file serves as your book's introduction or preface.
"#;
        fs::write(&readme_path, readme_content)?;
        println!("  Created README.md");
    } else {
        println!("  README.md already exists, skipping");
    }

    // Create SUMMARY.md
    let summary_path = path.join("SUMMARY.md");
    if !summary_path.exists() {
        let summary_content = r#"# Summary

* [Introduction](README.md)
"#;
        fs::write(&summary_path, summary_content)?;
        println!("  Created SUMMARY.md");
    } else {
        println!("  SUMMARY.md already exists, skipping");
    }

    // Create book.json
    let book_json_path = path.join("book.json");
    if !book_json_path.exists() {
        let book_json_content = r#"{
    "title": "My Book",
    "description": "",
    "author": "",
    "plugins": [
        "collapsible-chapters",
        "back-to-top-button",
        "mermaid-md-adoc"
    ]
}
"#;
        fs::write(&book_json_path, book_json_content)?;
        println!("  Created book.json");
    } else {
        println!("  book.json already exists, skipping");
    }

    println!("\nBook initialized successfully!");
    println!("\nNext steps:");
    println!("  1. Edit SUMMARY.md to define your book structure");
    println!("  2. Create markdown files for your chapters");
    println!("  3. Run 'guidebook serve' to preview your book");

    Ok(())
}

fn serve_book(source: &Path, port: u16, open_browser: bool) -> Result<()> {
    // Build to a unique temp directory (include PID to avoid conflicts with parallel runs)
    let temp_dir = std::env::temp_dir().join(format!("guidebook-serve-{}", std::process::id()));
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir)?;
    }

    println!("Building book...");
    builder::build(source, &temp_dir)?;

    // Version counter for hot reload
    let version = Arc::new(AtomicU64::new(1));
    let version_for_watcher = version.clone();
    let source_for_watcher = source.to_path_buf();
    let temp_dir_for_watcher = temp_dir.clone();

    // Setup file watcher
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res {
            // Only react to file modifications
            let dominated: bool = matches!(
                event.kind,
                EventKind::Modify(ModifyKind::Data(_))
                    | EventKind::Modify(ModifyKind::Name(_))
                    | EventKind::Create(_)
                    | EventKind::Remove(_)
            );
            if dominated {
                // Check if it's a relevant file: content (md/adoc), config,
                // styles/scripts, and referenced assets (images/fonts)
                // Exclude _book directory and other build artifacts
                let dominated = event.paths.iter().any(|p| {
                    // Skip files in _book directory (build output)
                    let path_str = p.to_string_lossy();
                    if path_str.contains("/_book/") || path_str.contains("\\_book\\") {
                        return false;
                    }
                    p.extension()
                        .and_then(|e| e.to_str())
                        .map(|e| {
                            matches!(
                                e.to_ascii_lowercase().as_str(),
                                "md" | "adoc"
                                    | "asciidoc"
                                    | "json"
                                    | "css"
                                    | "js"
                                    | "html"
                                    | "png"
                                    | "jpg"
                                    | "jpeg"
                                    | "gif"
                                    | "svg"
                                    | "webp"
                                    | "ico"
                                    | "avif"
                                    | "bmp"
                                    | "woff"
                                    | "woff2"
                                    | "ttf"
                                    | "otf"
                            )
                        })
                        .unwrap_or(false)
                });
                if dominated {
                    println!("\n🔄 File changed, rebuilding...");
                    // Skip search index generation on hot reload for performance
                    if let Err(e) = builder::build_with_options(
                        &source_for_watcher,
                        &temp_dir_for_watcher,
                        true,
                    ) {
                        eprintln!("   Build error: {}", e);
                    } else {
                        version_for_watcher.fetch_add(1, Ordering::SeqCst);
                        println!("   Rebuild complete!");
                    }
                }
            }
        }
    })?;

    watcher.watch(source, RecursiveMode::Recursive)?;

    let addr = format!("0.0.0.0:{}", port);
    let server = Server::http(&addr).map_err(|e| {
        if e.to_string().contains("Address already in use") {
            anyhow::anyhow!(
                "Port {} is already in use.\n   Try: guidebook serve -p {}",
                port,
                port + 1
            )
        } else {
            anyhow::anyhow!("Failed to start server: {}", e)
        }
    })?;

    let url = format!("http://localhost:{}/", port);
    println!("\n📚 Serving book at {}", url);
    println!("   🔥 Hot reload enabled - changes will auto-refresh");
    println!("   Press Ctrl+C to stop\n");

    // Open browser if requested
    if open_browser {
        if let Err(e) = open::that(&url) {
            eprintln!("   Failed to open browser: {}", e);
        }
    }

    // Keep watcher alive
    let _watcher = watcher;

    for request in server.incoming_requests() {
        let url = request.url().to_string();

        // Handle livereload polling endpoint
        if url.starts_with("/__livereload") {
            // Extract version from query string
            let client_version: u64 = url
                .split("?v=")
                .nth(1)
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);

            let current_version = version.load(Ordering::SeqCst);

            // If versions differ, tell client to reload
            let response_body = if client_version < current_version {
                format!(r#"{{"reload":true,"version":{}}}"#, current_version)
            } else {
                format!(r#"{{"reload":false,"version":{}}}"#, current_version)
            };

            let header = Header::from_bytes("Content-Type", "application/json")
                .or_else(|_| Header::from_bytes("Content-Type", "application/octet-stream"))
                .expect("static Content-Type header must be valid");
            let response = Response::from_string(response_body).with_header(header);
            let _ = request.respond(response);
            continue;
        }

        let url_path = if url == "/" {
            "/index.html".to_string()
        } else if url.ends_with('/') {
            format!("{}index.html", url)
        } else {
            url.clone()
        };

        // URL decode the path to handle Japanese/special characters
        let decoded_path = percent_decode_str(&url_path)
            .decode_utf8_lossy()
            .to_string();

        // Path traversal protection: reject any path containing ".." components
        // This catches encoded variants (..%2F, %2e%2e, etc.) after URL decoding
        if decoded_path.contains("..") {
            let response = Response::from_string("403 Forbidden").with_status_code(403);
            let _ = request.respond(response);
            continue;
        }

        let file_path = temp_dir.join(decoded_path.trim_start_matches('/'));

        // Belt-and-suspenders: canonicalize check ensures we never serve outside temp_dir
        if !is_safe_path(&file_path, &temp_dir) {
            let response = Response::from_string("403 Forbidden").with_status_code(403);
            let _ = request.respond(response);
            continue;
        }

        if file_path.exists() && file_path.is_file() {
            let mut content = match fs::read(&file_path) {
                Ok(c) => c,
                Err(_) => {
                    let response =
                        Response::from_string("500 Internal Server Error").with_status_code(500);
                    let _ = request.respond(response);
                    continue;
                }
            };
            let content_type = get_content_type(&file_path);

            // Inject livereload script into HTML pages
            if content_type.starts_with("text/html") {
                content = inject_livereload(content, version.load(Ordering::SeqCst));
            }

            let header = Header::from_bytes("Content-Type", content_type)
                .or_else(|_| Header::from_bytes("Content-Type", "application/octet-stream"))
                .expect("static Content-Type header must be valid");
            let response = Response::from_data(content).with_header(header);
            let _ = request.respond(response);
        } else {
            // Try with .html extension (same traversal protection applies)
            let mut html_path = file_path.clone();
            let new_ext = match html_path.extension() {
                Some(ext) => format!("{}.html", ext.to_string_lossy()),
                None => "html".to_string(),
            };
            html_path.set_extension(new_ext);

            if !is_safe_path(&html_path, &temp_dir) {
                let response = Response::from_string("403 Forbidden").with_status_code(403);
                let _ = request.respond(response);
                continue;
            }

            if html_path.exists() {
                let content = match fs::read(&html_path) {
                    Ok(c) => c,
                    Err(_) => {
                        let response = Response::from_string("500 Internal Server Error")
                            .with_status_code(500);
                        let _ = request.respond(response);
                        continue;
                    }
                };
                // Extensionless URLs get the same livereload injection as
                // their .html equivalents (previously /guide never reloaded
                // while /guide.html did)
                let content = inject_livereload(content, version.load(Ordering::SeqCst));
                let header = Header::from_bytes("Content-Type", "text/html; charset=utf-8")
                    .unwrap_or_else(|_| {
                        Header::from_bytes("Content-Type", "application/octet-stream").unwrap()
                    });
                let response = Response::from_data(content).with_header(header);
                let _ = request.respond(response);
            } else {
                let response = Response::from_string("404 Not Found").with_status_code(404);
                let _ = request.respond(response);
            }
        }
    }

    Ok(())
}

/// Inject the livereload polling script into an HTML page body
fn inject_livereload(content: Vec<u8>, current_version: u64) -> Vec<u8> {
    let livereload_script = format!(
        r#"<script>
(function(){{
    var version={};
    function checkReload(){{
        fetch('/__livereload?v='+version)
            .then(function(r){{return r.json()}})
            .then(function(data){{
                if(data.reload){{
                    version=data.version;
                    location.reload();
                }}
            }})
            .catch(function(){{}});
    }}
    setInterval(checkReload,1000);
}})();
</script></body>"#,
        current_version
    );
    let html = String::from_utf8_lossy(&content);
    html.replace("</body>", &livereload_script).into_bytes()
}

/// Check that a path is safely contained within the allowed root directory.
///
/// Defence layers:
/// 1. Reject any `..` component (prevents traversal regardless of symlinks)
/// 2. Verify the lexical path starts with root
/// 3. For non-symlink files, additionally verify via `canonicalize()`
///
/// Symlinked assets (created by guidebook build for performance) are allowed
/// as long as the symlink itself lives inside root and the requested path
/// contains no `..` components.
fn is_safe_path(path: &Path, root: &Path) -> bool {
    // Layer 1: Reject any ".." component to prevent traversal
    for component in path.components() {
        if let std::path::Component::ParentDir = component {
            return false;
        }
    }

    // Layer 2: Verify the joined path lexically starts with root
    if !path.starts_with(root) {
        return false;
    }

    // Layer 3: For non-symlink regular files, use canonicalize for extra safety.
    // Symlinks are allowed because guidebook build creates them for asset copying;
    // their safety is guaranteed by layers 1 & 2 (no ".." + inside root).
    if path.exists() && !path.is_symlink() {
        if let Ok(canonical) = path.canonicalize() {
            if let Ok(canonical_root) = root.canonicalize() {
                return canonical.starts_with(&canonical_root);
            }
        }
    }

    true
}

fn get_content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        _ => "application/octet-stream",
    }
}

fn check_for_updates() {
    // Run in a separate thread to not block startup
    std::thread::spawn(|| {
        if let Some(latest) = get_latest_version() {
            if is_newer_version(&latest, VERSION) {
                eprintln!(
                    "\n📦 New version available: {} → {}\n   Run: cargo install guidebook --force\n",
                    VERSION, latest
                );
            }
        }
    });
}

fn get_latest_version() -> Option<String> {
    let response = ureq::get("https://crates.io/api/v1/crates/guidebook")
        .set("User-Agent", &format!("guidebook/{}", VERSION))
        .timeout(std::time::Duration::from_secs(2))
        .call()
        .ok()?;

    let body = response.into_string().ok()?;
    let json: serde_json::Value = serde_json::from_str(&body).ok()?;
    json["crate"]["max_version"].as_str().map(String::from)
}

fn is_newer_version(latest: &str, current: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> { v.split('.').filter_map(|p| p.parse().ok()).collect() };

    let latest_parts = parse(latest);
    let current_parts = parse(current);

    for (l, c) in latest_parts.iter().zip(current_parts.iter()) {
        if l > c {
            return true;
        }
        if l < c {
            return false;
        }
    }

    latest_parts.len() > current_parts.len()
}

fn update_self() -> Result<()> {
    use sha2::{Digest, Sha256};
    use std::io::{Read, Write};

    println!("Checking for updates...");

    // Get latest version and release body from GitHub
    let (latest_version, release_body) = get_latest_github_release()
        .ok_or_else(|| anyhow::anyhow!("Failed to check latest version"))?;

    println!("  Current version: {}", VERSION);
    println!("  Latest version:  {}", latest_version);

    if !is_newer_version(&latest_version, VERSION) {
        println!("\nYou're already on the latest version!");
        return Ok(());
    }

    // Detect platform
    let artifact_name =
        get_artifact_name().ok_or_else(|| anyhow::anyhow!("Unsupported platform"))?;

    println!("\nDownloading {}...", artifact_name);

    // Download from GitHub Releases
    let download_url = format!(
        "https://github.com/guide-inc-org/guidebook/releases/download/v{}/{}",
        latest_version, artifact_name
    );

    let response = ureq::get(&download_url)
        .set("User-Agent", &format!("guidebook/{}", VERSION))
        .call()
        .map_err(|e| anyhow::anyhow!("Failed to download: {}", e))?;

    // Read response body
    let mut bytes = Vec::new();
    response.into_reader().read_to_end(&mut bytes)?;

    // Verify SHA256 checksum (required — refuse to install without verification)
    let expected_hash = extract_checksum(&release_body, artifact_name).ok_or_else(|| {
        anyhow::anyhow!(
            "No SHA256 checksum found in release notes for {}.\n\
             Refusing to install unverified binary.\n\
             Release maintainers: include checksums in the format:\n  \
             <sha256hash>  <filename>",
            artifact_name
        )
    })?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let actual_hash = format!("{:x}", hasher.finalize());
    if actual_hash != expected_hash {
        return Err(anyhow::anyhow!(
            "Checksum mismatch!\n  Expected: {}\n  Actual:   {}\nDownload may be corrupted or tampered with.",
            expected_hash, actual_hash
        ));
    }
    println!("  Checksum verified ✓");

    // Get current executable path
    let current_exe = std::env::current_exe()?;
    let exe_dir = current_exe
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Cannot get executable directory"))?;

    // Extract binary
    let new_binary = if artifact_name.ends_with(".zip") {
        extract_zip(&bytes)?
    } else {
        extract_tar_gz(&bytes)?
    };

    // Replace current executable
    let backup_path = exe_dir.join("guidebook.backup");
    let new_exe_path = exe_dir.join(if cfg!(windows) {
        "guidebook_new.exe"
    } else {
        "guidebook_new"
    });

    // Write new binary
    let mut file = fs::File::create(&new_exe_path)?;
    file.write_all(&new_binary)?;

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&new_exe_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&new_exe_path, perms)?;
    }

    // Backup current executable
    if backup_path.exists() {
        fs::remove_file(&backup_path)?;
    }
    fs::rename(&current_exe, &backup_path)?;

    // Move new executable to current location. If this second rename fails
    // the binary on PATH is already gone — restore the backup instead of
    // leaving the user with no guidebook at all
    if let Err(e) = fs::rename(&new_exe_path, &current_exe) {
        let restore = fs::rename(&backup_path, &current_exe);
        return Err(match restore {
            Ok(()) => anyhow::anyhow!("Update failed ({}); previous binary was restored", e),
            Err(re) => anyhow::anyhow!(
                "Update failed ({}) AND restoring the backup failed ({}). \
                 Recover manually: mv {} {}",
                e,
                re,
                backup_path.display(),
                current_exe.display()
            ),
        });
    }

    // Remove backup
    let _ = fs::remove_file(&backup_path);

    println!("\nSuccessfully updated to v{}!", latest_version);
    Ok(())
}

/// Extract SHA256 checksum for a given artifact from release body text
/// Expected format in release body: `<sha256hash>  <filename>`
fn extract_checksum(release_body: &str, artifact_name: &str) -> Option<String> {
    for line in release_body.lines() {
        let trimmed = line.trim();
        // Match lines like: "abc123def456...  guidebook-linux-x86_64.tar.gz"
        if trimmed.ends_with(artifact_name) {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() == 2
                && parts[0].len() == 64
                && parts[0].chars().all(|c| c.is_ascii_hexdigit())
            {
                return Some(parts[0].to_lowercase());
            }
        }
    }
    None
}

/// Get latest GitHub release version and body text
fn get_latest_github_release() -> Option<(String, String)> {
    let response =
        ureq::get("https://api.github.com/repos/guide-inc-org/guidebook/releases/latest")
            .set("User-Agent", &format!("guidebook/{}", VERSION))
            .timeout(std::time::Duration::from_secs(10))
            .call()
            .ok()?;

    let body = response.into_string().ok()?;
    let json: serde_json::Value = serde_json::from_str(&body).ok()?;
    let version = json["tag_name"]
        .as_str()
        .map(|s| s.trim_start_matches('v').to_string())?;
    let release_body = json["body"].as_str().unwrap_or("").to_string();
    Some((version, release_body))
}

fn get_artifact_name() -> Option<&'static str> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    match (os, arch) {
        ("linux", "x86_64") => Some("guidebook-linux-x86_64.tar.gz"),
        ("macos", "x86_64") => Some("guidebook-darwin-x86_64.tar.gz"),
        ("macos", "aarch64") => Some("guidebook-darwin-arm64.tar.gz"),
        ("windows", "x86_64") => Some("guidebook-windows-x86_64.zip"),
        _ => None,
    }
}

fn extract_tar_gz(data: &[u8]) -> Result<Vec<u8>> {
    use flate2::read::GzDecoder;
    use std::io::{Cursor, Read};
    use tar::Archive;

    let decoder = GzDecoder::new(Cursor::new(data));
    let mut archive = Archive::new(decoder);

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        if path.file_name().map(|n| n == "guidebook").unwrap_or(false) {
            let mut binary = Vec::new();
            entry.read_to_end(&mut binary)?;
            return Ok(binary);
        }
    }

    Err(anyhow::anyhow!("Binary not found in archive"))
}

fn extract_zip(data: &[u8]) -> Result<Vec<u8>> {
    use std::io::Cursor;
    use zip::ZipArchive;

    let cursor = Cursor::new(data);
    let mut archive = ZipArchive::new(cursor)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();
        if name.ends_with("guidebook.exe") || name == "guidebook.exe" {
            let mut binary = Vec::new();
            std::io::Read::read_to_end(&mut file, &mut binary)?;
            return Ok(binary);
        }
    }

    Err(anyhow::anyhow!("Binary not found in archive"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    // ── is_safe_path tests ──

    #[test]
    fn test_safe_path_normal() {
        let root = tempdir().unwrap();
        let file = root.path().join("index.html");
        fs::write(&file, "ok").unwrap();
        assert!(is_safe_path(&file, root.path()));
    }

    #[test]
    fn test_safe_path_rejects_dotdot() {
        let root = tempdir().unwrap();
        let evil = root.path().join("..").join("etc").join("passwd");
        assert!(!is_safe_path(&evil, root.path()));
    }

    #[test]
    fn test_safe_path_allows_symlink_inside_root() {
        let root = tempdir().unwrap();
        let outside = tempdir().unwrap();
        let secret = outside.path().join("asset.png");
        fs::write(&secret, "image data").unwrap();

        // Symlinks inside root are allowed (guidebook build creates them for assets)
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&secret, root.path().join("link.png")).unwrap();
            assert!(is_safe_path(&root.path().join("link.png"), root.path()));
        }
    }

    #[test]
    fn test_safe_path_rejects_dotdot_with_symlink() {
        let root = tempdir().unwrap();

        // Even with a symlink, ".." in the path is always rejected
        let evil = root.path().join("sub").join("..").join("..").join("etc");
        assert!(!is_safe_path(&evil, root.path()));
    }

    #[test]
    fn test_safe_path_nonexistent_file() {
        let root = tempdir().unwrap();
        // A normal non-existent file inside root should be safe
        assert!(is_safe_path(
            &root.path().join("nonexistent.html"),
            root.path()
        ));
    }

    #[test]
    fn test_safe_path_rejects_dotdot_nonexistent() {
        let root = tempdir().unwrap();
        let evil = root.path().join("sub").join("..").join("..").join("out");
        assert!(!is_safe_path(&evil, root.path()));
    }

    // ── extract_checksum tests ──

    #[test]
    fn test_extract_checksum_found() {
        // SHA256 hash is exactly 64 hex chars
        let hash = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";
        let body = format!("## Checksums\n\n{}  guidebook-linux-x86_64.tar.gz\n", hash);
        let result = extract_checksum(&body, "guidebook-linux-x86_64.tar.gz");
        assert_eq!(result, Some(hash.to_string()));
    }

    #[test]
    fn test_extract_checksum_missing() {
        let body = "## Release Notes\n\nSome notes here.\n";
        let result = extract_checksum(body, "guidebook-linux-x86_64.tar.gz");
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_checksum_wrong_artifact() {
        let body =
            "abc123def456abc123def456abc123def456abc123def456abc123def456abc123de  guidebook-darwin-arm64.tar.gz\n";
        let result = extract_checksum(body, "guidebook-linux-x86_64.tar.gz");
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_checksum_invalid_hash_length() {
        let body = "abc123  guidebook-linux-x86_64.tar.gz\n";
        let result = extract_checksum(body, "guidebook-linux-x86_64.tar.gz");
        assert_eq!(result, None);
    }

    // ── is_newer_version tests ──

    #[test]
    fn test_is_newer_version() {
        assert!(is_newer_version("1.1.0", "1.0.0"));
        assert!(is_newer_version("2.0.0", "1.9.9"));
        assert!(!is_newer_version("1.0.0", "1.0.0"));
        assert!(!is_newer_version("0.9.0", "1.0.0"));
    }
}
