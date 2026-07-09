//! Remote image downloading for offline viewing
//!
//! Downloads `https://` images at build time and replaces URLs in HTML
//! with local paths for offline access.

use crc32fast::Hasher;
use regex::Regex;
use reqwest::blocking::Client;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Maximum allowed image download size (50 MB)
const MAX_IMAGE_SIZE: usize = 50 * 1024 * 1024;

/// Downloads and caches remote images for offline viewing
pub struct ImageDownloader {
    client: Client,
    cache: HashMap<String, String>,
    images_dir: PathBuf,
}

impl ImageDownloader {
    /// Create a new ImageDownloader
    ///
    /// # Arguments
    /// * `output_dir` - The root output directory for the book build
    pub fn new(output_dir: &Path) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| Client::new());

        let images_dir = output_dir.join("_remote_images");

        ImageDownloader {
            client,
            cache: HashMap::new(),
            images_dir,
        }
    }

    /// Process HTML content and download any remote images
    ///
    /// Finds all `<img src="https://...">` tags and downloads the images,
    /// replacing the URLs with local paths.
    ///
    /// # Arguments
    /// * `html` - The HTML content to process
    /// * `depth` - Directory depth of this HTML file below the output root
    ///   (0 for output/index.html, 2 for output/a/b/page.html). The local
    ///   image path gets a matching `../` prefix so nested pages resolve it.
    ///
    /// # Returns
    /// The HTML with remote image URLs replaced with local paths
    pub fn process_html(
        &mut self,
        html: &str,
        depth: usize,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Regex to match img src attributes with https:// URLs
        let img_re = Regex::new(r#"<img\s+([^>]*?)src\s*=\s*["']((https?://[^"']+))["']([^>]*)>"#)?;

        let root_prefix = "../".repeat(depth);
        let mut result = html.to_string();
        let mut replacements: Vec<(String, String)> = Vec::new();

        for caps in img_re.captures_iter(html) {
            let full_match = caps.get(0).unwrap().as_str();
            let before_src = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let url = caps.get(2).unwrap().as_str();
            let after_src = caps.get(4).map(|m| m.as_str()).unwrap_or("");

            // Only process https:// URLs
            if !url.starts_with("https://") && !url.starts_with("http://") {
                continue;
            }

            // The src attribute comes from rendered HTML, so entities are
            // escaped (& is always &amp;) — decode before fetching or query
            // strings with multiple parameters break (signed URLs 403 etc.)
            let fetch_url = decode_html_entities(url);

            // Download the image and get local path
            match self.download_image(&fetch_url) {
                Ok(local_path) => {
                    // Drop a trailing "/" left in after_src by self-closing
                    // tags — the closer is re-appended below (previously this
                    // produced "//>")
                    let after_src_clean = after_src.trim_end().trim_end_matches('/');
                    let new_tag = format!(
                        r#"<img {}src="{}{}"{}"#,
                        before_src, root_prefix, local_path, after_src_clean
                    );
                    // Close the tag properly
                    let new_tag = if full_match.trim_end().ends_with("/>") {
                        format!("{}/>", new_tag)
                    } else {
                        format!("{}>", new_tag)
                    };
                    replacements.push((full_match.to_string(), new_tag));
                }
                Err(e) => {
                    eprintln!("  Warning: Failed to download image {}: {}", fetch_url, e);
                    // Keep original URL on failure
                }
            }
        }

        // Apply replacements
        for (old, new) in replacements {
            result = result.replace(&old, &new);
        }

        Ok(result)
    }

    /// Download an image from a URL and return the local path.
    /// Uses streaming download with Content-Length pre-check and in-flight size limit.
    fn download_image(&mut self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        use std::io::Read;

        // Check cache first
        if let Some(cached_path) = self.cache.get(url) {
            return Ok(cached_path.clone());
        }

        // Create images directory if needed
        fs::create_dir_all(&self.images_dir)?;

        // Download the image
        let response = self.client.get(url).send()?;

        if !response.status().is_success() {
            return Err(format!("HTTP {}", response.status()).into());
        }

        // Pre-check Content-Length header if present
        if let Some(content_length) = response.content_length() {
            if content_length as usize > MAX_IMAGE_SIZE {
                return Err(format!(
                    "Image too large (Content-Length: {:.1} MB, max {} MB)",
                    content_length as f64 / 1024.0 / 1024.0,
                    MAX_IMAGE_SIZE / 1024 / 1024
                )
                .into());
            }
        }

        // Capture the Content-Type before consuming the body
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_ascii_lowercase();

        // Stream the response body in chunks with size limit enforcement
        // Read up to MAX_IMAGE_SIZE+1 bytes; if we get more, the image is too large
        let mut bytes = Vec::new();
        let mut total_read: usize = 0;
        let mut buf = [0u8; 8192];
        let mut reader = response;
        loop {
            let n = reader.read(&mut buf)?;
            if n == 0 {
                break;
            }
            total_read += n;
            if total_read > MAX_IMAGE_SIZE {
                return Err(format!(
                    "Image too large (>{} MB, download aborted mid-stream)",
                    MAX_IMAGE_SIZE / 1024 / 1024
                )
                .into());
            }
            bytes.extend_from_slice(&buf[..n]);
        }

        // Refuse to save responses that are not recognizably images —
        // an expired-auth HTML page served with 200 would otherwise be
        // written as a broken ".png" without any warning
        let ext = match detect_extension(url, &content_type, &bytes) {
            Some(ext) => ext,
            None => {
                return Err(format!(
                    "response is not an image (content-type: {})",
                    if content_type.is_empty() {
                        "unknown"
                    } else {
                        &content_type
                    }
                )
                .into());
            }
        };

        // Generate filename from URL hash + detected extension
        let hash = crc32_hash(url);
        let filename = format!("{:08x}.{}", hash, ext);
        let file_path = self.images_dir.join(&filename);

        // Write the file
        fs::write(&file_path, &bytes)?;

        // Calculate relative path from output root (cache AFTER successful write)
        let relative_path = format!("_remote_images/{}", filename);
        self.cache.insert(url.to_string(), relative_path.clone());

        Ok(relative_path)
    }

    /// Get download statistics
    pub fn stats(&self) -> (usize, usize) {
        (self.cache.len(), 0) // (downloaded, failed)
    }
}

/// Calculate CRC32 hash of a string
fn crc32_hash(s: &str) -> u32 {
    let mut hasher = Hasher::new();
    hasher.update(s.as_bytes());
    hasher.finalize()
}

/// Decode the HTML entities that pulldown-cmark's escape_href produces in
/// attribute values, so the URL can be fetched as the author wrote it.
/// `&amp;` must be decoded LAST to avoid double-decoding (&amp;lt; → &lt; → <).
fn decode_html_entities(url: &str) -> String {
    url.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#x27;", "'")
        .replace("&#x2F;", "/")
        .replace("&amp;", "&")
}

/// Detect image extension from magic bytes, Content-Type, or URL.
/// Returns None when the payload is not recognizably an image.
fn detect_extension(url: &str, content_type: &str, bytes: &[u8]) -> Option<&'static str> {
    // Try to detect from magic bytes first
    if bytes.len() >= 8 {
        // PNG: 89 50 4E 47 0D 0A 1A 0A
        if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
            return Some("png");
        }
        // JPEG: FF D8 FF
        if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return Some("jpg");
        }
        // GIF: GIF87a or GIF89a
        if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
            return Some("gif");
        }
        // WebP: RIFF....WEBP
        if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
            return Some("webp");
        }
        // AVIF: ....ftypavif
        if bytes.len() >= 12 && &bytes[4..8] == b"ftyp" && &bytes[8..12] == b"avif" {
            return Some("avif");
        }
        // SVG: <svg root, possibly after an <?xml declaration
        let start = String::from_utf8_lossy(&bytes[..bytes.len().min(512)]);
        let start_trim = start.trim_start();
        if start_trim.starts_with("<svg")
            || (start_trim.starts_with("<?xml") && start.contains("<svg"))
        {
            return Some("svg");
        }
        // ICO: 00 00 01 00
        if bytes.starts_with(&[0x00, 0x00, 0x01, 0x00]) {
            return Some("ico");
        }
        // BMP: BM
        if bytes.starts_with(b"BM") {
            return Some("bmp");
        }
    }

    // No known magic bytes: only trust the server if it says image/*
    if !content_type.starts_with("image/") {
        return None;
    }

    // Fallback: try to get extension from URL
    let url_lower = url.to_lowercase();
    if let Some(ext_start) = url_lower.rfind('.') {
        let ext = &url_lower[ext_start + 1..];
        // Remove query parameters
        let ext = ext.split('?').next().unwrap_or(ext);
        let ext = ext.split('#').next().unwrap_or(ext);

        match ext {
            "png" => return Some("png"),
            "jpg" | "jpeg" => return Some("jpg"),
            "gif" => return Some("gif"),
            "webp" => return Some("webp"),
            "svg" => return Some("svg"),
            "ico" => return Some("ico"),
            "bmp" => return Some("bmp"),
            "avif" => return Some("avif"),
            _ => {}
        }
    }

    // Map the image/* content-type subtype
    match content_type.trim() {
        "image/png" => Some("png"),
        "image/jpeg" | "image/jpg" => Some("jpg"),
        "image/gif" => Some("gif"),
        "image/webp" => Some("webp"),
        "image/svg+xml" => Some("svg"),
        "image/x-icon" | "image/vnd.microsoft.icon" => Some("ico"),
        "image/bmp" => Some("bmp"),
        "image/avif" => Some("avif"),
        // Server asserts it's an image but we can't name the format
        _ => Some("png"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc32_hash() {
        let hash1 = crc32_hash("https://example.com/image.png");
        let hash2 = crc32_hash("https://example.com/image.png");
        let hash3 = crc32_hash("https://example.com/other.png");

        assert_eq!(hash1, hash2, "Same input should produce same hash");
        assert_ne!(
            hash1, hash3,
            "Different input should produce different hash"
        );
    }

    #[test]
    fn test_detect_extension_from_magic_bytes() {
        // PNG magic bytes
        let png_bytes = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00];
        assert_eq!(
            detect_extension("http://example.com/image", "", &png_bytes),
            Some("png")
        );

        // JPEG magic bytes
        let jpg_bytes = [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46];
        assert_eq!(
            detect_extension("http://example.com/image", "", &jpg_bytes),
            Some("jpg")
        );

        // GIF magic bytes
        let gif_bytes = b"GIF89a\x00\x00";
        assert_eq!(
            detect_extension("http://example.com/image", "", gif_bytes),
            Some("gif")
        );
    }

    #[test]
    fn test_detect_extension_from_url_requires_image_content_type() {
        let empty: &[u8] = &[];
        // With an image/* content-type, the URL extension is trusted
        assert_eq!(
            detect_extension("https://example.com/image.png", "image/png", empty),
            Some("png")
        );
        assert_eq!(
            detect_extension("https://example.com/image.jpeg", "image/jpeg", empty),
            Some("jpg")
        );
        // With query parameters
        assert_eq!(
            detect_extension("https://example.com/image.png?v=123", "image/png", empty),
            Some("png")
        );
    }

    #[test]
    fn test_detect_extension_rejects_non_image() {
        // Regression: an HTML error page served with 200 was saved as ".png"
        let html_body = b"<!DOCTYPE html><html><body>Session expired</body></html>";
        assert_eq!(
            detect_extension(
                "https://example.com/image.png",
                "text/html; charset=utf-8",
                html_body
            ),
            None
        );
        // Unknown bytes without image content-type
        assert_eq!(
            detect_extension("https://example.com/image.png", "text/plain", b"hello"),
            None
        );
    }

    #[test]
    fn test_decode_html_entities_amp_last() {
        assert_eq!(
            decode_html_entities("https://cdn.example.com/c.png?w=800&amp;h=600"),
            "https://cdn.example.com/c.png?w=800&h=600"
        );
        // &amp;lt; must decode to &lt; (not <)
        assert_eq!(decode_html_entities("a&amp;lt;b"), "a&lt;b");
    }

    #[test]
    fn test_max_image_size_is_50mb() {
        assert_eq!(MAX_IMAGE_SIZE, 50 * 1024 * 1024);
    }

    #[test]
    fn test_image_downloader_creation() {
        let temp = tempfile::tempdir().unwrap();
        let downloader = ImageDownloader::new(temp.path());
        assert_eq!(downloader.stats(), (0, 0));
        assert_eq!(downloader.images_dir, temp.path().join("_remote_images"));
    }

    #[test]
    fn test_detect_extension_default() {
        let empty: &[u8] = &[];
        // Unknown extension with an asserted image content-type falls back to png
        assert_eq!(
            detect_extension("https://example.com/image", "image/tiff", empty),
            Some("png")
        );
        assert_eq!(
            detect_extension("https://example.com/image.xyz", "image/unknown", empty),
            Some("png")
        );
    }

    /// Test that Content-Length pre-check rejects images larger than MAX_IMAGE_SIZE.
    /// Uses a raw TCP server to send a response with a faked Content-Length header,
    /// since tiny_http auto-sets Content-Length to the actual body size.
    #[test]
    fn test_download_rejects_oversized_content_length() {
        use std::io::Write;

        let listener =
            std::net::TcpListener::bind("127.0.0.1:0").expect("Failed to bind test server");
        let port = listener.local_addr().unwrap().port();
        let url = format!("http://127.0.0.1:{}/large.png", port);

        // Spawn a thread that sends a raw HTTP response with inflated Content-Length
        let handle = std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                // Read the request (discard)
                let mut buf = [0u8; 1024];
                let _ = std::io::Read::read(&mut stream, &mut buf);

                // Send response with Content-Length > MAX_IMAGE_SIZE but tiny body
                let fake_len = MAX_IMAGE_SIZE + 1;
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: image/png\r\n\r\nfake",
                    fake_len
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            }
        });

        let temp = tempfile::tempdir().unwrap();
        let mut downloader = ImageDownloader::new(temp.path());
        let result = downloader.download_image(&url);

        assert!(
            result.is_err(),
            "Should reject image exceeding MAX_IMAGE_SIZE"
        );
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("too large"),
            "Error should mention 'too large', got: {}",
            err
        );

        handle.join().ok();
    }

    /// Test that streaming abort works when Content-Length is absent but body exceeds limit.
    /// Uses a local tiny_http server that streams data without Content-Length.
    #[test]
    fn test_download_aborts_oversized_stream() {
        use std::sync::Arc;

        let server = Arc::new(
            tiny_http::Server::http("127.0.0.1:0").expect("Failed to start test HTTP server"),
        );
        let addr = server.server_addr().to_ip().unwrap();
        let url = format!("http://127.0.0.1:{}/stream.bin", addr.port());

        // Spawn a thread that sends slightly more than MAX_IMAGE_SIZE via chunked transfer
        let server_clone = Arc::clone(&server);
        let handle = std::thread::spawn(move || {
            if let Ok(request) = server_clone.recv() {
                // Send a body slightly larger than MAX_IMAGE_SIZE using chunked encoding.
                // tiny_http doesn't support true streaming, so we create a large Vec.
                // We only need MAX_IMAGE_SIZE + 1 bytes to trigger the abort.
                let body = vec![0u8; MAX_IMAGE_SIZE + 8192];
                let response = tiny_http::Response::from_data(body).with_status_code(200);
                // The client may close the connection early — ignore the error.
                let _ = request.respond(response);
            }
        });

        let temp = tempfile::tempdir().unwrap();
        let mut downloader = ImageDownloader::new(temp.path());
        let result = downloader.download_image(&url);

        assert!(
            result.is_err(),
            "Should abort download when stream exceeds MAX_IMAGE_SIZE"
        );
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("too large") || err.contains("aborted"),
            "Error should mention 'too large' or 'aborted', got: {}",
            err
        );

        handle.join().ok();
    }

    /// Test that HTTP redirects are followed when downloading images.
    #[test]
    fn test_download_follows_redirect() {
        use std::sync::Arc;

        // Server 1: redirects to Server 2
        let server2 =
            Arc::new(tiny_http::Server::http("127.0.0.1:0").expect("Failed to start server2"));
        let addr2 = server2.server_addr().to_ip().unwrap();

        // Spawn server2: serves actual image data
        let server2_clone = Arc::clone(&server2);
        let handle2 = std::thread::spawn(move || {
            if let Ok(request) = server2_clone.recv() {
                // Return a minimal 1x1 PNG
                let png: &[u8] = &[
                    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG header
                    0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
                ];
                let response = tiny_http::Response::from_data(png.to_vec()).with_status_code(200);
                let _ = request.respond(response);
            }
        });

        // Server 1: issues a 301 redirect to server2
        let server1 =
            Arc::new(tiny_http::Server::http("127.0.0.1:0").expect("Failed to start server1"));
        let addr1 = server1.server_addr().to_ip().unwrap();
        let redirect_target = format!("http://127.0.0.1:{}/image.png", addr2.port());

        let server1_clone = Arc::clone(&server1);
        let handle1 = std::thread::spawn(move || {
            if let Ok(request) = server1_clone.recv() {
                let header =
                    tiny_http::Header::from_bytes(b"Location" as &[u8], redirect_target.as_bytes())
                        .unwrap();
                let response = tiny_http::Response::from_string("Moved")
                    .with_status_code(301)
                    .with_header(header);
                let _ = request.respond(response);
            }
        });

        let temp = tempfile::tempdir().unwrap();
        let mut downloader = ImageDownloader::new(temp.path());
        let url = format!("http://127.0.0.1:{}/old.png", addr1.port());
        let result = downloader.download_image(&url);

        // reqwest follows redirects by default, so this should succeed
        assert!(
            result.is_ok(),
            "Redirect should be followed, got: {:?}",
            result.err()
        );

        handle1.join().ok();
        handle2.join().ok();
    }
}
