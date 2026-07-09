//! OpenAPI/Swagger UI generation module
//!
//! Generates Swagger UI pages for API documentation when `openapi` is configured in book.json

use crate::parser::OpenApiConfig;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Swagger UI version to use from CDN
const SWAGGER_UI_VERSION: &str = "5.11.0";

/// Generate Swagger UI pages based on OpenAPI configuration
pub fn generate_swagger_ui(source: &Path, output: &Path, config: &OpenApiConfig) -> Result<()> {
    println!("Generating Swagger UI...");

    match config {
        OpenApiConfig::Single(path) => {
            // Single file -> api-docs/
            generate_single_swagger_ui(source, output, "api-docs", path)?;
        }
        OpenApiConfig::Multiple(map) => {
            // Multiple files -> each output directory
            for (output_dir, swagger_path) in map {
                generate_single_swagger_ui(source, output, output_dir, swagger_path)?;
            }
        }
    }

    Ok(())
}

/// Reject config-supplied path fragments that would escape the base
/// directory. `Path::join` REPLACES the base when given an absolute path, and
/// ".." components walk out of it — both would let a book.json read or write
/// arbitrary filesystem locations.
fn validate_contained_path(value: &str, what: &str) -> Result<()> {
    let p = Path::new(value);
    if p.is_absolute() {
        anyhow::bail!("{} must be a relative path, got: {}", what, value);
    }
    if p.components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        anyhow::bail!("{} must not contain '..', got: {}", what, value);
    }
    Ok(())
}

/// Generate a single Swagger UI page
fn generate_single_swagger_ui(
    source: &Path,
    output: &Path,
    output_dir: &str,
    openapi_path: &str,
) -> Result<()> {
    validate_contained_path(output_dir, "openapi output directory")?;
    validate_contained_path(openapi_path, "openapi spec path")?;

    // Create output directory
    let api_docs_dir = output.join(output_dir);
    fs::create_dir_all(&api_docs_dir)?;

    // Copy swagger.json to output
    let src_swagger = source.join(openapi_path);
    if !src_swagger.exists() {
        anyhow::bail!("OpenAPI file not found: {}", src_swagger.display());
    }

    // Determine output filename (keep original extension)
    let swagger_filename = Path::new(openapi_path)
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or("swagger.json");

    let dest_swagger = api_docs_dir.join(swagger_filename);
    fs::copy(&src_swagger, &dest_swagger)
        .with_context(|| format!("Failed to copy {} to {}/", openapi_path, output_dir))?;

    println!(
        "  Copied {} to {}/{}",
        openapi_path, output_dir, swagger_filename
    );

    // Generate index.html with Swagger UI
    let html = generate_swagger_html(swagger_filename);
    fs::write(api_docs_dir.join("index.html"), html)?;

    println!("  Generated {}/index.html", output_dir);

    Ok(())
}

/// Generate HTML page with Swagger UI
fn generate_swagger_html(swagger_filename: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>API Documentation</title>
    <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@{version}/swagger-ui.css">
    <style>
        html {{
            box-sizing: border-box;
            overflow-y: scroll;
        }}
        *, *::before, *::after {{
            box-sizing: inherit;
        }}
        body {{
            margin: 0;
            background: #fafafa;
        }}
    </style>
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@{version}/swagger-ui-bundle.js"></script>
    <script src="https://unpkg.com/swagger-ui-dist@{version}/swagger-ui-standalone-preset.js"></script>
    <script>
        window.onload = function() {{
            window.ui = SwaggerUIBundle({{
                url: "./{spec_file}",
                dom_id: '#swagger-ui',
                deepLinking: true,
                presets: [
                    SwaggerUIBundle.presets.apis,
                    SwaggerUIStandalonePreset
                ],
                plugins: [
                    SwaggerUIBundle.plugins.DownloadUrl
                ],
                layout: "StandaloneLayout"
            }});
        }};
    </script>
</body>
</html>
"#,
        version = SWAGGER_UI_VERSION,
        spec_file = swagger_filename
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_swagger_html() {
        let html = generate_swagger_html("swagger.json");
        assert!(html.contains("swagger-ui-bundle.js"));
        assert!(html.contains("swagger-ui.css"));
        assert!(html.contains("./swagger.json"));
        assert!(html.contains(SWAGGER_UI_VERSION));
    }

    #[test]
    fn test_generate_swagger_html_custom_filename() {
        let html = generate_swagger_html("openapi.yaml");
        assert!(html.contains("./openapi.yaml"));
    }

    #[test]
    fn test_path_traversal_rejected() {
        // Regression: book.json could point openapi at absolute paths or
        // ../ escapes, reading/writing outside source and output
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        let out = tmp.path().join("out");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::create_dir_all(&out).unwrap();

        assert!(generate_single_swagger_ui(&src, &out, "api-docs", "../../etc/passwd").is_err());
        assert!(generate_single_swagger_ui(&src, &out, "api-docs", "/etc/passwd").is_err());
        assert!(generate_single_swagger_ui(&src, &out, "/tmp/pwned", "swagger.json").is_err());
        assert!(generate_single_swagger_ui(&src, &out, "../outside", "swagger.json").is_err());
    }

    #[test]
    fn test_valid_relative_paths_accepted() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        let out = tmp.path().join("out");
        std::fs::create_dir_all(src.join("specs")).unwrap();
        std::fs::create_dir_all(&out).unwrap();
        std::fs::write(src.join("specs/swagger.json"), "{}").unwrap();

        assert!(generate_single_swagger_ui(&src, &out, "api-docs", "specs/swagger.json").is_ok());
        assert!(out.join("api-docs/swagger.json").exists());
    }
}
