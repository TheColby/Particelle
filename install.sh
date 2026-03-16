#!/usr/bin/env bash
set -euo pipefail

INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
PARTICELLE_REPO="${PARTICELLE_REPO:-TheColby/Particelle}"
CHANNEL="stable"
VERSION=""
SKIP_VERIFY=0

usage() {
  cat <<'USAGE'
Particelle installer

Usage:
  ./install.sh [options]

Options:
  --channel <stable|nightly|source>  Install channel (default: stable)
  --version <tag>                    Pin stable install to a specific tag (e.g., v0.2.0)
  --install-dir <path>               Install directory (default: /usr/local/bin)
  --repo <owner/repo>                GitHub repo override (default: TheColby/Particelle)
  --skip-verify                      Skip SHA-256 verification (not recommended)
  -h, --help                         Show help

Examples:
  ./install.sh
  ./install.sh --channel nightly
  ./install.sh --channel stable --version v0.2.0
  ./install.sh --channel source
USAGE
}

require_cmd() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "Missing required command: $cmd" >&2
    exit 1
  fi
}

detect_target() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Linux)
      case "$arch" in
        x86_64) echo "x86_64-unknown-linux-gnu" ;;
        aarch64|arm64)
          echo "Prebuilt Linux arm64 artifacts are not published yet. Use --channel source." >&2
          exit 1
          ;;
        *)
          echo "Unsupported Linux architecture: $arch" >&2
          exit 1
          ;;
      esac
      ;;
    Darwin)
      case "$arch" in
        x86_64) echo "x86_64-apple-darwin" ;;
        arm64) echo "aarch64-apple-darwin" ;;
        *)
          echo "Unsupported macOS architecture: $arch" >&2
          exit 1
          ;;
      esac
      ;;
    *)
      echo "Unsupported operating system: $os" >&2
      exit 1
      ;;
  esac
}

resolve_latest_tag() {
  require_cmd curl
  local api_url tag
  api_url="https://api.github.com/repos/${PARTICELLE_REPO}/releases/latest"
  tag="$(
    curl -fsSL "$api_url" \
      | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' \
      | head -n 1
  )"
  if [[ -z "$tag" ]]; then
    echo "Failed to resolve latest release tag from ${api_url}" >&2
    exit 1
  fi
  echo "$tag"
}

install_binary() {
  local src="$1"
  mkdir -p "$INSTALL_DIR"
  if [[ -w "$INSTALL_DIR" ]]; then
    cp "$src" "$INSTALL_DIR/particelle"
    ln -sf "$INSTALL_DIR/particelle" "$INSTALL_DIR/ptc"
  else
    echo "Installing with sudo into $INSTALL_DIR"
    sudo cp "$src" "$INSTALL_DIR/particelle"
    sudo ln -sf "$INSTALL_DIR/particelle" "$INSTALL_DIR/ptc"
  fi
}

verify_checksum() {
  local checksums_file="$1"
  local work_dir="$2"
  if command -v sha256sum >/dev/null 2>&1; then
    (cd "$work_dir" && sha256sum -c "$checksums_file")
  elif command -v shasum >/dev/null 2>&1; then
    (cd "$work_dir" && shasum -a 256 -c "$checksums_file")
  else
    echo "No checksum tool found (sha256sum or shasum)." >&2
    exit 1
  fi
}

install_prebuilt() {
  local tag="$1"
  local target="$2"
  local asset="particelle_${tag}_${target}.tar.gz"
  local pkg_dir="${asset%.tar.gz}"
  local release_url="https://github.com/${PARTICELLE_REPO}/releases/download/${tag}"

  require_cmd curl
  require_cmd tar

  local tmp_dir
  tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/particelle-install.XXXXXX")"
  trap 'rm -rf "$tmp_dir"' EXIT

  echo "Downloading ${asset} from ${release_url}"
  curl -fsSL -o "${tmp_dir}/${asset}" "${release_url}/${asset}"
  curl -fsSL -o "${tmp_dir}/SHA256SUMS" "${release_url}/SHA256SUMS"

  if [[ "$SKIP_VERIFY" -eq 0 ]]; then
    grep " ${asset}\$" "${tmp_dir}/SHA256SUMS" > "${tmp_dir}/${asset}.sha256"
    verify_checksum "${asset}.sha256" "$tmp_dir"
  else
    echo "Checksum verification skipped."
  fi

  tar -xzf "${tmp_dir}/${asset}" -C "$tmp_dir"
  if [[ ! -f "${tmp_dir}/${pkg_dir}/bin/particelle" ]]; then
    echo "Unexpected package structure in ${asset}" >&2
    exit 1
  fi

  install_binary "${tmp_dir}/${pkg_dir}/bin/particelle"
}

install_from_source() {
  require_cmd cargo
  require_cmd rustc
  echo "Building particelle from source..."
  cargo build --release -p particelle-cli
  install_binary "target/release/particelle"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --channel)
      CHANNEL="${2:-}"
      shift 2
      ;;
    --version)
      VERSION="${2:-}"
      shift 2
      ;;
    --install-dir)
      INSTALL_DIR="${2:-}"
      shift 2
      ;;
    --repo)
      PARTICELLE_REPO="${2:-}"
      shift 2
      ;;
    --skip-verify)
      SKIP_VERIFY=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage
      exit 1
      ;;
  esac
done

case "$CHANNEL" in
  stable)
    target="$(detect_target)"
    tag="${VERSION:-$(resolve_latest_tag)}"
    install_prebuilt "$tag" "$target"
    ;;
  nightly)
    target="$(detect_target)"
    install_prebuilt "nightly" "$target"
    ;;
  source)
    install_from_source
    ;;
  *)
    echo "Invalid channel '$CHANNEL'. Expected: stable, nightly, source." >&2
    exit 1
    ;;
esac

echo "Installed: $(command -v particelle || echo "${INSTALL_DIR}/particelle")"
particelle --version 2>/dev/null || true
echo "Run 'particelle --help' to get started."
