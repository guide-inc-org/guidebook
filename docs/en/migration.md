# Migration from HonKit

guidebook is designed as a drop-in replacement for HonKit.

## Quick Migration

1. Install guidebook:
   ```bash
   curl -fsSL https://raw.githubusercontent.com/guide-inc-org/guidebook/main/install.sh | sh
   ```

2. Replace your commands:
   ```bash
   # Before (HonKit)
   npx honkit build
   npx honkit serve

   # After (guidebook)
   guidebook build
   guidebook serve
   ```

That's it! No configuration changes needed.

## Compatibility

### Supported

- `book.json` configuration
- `SUMMARY.md` format
- `LANGS.md` multi-language
- Markdown rendering
- Custom styles
- Most plugins (collapsible-chapters, back-to-top-button)

### Not Supported

- PDF/EPUB export (web only)
- JavaScript plugins (guidebook uses built-in Rust implementations)
- GitBook legacy format

## Benefits of Switching

| Feature | HonKit | guidebook |
|---------|--------|-----------|
| Install | Node.js + npm | Single binary |
| Build speed | Seconds | Milliseconds |
| Dependencies | Many | None |
| Self-update | Manual | `guidebook update` |

## CI/CD Migration

### Before (HonKit)

```yaml
- uses: actions/setup-node@v4
- run: npm install -g honkit
- run: honkit build
```

### After (guidebook)

```yaml
- run: |
    curl -sL https://github.com/guide-inc-org/guidebook/releases/latest/download/guidebook-linux-x86_64.tar.gz | tar xz
    ./guidebook build
```
