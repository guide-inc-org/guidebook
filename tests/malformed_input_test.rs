//! Malformed input tests for guidebook
//!
//! Tests that guidebook handles invalid/corrupted input gracefully
//! without panicking or producing nonsensical output.

use std::fs;
use std::process::Command;
use tempfile::tempdir;

/// Get the path to the guidebook binary built by cargo
fn guidebook_bin() -> String {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // remove test binary name
    path.pop(); // remove deps/
    path.push("guidebook");
    path.to_string_lossy().to_string()
}

#[test]
fn test_invalid_book_json() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("book");
    let output = temp.path().join("output");
    fs::create_dir_all(&source).unwrap();

    // Write invalid JSON
    fs::write(source.join("book.json"), "{ invalid json }}}").unwrap();
    fs::write(
        source.join("SUMMARY.md"),
        "# Summary\n* [Intro](README.md)\n",
    )
    .unwrap();
    fs::write(source.join("README.md"), "# Hello\n").unwrap();

    let result = Command::new(guidebook_bin())
        .arg("build")
        .arg(source.to_str().unwrap())
        .arg("-o")
        .arg(output.to_str().unwrap())
        .output()
        .expect("Failed to execute guidebook build");

    // Should handle gracefully (either succeed with defaults or fail with clear error)
    // The important thing is it doesn't panic
    let stderr = String::from_utf8_lossy(&result.stderr);
    if !result.status.success() {
        assert!(
            !stderr.contains("panicked"),
            "Should not panic on invalid book.json"
        );
    }
}

#[test]
fn test_empty_summary() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("book");
    let output = temp.path().join("output");
    fs::create_dir_all(&source).unwrap();

    fs::write(source.join("book.json"), r#"{"title": "Test"}"#).unwrap();
    fs::write(source.join("SUMMARY.md"), "").unwrap();
    fs::write(source.join("README.md"), "# Hello\n").unwrap();

    let result = Command::new(guidebook_bin())
        .arg("build")
        .arg(source.to_str().unwrap())
        .arg("-o")
        .arg(output.to_str().unwrap())
        .output()
        .expect("Failed to execute guidebook build");

    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        !stderr.contains("panicked"),
        "Should not panic on empty SUMMARY.md"
    );
}

#[test]
fn test_missing_summary() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("book");
    let output = temp.path().join("output");
    fs::create_dir_all(&source).unwrap();

    fs::write(source.join("book.json"), r#"{"title": "Test"}"#).unwrap();
    // No SUMMARY.md
    fs::write(source.join("README.md"), "# Hello\n").unwrap();

    let result = Command::new(guidebook_bin())
        .arg("build")
        .arg(source.to_str().unwrap())
        .arg("-o")
        .arg(output.to_str().unwrap())
        .output()
        .expect("Failed to execute guidebook build");

    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        !stderr.contains("panicked"),
        "Should not panic on missing SUMMARY.md"
    );
}

#[test]
fn test_summary_with_missing_files() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("book");
    let output = temp.path().join("output");
    fs::create_dir_all(&source).unwrap();

    fs::write(source.join("book.json"), r#"{"title": "Test"}"#).unwrap();
    fs::write(
        source.join("SUMMARY.md"),
        r#"# Summary

* [Intro](README.md)
* [Missing](does-not-exist.md)
* [Also Missing](another/missing.md)
"#,
    )
    .unwrap();
    fs::write(source.join("README.md"), "# Hello\n").unwrap();

    let result = Command::new(guidebook_bin())
        .arg("build")
        .arg(source.to_str().unwrap())
        .arg("-o")
        .arg(output.to_str().unwrap())
        .output()
        .expect("Failed to execute guidebook build");

    // Should succeed (with warnings about missing files) but not panic
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        !stderr.contains("panicked"),
        "Should not panic on missing chapter files"
    );
    // The build should still succeed for existing files
    assert!(
        result.status.success(),
        "Build should succeed even with missing files"
    );
}

#[test]
fn test_malformed_markdown() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("book");
    let output = temp.path().join("output");
    fs::create_dir_all(&source).unwrap();

    fs::write(source.join("book.json"), r#"{"title": "Test"}"#).unwrap();
    fs::write(
        source.join("SUMMARY.md"),
        "# Summary\n* [Intro](README.md)\n",
    )
    .unwrap();

    // Write markdown with edge cases
    fs::write(
        source.join("README.md"),
        r#"# Malformed Content

## Unclosed code block
```
fn never_closed() {

## Deeply nested headers
###### Level 6
####### Level 7 (invalid)

## Broken links
[link with no url]()
[link to nowhere](javascript:alert(1))
![image with no src]()

## Mixed content
<div>Raw HTML that might break things</div>

## Unicode extremes
🎉🎊🎈 Emoji headers 🎉🎊🎈

## Empty sections

---

---

---

"#,
    )
    .unwrap();

    let result = Command::new(guidebook_bin())
        .arg("build")
        .arg(source.to_str().unwrap())
        .arg("-o")
        .arg(output.to_str().unwrap())
        .output()
        .expect("Failed to execute guidebook build");

    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        !stderr.contains("panicked"),
        "Should not panic on malformed markdown"
    );
    assert!(
        result.status.success(),
        "Build should succeed with malformed markdown"
    );
}

#[test]
fn test_empty_book_json() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("book");
    let output = temp.path().join("output");
    fs::create_dir_all(&source).unwrap();

    // Empty book.json (valid JSON but empty object)
    fs::write(source.join("book.json"), "{}").unwrap();
    fs::write(
        source.join("SUMMARY.md"),
        "# Summary\n* [Intro](README.md)\n",
    )
    .unwrap();
    fs::write(source.join("README.md"), "# Hello\n").unwrap();

    let result = Command::new(guidebook_bin())
        .arg("build")
        .arg(source.to_str().unwrap())
        .arg("-o")
        .arg(output.to_str().unwrap())
        .output()
        .expect("Failed to execute guidebook build");

    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        !stderr.contains("panicked"),
        "Should not panic on empty book.json"
    );
    assert!(
        result.status.success(),
        "Build should succeed with empty book.json"
    );
}

#[test]
fn test_binary_content_in_markdown() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("book");
    let output = temp.path().join("output");
    fs::create_dir_all(&source).unwrap();

    fs::write(source.join("book.json"), r#"{"title": "Test"}"#).unwrap();
    fs::write(
        source.join("SUMMARY.md"),
        "# Summary\n* [Intro](README.md)\n",
    )
    .unwrap();

    // Write some binary-like content mixed with markdown
    let mut content = b"# Title\n\nSome text\n\n".to_vec();
    content.extend_from_slice(&[0x00, 0xFF, 0xFE, 0x89, 0x50, 0x4E, 0x47]);
    content.extend_from_slice(b"\n\nMore text after binary\n");
    fs::write(source.join("README.md"), content).unwrap();

    let result = Command::new(guidebook_bin())
        .arg("build")
        .arg(source.to_str().unwrap())
        .arg("-o")
        .arg(output.to_str().unwrap())
        .output()
        .expect("Failed to execute guidebook build");

    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        !stderr.contains("panicked"),
        "Should not panic on binary content in markdown"
    );
}

#[test]
fn test_nonexistent_source_directory() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("does-not-exist");
    let output = temp.path().join("output");

    let result = Command::new(guidebook_bin())
        .arg("build")
        .arg(source.to_str().unwrap())
        .arg("-o")
        .arg(output.to_str().unwrap())
        .output()
        .expect("Failed to execute guidebook build");

    assert!(
        !result.status.success(),
        "Build should fail for nonexistent source"
    );
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        !stderr.contains("panicked"),
        "Should not panic on nonexistent source"
    );
}

#[test]
fn test_large_summary() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("book");
    let output = temp.path().join("output");
    fs::create_dir_all(&source).unwrap();

    fs::write(source.join("book.json"), r#"{"title": "Test"}"#).unwrap();
    fs::write(source.join("README.md"), "# Hello\n").unwrap();

    // Create a SUMMARY with many entries
    let mut summary = String::from("# Summary\n\n* [Intro](README.md)\n");
    for i in 0..50 {
        let filename = format!("chapter_{}.md", i);
        summary.push_str(&format!("* [Chapter {}]({})\n", i, filename));
        // Only create a few files to test the "missing file" handling
        if i < 5 {
            fs::write(
                source.join(&filename),
                format!("# Chapter {}\nContent\n", i),
            )
            .unwrap();
        }
    }
    fs::write(source.join("SUMMARY.md"), summary).unwrap();

    let result = Command::new(guidebook_bin())
        .arg("build")
        .arg(source.to_str().unwrap())
        .arg("-o")
        .arg(output.to_str().unwrap())
        .output()
        .expect("Failed to execute guidebook build");

    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        !stderr.contains("panicked"),
        "Should not panic on large summary with missing files"
    );
    assert!(result.status.success(), "Build should succeed");
}
