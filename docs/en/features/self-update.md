# Self-Update

Update guidebook to the latest version directly from the CLI.

## Usage

```bash
guidebook update
```

This downloads the latest release from GitHub and replaces the current binary.

## How It Works

1. Checks the latest release on GitHub
2. Compares with the current version
3. Downloads the appropriate binary for your platform (macOS, Linux, or Windows)
4. Verifies the SHA256 checksum (mandatory — update is refused if missing)
5. Replaces the current binary

## Platform Support

| Platform | Archive Format |
|----------|---------------|
| macOS (Intel/ARM) | `.tar.gz` |
| Linux | `.tar.gz` |
| Windows | `.zip` |
