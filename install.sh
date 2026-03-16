#!/usr/bin/env bash
set -euo pipefail

INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
PARTICELLE_REPO="${PARTICELLE_REPO:-TheColby/Particelle}"
CHANNEL="stable"
VERSION=""
SKIP_VERIFY=0
SIGNATURE_VERIFY_MODE="${SIGNATURE_VERIFY_MODE:-auto}"
EXPECTED_OIDC_ISSUER="https://token.actions.githubusercontent.com"

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
  --verify-signatures                Require Sigstore signature verification for downloads
  --skip-signature-verify            Disable Sigstore signature verification
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

signature_identity_regex() {
  local tag="$1"
  if [[ "$tag" == "nightly" ]]; then
    echo "^https://github.com/${PARTICELLE_REPO}/.github/workflows/nightly.yml@.*$"
  else
    echo "^https://github.com/${PARTICELLE_REPO}/.github/workflows/release.yml@.*$"
  fi
}

verify_signature_blob() {
  local blob="$1"
  local sig="$2"
  local cert="$3"
  local identity_regex="$4"
  cosign verify-blob \
    --certificate "$cert" \
    --signature "$sig" \
    --certificate-identity-regexp "$identity_regex" \
    --certificate-oidc-issuer "$EXPECTED_OIDC_ISSUER" \
    "$blob" >/dev/null
}

maybe_verify_signatures() {
  local release_url="$1"
  local asset="$2"
  local tag="$3"
  local tmp_dir="$4"

  case "$SIGNATURE_VERIFY_MODE" in
    off)
      echo "Signature verification disabled."
      return 0
      ;;
    auto)
      if ! command -v cosign >/dev/null 2>&1; then
        echo "cosign not found; skipping signature verification (auto mode)."
        return 0
      fi
      ;;
    on)
      require_cmd cosign
      ;;
    *)
      echo "Invalid SIGNATURE_VERIFY_MODE '${SIGNATURE_VERIFY_MODE}'. Expected: auto, on, off." >&2
      exit 1
      ;;
  esac

  local signature_assets=(
    "${asset}.sig"
    "${asset}.pem"
    "SHA256SUMS.sig"
    "SHA256SUMS.pem"
  )
  local missing_signature_asset=0
  for sig_asset in "${signature_assets[@]}"; do
    if ! curl -fsSL -o "${tmp_dir}/${sig_asset}" "${release_url}/${sig_asset}"; then
      missing_signature_asset=1
      break
    fi
  done

  if [[ "$missing_signature_asset" -ne 0 ]]; then
    if [[ "$SIGNATURE_VERIFY_MODE" == "on" ]]; then
      echo "Signature verification required, but one or more signature assets were unavailable." >&2
      exit 1
    fi
    echo "Signature assets unavailable; skipping signature verification."
    return 0
  fi

  local identity_regex
  identity_regex="$(signature_identity_regex "$tag")"
  verify_signature_blob \
    "${tmp_dir}/SHA256SUMS" \
    "${tmp_dir}/SHA256SUMS.sig" \
    "${tmp_dir}/SHA256SUMS.pem" \
    "$identity_regex"
  verify_signature_blob \
    "${tmp_dir}/${asset}" \
    "${tmp_dir}/${asset}.sig" \
    "${tmp_dir}/${asset}.pem" \
    "$identity_regex"
  echo "Verified Sigstore signatures for ${asset} and SHA256SUMS."
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
  maybe_verify_signatures "$release_url" "$asset" "$tag" "$tmp_dir"

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
    --verify-signatures)
      SIGNATURE_VERIFY_MODE="on"
      shift
      ;;
    --skip-signature-verify)
      SIGNATURE_VERIFY_MODE="off"
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
