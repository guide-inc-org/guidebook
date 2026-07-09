use anyhow::Result;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Language {
    pub code: String,
    pub title: String,
}

/// Parse LANGS.md to get available languages
/// Format:
/// * [Japanese](jp/)
/// * [Vietnamese](vn/)
pub fn parse_langs(book_dir: &Path) -> Result<Vec<Language>> {
    let langs_path = book_dir.join("LANGS.md");

    if !langs_path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&langs_path)?;
    let mut languages = Vec::new();

    for line in content.lines() {
        let line = line.trim();

        // Match pattern: * [Title](code/) or - [Title](code/)
        if (line.starts_with('*') || line.starts_with('-'))
            && line.contains('[')
            && line.contains("](")
        {
            if let Some(lang) = parse_lang_line(line) {
                languages.push(lang);
            }
        }
    }

    Ok(languages)
}

fn parse_lang_line(line: &str) -> Option<Language> {
    // Parse strictly left-to-right so malformed lines (missing bracket,
    // ')' appearing before "](", etc.) return None instead of panicking
    // on an inverted slice range
    let title_start = line.find('[')? + 1;
    let rest = &line[title_start..];

    // Title runs up to the "](" separator
    let title_len = rest.find("](")?;
    let title = rest[..title_len].to_string();

    // Code runs from after "](" to the next ')'
    let after_sep = &rest[title_len + 2..];
    let code_len = after_sep.find(')')?;
    let mut code = after_sep[..code_len].to_string();

    // Remove trailing slash if present
    if code.ends_with('/') {
        code.pop();
    }

    Some(Language { code, title })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lang_line() {
        let lang = parse_lang_line("* [Japanese](jp/)").unwrap();
        assert_eq!(lang.code, "jp");
        assert_eq!(lang.title, "Japanese");

        let lang = parse_lang_line("- [Vietnamese](vn/)").unwrap();
        assert_eq!(lang.code, "vn");
        assert_eq!(lang.title, "Vietnamese");
    }

    #[test]
    fn test_parse_lang_line_malformed_no_panic() {
        // Regression: inverted slice ranges panicked on malformed lines
        let malformed = [
            "* [Japanese](jp/",  // missing closing paren
            "* [Japanese(jp/)",  // missing closing bracket
            "* Japanese](jp/)",  // missing opening bracket (has "](")
            "* [Japanese])(jp/", // ')' before "]("
            "* )[Japanese](",    // ')' before, nothing after "]("
            "* [](",             // empty everything
        ];
        for line in malformed {
            // Must not panic; None or a best-effort parse are both acceptable
            let _ = parse_lang_line(line);
        }
    }

    #[test]
    fn test_parse_langs_skips_malformed_line() {
        // A malformed line must not break parsing of valid lines around it
        use std::io::Write;
        let dir = std::env::temp_dir().join("gb_langs_test");
        std::fs::create_dir_all(&dir).unwrap();
        let mut f = std::fs::File::create(dir.join("LANGS.md")).unwrap();
        writeln!(f, "* [Japanese](jp/)").unwrap();
        writeln!(f, "* [Broken](oops/").unwrap();
        writeln!(f, "* [Vietnamese](vn/)").unwrap();
        drop(f);

        let langs = parse_langs(&dir).unwrap();
        let codes: Vec<&str> = langs.iter().map(|l| l.code.as_str()).collect();
        assert!(codes.contains(&"jp"));
        assert!(codes.contains(&"vn"));
        std::fs::remove_dir_all(&dir).ok();
    }
}
