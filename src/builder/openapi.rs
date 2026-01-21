//! OpenAPI/Swagger UI generation module
//!
//! Generates a Swagger UI page for API documentation when `openapi` is configured in book.json

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Swagger UI version to use from CDN
const SWAGGER_UI_VERSION: &str = "5.11.0";

/// Generate Swagger UI page for OpenAPI documentation
pub fn generate_swagger_ui(source: &Path, output: &Path, openapi_path: &str) -> Result<()> {
    println!("Generating Swagger UI...");

    // Create api-docs directory
    let api_docs_dir = output.join("api-docs");
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
        .with_context(|| format!("Failed to copy {} to api-docs/", openapi_path))?;

    println!("  Copied {} to api-docs/{}", openapi_path, swagger_filename);

    // Generate index.html with Swagger UI
    let html = generate_swagger_html(swagger_filename);
    fs::write(api_docs_dir.join("index.html"), html)?;

    println!("  Generated api-docs/index.html");

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
}
