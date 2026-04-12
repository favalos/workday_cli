#!/bin/sh
set -e

REPO="favalos/workday_cli"
VERSION="v0.0.4"
BINARY_URL="https://github.com/${REPO}/releases/download/${VERSION}/workday_cli"
SKILL_URL="https://raw.githubusercontent.com/${REPO}/main/SKILL.md"

BIN_DIR="$HOME/.local/bin"
CERT_DIR="$HOME/.w-cli"
SKILL_DIR="$HOME/.claude/skills/workday-cli"

echo "==> Installing workday_cli ${VERSION}"

# 1. Download the binary
echo "==> Downloading workday_cli binary..."
mkdir -p "$BIN_DIR"
curl -fSL "$BINARY_URL" -o "$BIN_DIR/workday_cli"
chmod +x "$BIN_DIR/workday_cli"
echo "    Installed to $BIN_DIR/workday_cli"

# 2. Ensure ~/.local/bin is on PATH
add_to_path() {
    local shell_rc="$1"
    if [ -f "$shell_rc" ] && grep -q '\.local/bin' "$shell_rc" 2>/dev/null; then
        return
    fi
    echo '' >> "$shell_rc"
    echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$shell_rc"
    echo "    Added ~/.local/bin to PATH in $shell_rc"
}

case "$PATH" in
    *"$BIN_DIR"*) echo "==> ~/.local/bin already on PATH" ;;
    *)
        echo "==> Adding ~/.local/bin to PATH..."
        case "$(basename "$SHELL")" in
            zsh)  add_to_path "$HOME/.zshrc" ;;
            bash) add_to_path "$HOME/.bashrc" ;;
            *)    add_to_path "$HOME/.profile" ;;
        esac
        export PATH="$BIN_DIR:$PATH"
        ;;
esac

# 3. Install mkcert and generate certificates
echo "==> Setting up mkcert and certificates..."
if ! command -v mkcert >/dev/null 2>&1; then
    if command -v brew >/dev/null 2>&1; then
        echo "    Installing mkcert via Homebrew..."
        brew install mkcert
    else
        echo "    ERROR: mkcert is not installed and Homebrew is not available."
        echo "    Please install mkcert manually: https://github.com/FiloSottile/mkcert#installation"
        exit 1
    fi
else
    echo "    mkcert already installed"
fi

echo "    Installing local CA (may require sudo)..."
mkcert -install

mkdir -p "$CERT_DIR"
if [ -f "$CERT_DIR/localhost.pem" ] && [ -f "$CERT_DIR/localhost-key.pem" ]; then
    echo "    Certificates already exist in $CERT_DIR"
else
    echo "    Generating localhost certificates..."
    mkcert \
        -cert-file "$CERT_DIR/localhost.pem" \
        -key-file "$CERT_DIR/localhost-key.pem" \
        localhost 127.0.0.1
    echo "    Certificates created in $CERT_DIR"
fi

# 4. Install SKILL.md for Claude Code
echo "==> Installing Claude Code skill..."
mkdir -p "$SKILL_DIR"
curl -fSL "$SKILL_URL" -o "$SKILL_DIR/SKILL.md"
echo "    Installed to $SKILL_DIR/SKILL.md"

echo ""
echo "==> workday_cli installed successfully!"
echo "    Run 'workday_cli --help' to get started."
echo "    (You may need to restart your shell or run: export PATH=\"\$HOME/.local/bin:\$PATH\")"
