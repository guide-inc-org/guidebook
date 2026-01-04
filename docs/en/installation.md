# Installation

## Quick Install (Recommended)

### macOS / Linux

```bash
curl -fsSL https://raw.githubusercontent.com/guide-inc-org/guidebook/main/install.sh | sh
```

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/guide-inc-org/guidebook/main/install.ps1 | iex
```

## Via Cargo

If you have Rust installed:

```bash
cargo install guidebook
```

## Verify Installation

```bash
guidebook --version
```

## Update

To update to the latest version:

```bash
guidebook update
```

## Uninstall

### If installed via install script

```bash
rm ~/.local/bin/guidebook
# or
rm /usr/local/bin/guidebook
```

### If installed via Cargo

```bash
cargo uninstall guidebook
```
