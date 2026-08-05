//! Integration tests for guidebook build process
//!
//! Tests the full build pipeline by creating a book structure in a temp directory,
//! running the build process, and verifying the output.

use std::fs;
use std::process::{Command, Stdio};
use tempfile::tempdir;

/// Get the path to the guidebook binary built by cargo
fn guidebook_bin() -> String {
    // cargo test builds the binary in target/debug/
    let mut path = std::env::current_exe().unwrap();
    // Walk up from the test binary to the target directory
    path.pop(); // remove test binary name
    path.pop(); // remove deps/
    path.push("guidebook");
    path.to_string_lossy().to_string()
}

/// Create a minimal book structure for testing
fn create_test_book(dir: &std::path::Path) {
    // book.json
    fs::write(
        dir.join("book.json"),
        r#"{
    "title": "Test Book",
    "description": "A test book",
    "author": "Test Author"
}"#,
    )
    .unwrap();

    // SUMMARY.md
    fs::write(
        dir.join("SUMMARY.md"),
        r#"# Summary

* [Introduction](README.md)
* [Chapter 1](chapter1.md)
* [Chapter 2](chapter2/README.md)
"#,
    )
    .unwrap();

    // README.md
    fs::write(
        dir.join("README.md"),
        r#"# Test Book

Welcome to the test book!

This has a [link to chapter 1](chapter1.md).
"#,
    )
    .unwrap();

    // chapter1.md
    fs::write(
        dir.join("chapter1.md"),
        r#"# Chapter 1

This is chapter 1 content.

## Section 1.1

Some details here.

```rust
fn main() {
    println!("Hello, world!");
}
```
"#,
    )
    .unwrap();

    // chapter2/README.md
    fs::create_dir_all(dir.join("chapter2")).unwrap();
    fs::write(
        dir.join("chapter2/README.md"),
        r#"# Chapter 2

Nested chapter content.
"#,
    )
    .unwrap();

    // assets directory with a test file
    fs::create_dir_all(dir.join("assets")).unwrap();
    fs::write(dir.join("assets/test.txt"), "test asset content").unwrap();
}

#[test]
fn test_full_build() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("book");
    let output = temp.path().join("output");
    fs::create_dir_all(&source).unwrap();

    create_test_book(&source);

    let status = Command::new(guidebook_bin())
        .arg("build")
        .arg(source.to_str().unwrap())
        .arg("-o")
        .arg(output.to_str().unwrap())
        .status()
        .expect("Failed to execute guidebook build");

    assert!(status.success(), "Build should succeed");

    // Verify index.html was generated
    let index = output.join("index.html");
    assert!(index.exists(), "index.html should be generated");
    let index_content = fs::read_to_string(&index).unwrap();
    assert!(
        index_content.contains("Test Book"),
        "index.html should contain the book title"
    );

    // Verify chapter1.html was generated
    let ch1 = output.join("chapter1.html");
    assert!(ch1.exists(), "chapter1.html should be generated");
    let ch1_content = fs::read_to_string(&ch1).unwrap();
    assert!(
        ch1_content.contains("Chapter 1"),
        "chapter1.html should contain the chapter title"
    );

    // Verify nested chapter was generated (README.md → README.html)
    let ch2 = output.join("chapter2/README.html");
    assert!(ch2.exists(), "chapter2/README.html should be generated");

    // Verify static assets (gitbook CSS/JS)
    let css = output.join("gitbook/gitbook.css");
    assert!(css.exists(), "gitbook.css should be generated");

    let js = output.join("gitbook/gitbook.js");
    assert!(js.exists(), "gitbook.js should be generated");

    // Verify search index
    let search_index = output.join("search_index.json");
    assert!(
        search_index.exists(),
        "search_index.json should be generated"
    );
    let search_content = fs::read_to_string(&search_index).unwrap();
    let search_json: serde_json::Value = serde_json::from_str(&search_content).unwrap();
    assert!(
        search_json.is_array(),
        "Search index should be a JSON array"
    );
    assert!(
        search_json.as_array().unwrap().len() >= 2,
        "Search index should have entries for chapters"
    );

    // Verify assets were copied
    let asset = output.join("assets/test.txt");
    assert!(
        asset.exists() || output.join("assets").join("test.txt").read_link().is_ok(),
        "Asset file should be copied or symlinked"
    );
}

#[test]
fn test_build_generates_valid_html() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("book");
    let output = temp.path().join("output");
    fs::create_dir_all(&source).unwrap();

    create_test_book(&source);

    let status = Command::new(guidebook_bin())
        .arg("build")
        .arg(source.to_str().unwrap())
        .arg("-o")
        .arg(output.to_str().unwrap())
        .status()
        .expect("Failed to execute guidebook build");

    assert!(status.success());

    // Verify HTML structure
    let index = fs::read_to_string(output.join("index.html")).unwrap();
    assert!(
        index.contains("<!DOCTYPE html>") || index.contains("<!DOCTYPE HTML>"),
        "Should have DOCTYPE declaration"
    );
    assert!(index.contains("<html"), "Should have html tag");
    assert!(index.contains("</html>"), "Should have closing html tag");
    assert!(
        index.contains("<head>") || index.contains("<head "),
        "Should have head tag"
    );
    assert!(index.contains("<body"), "Should have body tag");
    assert!(index.contains("</body>"), "Should have closing body tag");
}

#[test]
fn test_build_with_code_blocks() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("book");
    let output = temp.path().join("output");
    fs::create_dir_all(&source).unwrap();

    create_test_book(&source);

    let status = Command::new(guidebook_bin())
        .arg("build")
        .arg(source.to_str().unwrap())
        .arg("-o")
        .arg(output.to_str().unwrap())
        .status()
        .expect("Failed to execute guidebook build");

    assert!(status.success());

    let ch1 = fs::read_to_string(output.join("chapter1.html")).unwrap();
    // Code blocks should be wrapped in <pre><code>
    assert!(
        ch1.contains("<pre>") || ch1.contains("<code"),
        "Code blocks should be rendered as <pre><code>"
    );
}

#[test]
fn test_init_command() {
    let temp = tempdir().unwrap();
    let book_dir = temp.path().join("new-book");

    let status = Command::new(guidebook_bin())
        .arg("init")
        .arg(book_dir.to_str().unwrap())
        .status()
        .expect("Failed to execute guidebook init");

    assert!(status.success(), "Init should succeed");

    // Verify files were created
    assert!(
        book_dir.join("README.md").exists(),
        "README.md should be created"
    );
    assert!(
        book_dir.join("SUMMARY.md").exists(),
        "SUMMARY.md should be created"
    );
    assert!(
        book_dir.join("book.json").exists(),
        "book.json should be created"
    );

    // Verify the initialized book can be built
    let output = temp.path().join("output");
    let status = Command::new(guidebook_bin())
        .arg("build")
        .arg(book_dir.to_str().unwrap())
        .arg("-o")
        .arg(output.to_str().unwrap())
        .status()
        .expect("Failed to build initialized book");

    assert!(status.success(), "Building initialized book should succeed");
    assert!(
        output.join("index.html").exists(),
        "Built book should have index.html"
    );
}

#[test]
fn test_sidebar_navigation() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("book");
    let output = temp.path().join("output");
    fs::create_dir_all(&source).unwrap();

    create_test_book(&source);

    let status = Command::new(guidebook_bin())
        .arg("build")
        .arg(source.to_str().unwrap())
        .arg("-o")
        .arg(output.to_str().unwrap())
        .status()
        .expect("Failed to execute guidebook build");

    assert!(status.success());

    let index = fs::read_to_string(output.join("index.html")).unwrap();
    // Sidebar should contain links to chapters
    assert!(
        index.contains("chapter1"),
        "Sidebar should link to chapter1"
    );
    assert!(
        index.contains("Chapter 1"),
        "Sidebar should display chapter title"
    );
}

// ── Serve path traversal tests ──

/// Find an available port by binding to port 0 and reading the assigned port.
fn find_available_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

/// Start the serve command on a given port and return the child process
fn start_serve(source: &std::path::Path, port: u16) -> std::process::Child {
    Command::new(guidebook_bin())
        .arg("serve")
        .arg(source.to_str().unwrap())
        .arg("-p")
        .arg(port.to_string())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start guidebook serve")
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

#[test]
fn test_serve_rejects_path_traversal() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("book");
    fs::create_dir_all(&source).unwrap();
    create_test_book(&source);

    // Create a secret file outside the book output
    fs::write(temp.path().join("secret.txt"), "TOP SECRET").unwrap();

    let port = find_available_port();
    let mut child = start_serve(&source, port);

    if !wait_for_server(port) {
        child.kill().ok();
        panic!(
            "Server did not start on port {} within 20s — security tests cannot run",
            port
        );
    }

    let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    // Test various path traversal attempts
    let traversal_paths = vec![
        "/../secret.txt",
        "/..%2Fsecret.txt",
        "/%2e%2e/secret.txt",
        "/%2e%2e%2fsecret.txt",
        "/sub/../../secret.txt",
        "/..\\secret.txt",
    ];

    for path in &traversal_paths {
        let url = format!("http://127.0.0.1:{}{}", port, path);
        match client.get(&url).send() {
            Ok(resp) => {
                let status = resp.status().as_u16();
                let body = resp.text().unwrap_or_default();
                assert!(
                    status == 403 || status == 404,
                    "Path traversal '{}' should be blocked (got {} with body: {})",
                    path,
                    status,
                    &body[..body.len().min(100)]
                );
                assert!(
                    !body.contains("TOP SECRET"),
                    "Path traversal '{}' leaked secret content!",
                    path
                );
            }
            Err(_) => {
                // Connection error is acceptable (server might reject early)
            }
        }
    }

    // Verify normal access still works
    let resp = client
        .get(format!("http://127.0.0.1:{}/", port))
        .send()
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200, "Normal access should work");

    // Also test double-encoded traversal in the same server session
    let url = format!("http://127.0.0.1:{}/..%252F..%252Fetc/passwd", port);
    if let Ok(resp) = client.get(&url).send() {
        let status = resp.status().as_u16();
        assert!(
            status == 403 || status == 404,
            "Double-encoded traversal should be blocked (got {})",
            status
        );
    }

    child.kill().ok();
    child.wait().ok();
}

// ── Multi-language build test ──

/// Create a multi-language book structure for testing
fn create_multilang_book(dir: &std::path::Path) {
    // LANGS.md at root
    fs::write(
        dir.join("LANGS.md"),
        "* [English](en/)\n* [Japanese](ja/)\n",
    )
    .unwrap();

    // book.json
    fs::write(dir.join("book.json"), r#"{"title": "Multi-lang Book"}"#).unwrap();

    // English
    let en = dir.join("en");
    fs::create_dir_all(&en).unwrap();
    fs::write(en.join("README.md"), "# English Intro\nWelcome\n").unwrap();
    fs::write(
        en.join("SUMMARY.md"),
        "# Summary\n\n* [Introduction](README.md)\n* [Chapter 1](ch1.md)\n",
    )
    .unwrap();
    fs::write(en.join("ch1.md"), "# Chapter 1\nEnglish content\n").unwrap();

    // Japanese
    let ja = dir.join("ja");
    fs::create_dir_all(&ja).unwrap();
    fs::write(ja.join("README.md"), "# Japanese Intro\nようこそ\n").unwrap();
    fs::write(
        ja.join("SUMMARY.md"),
        "# Summary\n\n* [Introduction](README.md)\n* [Chapter 1](ch1.md)\n",
    )
    .unwrap();
    fs::write(ja.join("ch1.md"), "# Chapter 1\n日本語コンテンツ\n").unwrap();
}

#[test]
fn test_multilang_build() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("book");
    let output = temp.path().join("output");
    fs::create_dir_all(&source).unwrap();

    create_multilang_book(&source);

    let status = Command::new(guidebook_bin())
        .arg("build")
        .arg(source.to_str().unwrap())
        .arg("-o")
        .arg(output.to_str().unwrap())
        .status()
        .expect("Failed to execute guidebook build");

    assert!(status.success(), "Multi-language build should succeed");

    // Verify language index was generated
    let index = output.join("index.html");
    assert!(
        index.exists(),
        "Root index.html should be generated for language selection"
    );

    // Verify English output
    let en_index = output.join("en/index.html");
    assert!(en_index.exists(), "en/index.html should be generated");
    let en_content = fs::read_to_string(&en_index).unwrap();
    assert!(
        en_content.contains("English Intro") || en_content.contains("Welcome"),
        "English index should contain English content"
    );

    // Verify Japanese output
    let ja_index = output.join("ja/index.html");
    assert!(ja_index.exists(), "ja/index.html should be generated");
    let ja_content = fs::read_to_string(&ja_index).unwrap();
    assert!(
        ja_content.contains("Japanese Intro") || ja_content.contains("ようこそ"),
        "Japanese index should contain Japanese content"
    );

    // Verify chapter pages for both languages
    let en_ch1 = output.join("en/ch1.html");
    assert!(en_ch1.exists(), "en/ch1.html should be generated");

    let ja_ch1 = output.join("ja/ch1.html");
    assert!(ja_ch1.exists(), "ja/ch1.html should be generated");
}

#[test]
fn test_repeated_import_is_expanded_twice() {
    // Regression: the @import visited-set treated a second import of the
    // same file as "circular" and silently dropped it
    let temp = tempdir().unwrap();
    let source = temp.path().join("book");
    let output = temp.path().join("output");
    fs::create_dir_all(&source).unwrap();
    create_test_book(&source);

    fs::write(source.join("snippet.md"), "REUSABLE-NOTICE\n").unwrap();
    fs::write(
        source.join("chapter1.md"),
        "# Chapter 1\n\n<!-- @import(\"snippet.md\") -->\n\nmiddle\n\n<!-- @import(\"snippet.md\") -->\n",
    )
    .unwrap();

    let status = Command::new(guidebook_bin())
        .arg("build")
        .arg(source.to_str().unwrap())
        .arg("-o")
        .arg(output.to_str().unwrap())
        .status()
        .unwrap();
    assert!(status.success());

    let html = fs::read_to_string(output.join("chapter1.html")).unwrap();
    assert_eq!(
        html.matches("REUSABLE-NOTICE").count(),
        2,
        "both imports must be expanded: {}",
        html
    );
}

#[test]
fn test_orphan_files_removed_and_dotfiles_preserved() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("book");
    let output = temp.path().join("output");
    fs::create_dir_all(&source).unwrap();
    create_test_book(&source);

    // First build + plant an orphan and a dotfile
    let run = |arg_src: &str, arg_out: &str| {
        Command::new(guidebook_bin())
            .arg("build")
            .arg(arg_src)
            .arg("-o")
            .arg(arg_out)
            .status()
            .unwrap()
    };
    assert!(run(source.to_str().unwrap(), output.to_str().unwrap()).success());
    fs::write(output.join("orphan.html"), "stale page").unwrap();
    fs::write(output.join(".nojekyll"), "").unwrap();

    assert!(run(source.to_str().unwrap(), output.to_str().unwrap()).success());
    assert!(
        !output.join("orphan.html").exists(),
        "stale output must be cleaned"
    );
    assert!(
        output.join(".nojekyll").exists(),
        "dot entries must be preserved"
    );
    assert!(output.join("index.html").exists());
}

#[test]
fn test_changed_asset_is_refreshed_on_rebuild() {
    // Regression: any existing destination file was skipped, so edited
    // assets never reached the output on rebuild
    let temp = tempdir().unwrap();
    let source = temp.path().join("book");
    let output = temp.path().join("output");
    fs::create_dir_all(&source).unwrap();
    create_test_book(&source);

    let build = || {
        Command::new(guidebook_bin())
            .arg("build")
            .arg(source.to_str().unwrap())
            .arg("-o")
            .arg(output.to_str().unwrap())
            .status()
            .unwrap()
    };
    assert!(build().success());
    assert_eq!(
        fs::read_to_string(output.join("assets/test.txt")).unwrap(),
        "test asset content"
    );

    fs::write(source.join("assets/test.txt"), "UPDATED asset content!").unwrap();
    assert!(build().success());
    assert_eq!(
        fs::read_to_string(output.join("assets/test.txt")).unwrap(),
        "UPDATED asset content!",
        "changed assets must be refreshed"
    );
}

#[test]
fn test_output_is_self_contained_no_symlinks() {
    // Regression: assets were symlinked to absolute source paths, so a
    // deployed _book directory was full of dangling links
    let temp = tempdir().unwrap();
    let source = temp.path().join("book");
    let output = temp.path().join("output");
    fs::create_dir_all(&source).unwrap();
    create_test_book(&source);

    let status = Command::new(guidebook_bin())
        .arg("build")
        .arg(source.to_str().unwrap())
        .arg("-o")
        .arg(output.to_str().unwrap())
        .status()
        .unwrap();
    assert!(status.success());

    for entry in walkdir_all(&output) {
        let meta = fs::symlink_metadata(&entry).unwrap();
        assert!(
            !meta.file_type().is_symlink(),
            "output must not contain symlinks: {}",
            entry.display()
        );
    }
}

fn walkdir_all(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                out.extend(walkdir_all(&p));
            } else {
                out.push(p);
            }
        }
    }
    out
}

#[test]
fn test_favicon_files_emitted() {
    // Regression: templates linked gitbook/images/favicon.ico but the build
    // never wrote the file — every guidebook site had a 404 favicon
    let temp = tempdir().unwrap();
    let source = temp.path().join("book");
    let output = temp.path().join("output");
    fs::create_dir_all(&source).unwrap();
    create_test_book(&source);

    let status = Command::new(guidebook_bin())
        .arg("build")
        .arg(source.to_str().unwrap())
        .arg("-o")
        .arg(output.to_str().unwrap())
        .status()
        .unwrap();
    assert!(status.success());

    let favicon = output.join("gitbook/images/favicon.ico");
    assert!(favicon.exists(), "favicon.ico must be emitted");
    assert!(fs::metadata(&favicon).unwrap().len() > 0);
    assert!(output
        .join("gitbook/images/apple-touch-icon-precomposed-152.png")
        .exists());

    // Pages must link it with the depth-aware root path
    let index = fs::read_to_string(output.join("index.html")).unwrap();
    assert!(
        index.contains("gitbook/images/favicon.ico"),
        "pages must link the favicon"
    );
}

#[test]
fn test_heading_anchors_shipped_and_headings_have_ids() {
    // Heading anchor links (hover a heading -> link icon -> click copies a #id URL)
    // only work if three things hold together: headings carry ids, the JS that
    // injects the icons ships and is wired up (also after SPA navigation), and the
    // CSS that reveals them on hover ships too. Dropping any one of them silently
    // breaks sharing links to a section.
    let temp = tempdir().unwrap();
    let source = temp.path().join("book");
    let output = temp.path().join("output");
    fs::create_dir_all(&source).unwrap();
    create_test_book(&source);

    let status = Command::new(guidebook_bin())
        .arg("build")
        .arg(source.to_str().unwrap())
        .arg("-o")
        .arg(output.to_str().unwrap())
        .status()
        .unwrap();
    assert!(status.success());

    // Headings the anchors attach to must carry ids
    let ch1 = fs::read_to_string(output.join("chapter1.html")).unwrap();
    assert!(
        ch1.contains(r#"<h2 id="section-11">"#),
        "headings must carry ids for anchor links to target, got: {}",
        ch1
    );

    let js = fs::read_to_string(output.join("gitbook/gitbook.js")).unwrap();
    assert!(
        js.contains("function setupHeadingAnchors()"),
        "gitbook.js must ship the heading anchor injector"
    );
    // Called once on load and again after SPA navigation replaces the content
    assert!(
        js.matches("setupHeadingAnchors();").count() >= 2,
        "setupHeadingAnchors must run on load and after SPA navigation"
    );
    assert!(
        js.contains("copyTextToClipboard"),
        "clicking an anchor must copy the shareable URL"
    );

    let css = fs::read_to_string(output.join("gitbook/gitbook.css")).unwrap();
    assert!(
        css.contains(".heading-anchor"),
        "gitbook.css must ship the heading anchor styles"
    );
    assert!(
        css.contains("h2:hover .heading-anchor"),
        "the icon must be revealed on heading hover"
    );
}

#[test]
fn test_search_index_handles_anchors_and_front_matter() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("book");
    let output = temp.path().join("output");
    fs::create_dir_all(&source).unwrap();
    create_test_book(&source);

    fs::write(
        source.join("SUMMARY.md"),
        "# Summary\n\n* [Introduction](README.md)\n* [Anchored](chapter1.md#setup)\n",
    )
    .unwrap();
    fs::write(
        source.join("chapter1.md"),
        "---\ntitle: FM Title\ndescription: SECRETMETA\n---\n# Setup\n\nSearchable body text.\n",
    )
    .unwrap();

    let status = Command::new(guidebook_bin())
        .arg("build")
        .arg(source.to_str().unwrap())
        .arg("-o")
        .arg(output.to_str().unwrap())
        .status()
        .unwrap();
    assert!(status.success());

    let idx = fs::read_to_string(output.join("search_index.json")).unwrap();
    // Regression: anchor paths silently dropped the page from the index
    assert!(
        idx.contains("Searchable body text"),
        "anchored page must be indexed: {}",
        idx
    );
    assert!(
        idx.contains("chapter1.html"),
        "index path must be the html file: {}",
        idx
    );
    // Regression: raw front matter leaked into the index content
    assert!(
        !idx.contains("SECRETMETA"),
        "front matter must not be indexed: {}",
        idx
    );
}

#[test]
fn test_non_utf8_page_does_not_abort_build() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("book");
    let output = temp.path().join("output");
    fs::create_dir_all(&source).unwrap();
    create_test_book(&source);

    // chapter1.md with an invalid UTF-8 byte sequence
    let mut bytes = b"# Chapter 1\n\nvalid text ".to_vec();
    bytes.extend_from_slice(&[0xFF, 0xFE]);
    bytes.extend_from_slice(b" more text\n");
    fs::write(source.join("chapter1.md"), bytes).unwrap();

    let status = Command::new(guidebook_bin())
        .arg("build")
        .arg(source.to_str().unwrap())
        .arg("-o")
        .arg(output.to_str().unwrap())
        .status()
        .unwrap();
    assert!(
        status.success(),
        "a single non-UTF-8 page must not abort the whole build"
    );
    let html = fs::read_to_string(output.join("chapter1.html")).unwrap();
    assert!(html.contains("more text"));
}

#[test]
fn test_output_inside_source_does_not_snowball() {
    // Regression: `build . -o out` re-copied out/assets into out/out/assets
    // on every rebuild, one level deeper each time
    let temp = tempdir().unwrap();
    let source = temp.path().join("book");
    fs::create_dir_all(&source).unwrap();
    create_test_book(&source);
    let output = source.join("out");

    let build = || {
        Command::new(guidebook_bin())
            .arg("build")
            .arg(source.to_str().unwrap())
            .arg("-o")
            .arg(output.to_str().unwrap())
            .status()
            .unwrap()
    };
    assert!(build().success());
    assert!(build().success());
    assert!(build().success());

    assert!(output.join("assets/test.txt").exists());
    assert!(
        !output.join("out").exists(),
        "output dir must not be copied into itself"
    );
}
