#!/bin/sh
set -e

# --- Replace this with your actual GitHub owner/repo ---
REPO="tdwells90/lncli"
BINARY="lncli"

main() {
    os=$(uname -s)
    arch=$(uname -m)

    case "$os" in
        Linux)  os_target="unknown-linux-musl" ;;
        Darwin) os_target="apple-darwin" ;;
        *)      echo "Unsupported OS: $os" >&2; exit 1 ;;
    esac

    case "$arch" in
        x86_64)         arch_target="x86_64" ;;
        aarch64|arm64)  arch_target="aarch64"
                        # Linux aarch64 uses gnu, not musl
                        if [ "$os" = "Linux" ]; then os_target="unknown-linux-gnu"; fi
                        ;;
        *)              echo "Unsupported architecture: $arch" >&2; exit 1 ;;
    esac

    target="${arch_target}-${os_target}"
    install_dir="${LNCLI_INSTALL_DIR:-$HOME/.local/bin}"

    echo "Detecting platform: ${target}"

    # Get latest release tag
    latest=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | cut -d'"' -f4)
    if [ -z "$latest" ]; then
        echo "Failed to fetch latest release" >&2
        exit 1
    fi

    echo "Installing ${BINARY} ${latest}..."

    url="https://github.com/${REPO}/releases/download/${latest}/${BINARY}-${target}.tar.gz"

    tmpdir=$(mktemp -d)
    trap 'rm -rf "$tmpdir"' EXIT

    curl -fsSL "$url" -o "${tmpdir}/${BINARY}.tar.gz"
    tar xzf "${tmpdir}/${BINARY}.tar.gz" -C "$tmpdir"

    mkdir -p "$install_dir"
    mv "${tmpdir}/${BINARY}" "${install_dir}/${BINARY}"
    chmod +x "${install_dir}/${BINARY}"

    echo "Installed ${BINARY} to ${install_dir}/${BINARY}"

    if ! command -v "$BINARY" >/dev/null 2>&1; then
        echo ""
        echo "Add ${install_dir} to your PATH:"
        echo "  export PATH=\"${install_dir}:\$PATH\""
    else
        "$BINARY" --version
    fi
}

main
