//! Integration tests for guidebook build process
//!
//! Tests the full build pipeline by creating a book structure in a temp directory,
//! running the build process, and verifying the output.

use std::fs;
use std::process::Command;
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
