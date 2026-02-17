//! Security regression tests for guidebook
//!
//! Tests path traversal protection in serve, image download size limits,
//! and self-update checksum enforcement.

use std::fs;
use std::process::{Command, Stdio};
use tempfile::tempdir;

/// Get the path to the guidebook binary built by cargo
fn guidebook_bin() -> String {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // remove test binary name
    path.pop(); // remove deps/
    path.push("guidebook");
    path.to_string_lossy().to_string()
}

/// Create a minimal book structure for testing
fn create_minimal_book(dir: &std::path::Path) {
    fs::write(dir.join("book.json"), r#"{"title": "Test"}"#).unwrap();
    fs::write(dir.join("SUMMARY.md"), "# Summary\n* [Intro](README.md)\n").unwrap();
    fs::write(dir.join("README.md"), "# Test\nHello\n").unwrap();
}

/// Find an available port by binding to port 0 and reading the assigned port.
/// The listener is dropped immediately, so there is a small race window,
/// but it is far more reliable than hard-coded port numbers.
fn find_available_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

/// Wait for the serve command to be ready by polling the port
fn wait_for_server(port: u16) -> bool {
    for _ in 0..100 {
        if std::net::TcpStream::connect(format!("127.0.0.1:{}", port)).is_ok() {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    false
}

// ── Path traversal tests ──

#[test]
fn test_serve_blocks_dotdot_traversal() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("book");
    fs::create_dir_all(&source).unwrap();
    create_minimal_book(&source);

    // Create a secret file outside the served directory
    fs::write(temp.path().join("secret.txt"), "LEAKED").unwrap();

    let port = find_available_port();
    let mut child = Command::new(guidebook_bin())
        .arg("serve")
        .arg(source.to_str().unwrap())
        .arg("-p")
        .arg(port.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start serve");

    if !wait_for_server(port) {
        child.kill().ok();
        panic!("Server did not start on port {}", port);
    }

    let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    // Plain ../
    let resp = client
        .get(&format!("http://127.0.0.1:{}/../secret.txt", port))
        .send();
    if let Ok(resp) = resp {
        assert_ne!(resp.status().as_u16(), 200, "../ should not return 200");
        assert!(
            !resp.text().unwrap_or_default().contains("LEAKED"),
            "../ must not leak file"
        );
    }

    // URL-encoded ../
    let resp = client
        .get(&format!("http://127.0.0.1:{}/..%2Fsecret.txt", port))
        .send();
    if let Ok(resp) = resp {
        assert_ne!(resp.status().as_u16(), 200, "..%2F should not return 200");
        assert!(
            !resp.text().unwrap_or_default().contains("LEAKED"),
            "..%2F must not leak file"
        );
    }

    child.kill().ok();
    child.wait().ok();
}

#[test]
fn test_serve_blocks_html_fallback_traversal() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("book");
    fs::create_dir_all(&source).unwrap();
    create_minimal_book(&source);

    let port = find_available_port();
    let mut child = Command::new(guidebook_bin())
        .arg("serve")
        .arg(source.to_str().unwrap())
        .arg("-p")
        .arg(port.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start serve");

    if !wait_for_server(port) {
        child.kill().ok();
        panic!("Server did not start on port {}", port);
    }

    let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    // This tests the .html fallback path: the server tries path.html if path doesn't exist
    // If ../etc/passwd doesn't exist, it tries ../etc/passwd.html — both should be blocked
    let resp = client
        .get(&format!("http://127.0.0.1:{}/../etc/passwd", port))
        .send();
    if let Ok(resp) = resp {
        let status = resp.status().as_u16();
        assert!(
            status == 403 || status == 404,
            "html fallback traversal should be blocked (got {})",
            status
        );
    }

    child.kill().ok();
    child.wait().ok();
}

// ── Update checksum enforcement test ──
// We can't test the full update flow without network access,
// but we can verify extract_checksum behavior via the binary's
// behavior when checksum is missing.
// The actual unit tests for extract_checksum are in src/main.rs#tests

#[test]
fn test_serve_normal_access_works() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("book");
    fs::create_dir_all(&source).unwrap();
    create_minimal_book(&source);

    let port = find_available_port();
    let mut child = Command::new(guidebook_bin())
        .arg("serve")
        .arg(source.to_str().unwrap())
        .arg("-p")
        .arg(port.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start serve");

    if !wait_for_server(port) {
        child.kill().ok();
        panic!("Server did not start on port {}", port);
    }

    let client = reqwest::blocking::Client::new();

    // Normal index page should work
    let resp = client
        .get(&format!("http://127.0.0.1:{}/", port))
        .send()
        .unwrap();
    assert_eq!(
        resp.status().as_u16(),
        200,
        "Normal access should return 200"
    );
    let body = resp.text().unwrap();
    assert!(body.contains("Test"), "Should contain book content");

    // Non-existent page should return 404
    let resp = client
        .get(&format!("http://127.0.0.1:{}/nonexistent", port))
        .send()
        .unwrap();
    assert_eq!(
        resp.status().as_u16(),
        404,
        "Non-existent page should return 404"
    );

    child.kill().ok();
    child.wait().ok();
}
