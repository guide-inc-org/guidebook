#!/bin/sh
set -e

# guidebook installer
# Usage: curl -fsSL https://raw.githubusercontent.com/guide-inc-org/guidebook/main/install.sh | sh

REPO="guide-inc-org/guidebook"

# Try /usr/local/bin first, fallback to ~/.local/bin
if [ -w "/usr/local/bin" ]; then
    INSTALL_DIR="/usr/local/bin"
else
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
fi

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Linux*)  echo "linux" ;;
        Darwin*) echo "darwin" ;;
        *)       echo "unsupported" ;;
    esac
}

# Detect architecture
detect_arch() {
    case "$(uname -m)" in
        x86_64)  echo "x86_64" ;;
        amd64)   echo "x86_64" ;;
        aarch64) echo "arm64" ;;
        arm64)   echo "arm64" ;;
        *)       echo "unsupported" ;;
    esac
}

main() {
    OS=$(detect_os)
    ARCH=$(detect_arch)

    if [ "$OS" = "unsupported" ]; then
        echo "Error: Unsupported operating system"
        exit 1
    fi

    if [ "$ARCH" = "unsupported" ]; then
        echo "Error: Unsupported architecture"
        exit 1
    fi

    ARTIFACT="guidebook-${OS}-${ARCH}.tar.gz"
    URL="https://github.com/${REPO}/releases/latest/download/${ARTIFACT}"

    echo "Detected: ${OS} ${ARCH}"
    echo "Downloading ${ARTIFACT}..."

    # Create temp directory
    TMP_DIR=$(mktemp -d)
    trap "rm -rf ${TMP_DIR}" EXIT

    # Download and extract
    curl -fsSL "${URL}" | tar xz -C "${TMP_DIR}"

    # Install
    mv "${TMP_DIR}/guidebook" "${INSTALL_DIR}/guidebook"

    # Check if INSTALL_DIR is in PATH
    case ":$PATH:" in
        *":${INSTALL_DIR}:"*) ;;
        *)
            echo ""
            echo "Note: Add ${INSTALL_DIR} to your PATH:"
            echo "  export PATH=\"\$PATH:${INSTALL_DIR}\""
            ;;
    esac

    echo ""
    echo "guidebook installed successfully!"
    echo "Run 'guidebook --help' to get started."
}

main
