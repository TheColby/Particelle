#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: scripts/update_homebrew_formula.sh [options]

Options:
  --tag <vX.Y.Z>         Release tag to target (default: latest release)
  --repo <owner/repo>    GitHub repo (default: TheColby/Particelle)
  --output <path>        Formula output path (default: Formula/particelle.rb)
  -h, --help             Show help
USAGE
}

repo="TheColby/Particelle"
tag=""
output="Formula/particelle.rb"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --tag)
      tag="${2:-}"
      shift 2
      ;;
    --repo)
      repo="${2:-}"
      shift 2
      ;;
    --output)
      output="${2:-}"
      shift 2
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

if ! command -v gh >/dev/null 2>&1; then
  echo "Missing required command: gh" >&2
  exit 1
fi

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

if [[ -z "$tag" ]]; then
  tag="$(gh release list --repo "$repo" --limit 1 --json tagName --jq '.[0].tagName')"
fi

if [[ -z "$tag" ]]; then
  echo "Failed to resolve release tag." >&2
  exit 1
fi

version="${tag#v}"

asset_sha() {
  local asset_name="$1"
  local digest
  if ! digest="$(gh release view "$tag" --repo "$repo" --json assets --jq ".assets[] | select(.name==\"$asset_name\") | .digest")"; then
    echo "Failed to query GitHub release metadata for '$tag'." >&2
    exit 1
  fi
  if [[ -z "$digest" ]]; then
    echo "Unable to find digest for asset '$asset_name' on tag '$tag'." >&2
    exit 1
  fi
  echo "${digest#sha256:}"
}

macos_arm_asset="particelle_${tag}_aarch64-apple-darwin.tar.gz"
macos_x86_asset="particelle_${tag}_x86_64-apple-darwin.tar.gz"
linux_x86_asset="particelle_${tag}_x86_64-unknown-linux-gnu.tar.gz"

macos_arm_sha="$(asset_sha "$macos_arm_asset")"
macos_x86_sha="$(asset_sha "$macos_x86_asset")"
linux_x86_sha="$(asset_sha "$linux_x86_asset")"

mkdir -p "$(dirname "$output")"

cat >"$output" <<FORMULA
class Particelle < Formula
  desc "Algorithmic granular synthesis engine with microtonal and multichannel support"
  homepage "https://github.com/TheColby/Particelle"
  version "${version}"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/${repo}/releases/download/${tag}/${macos_arm_asset}"
      sha256 "${macos_arm_sha}"
    else
      url "https://github.com/${repo}/releases/download/${tag}/${macos_x86_asset}"
      sha256 "${macos_x86_sha}"
    end
  end

  on_linux do
    url "https://github.com/${repo}/releases/download/${tag}/${linux_x86_asset}"
    sha256 "${linux_x86_sha}"
  end

  def install
    binary = Dir["*/bin/particelle"].first
    odie "Expected particelle binary in release tarball, but none was found." if binary.nil?

    bin.install binary => "particelle"
    bin.install_symlink "particelle" => "ptc"
  end

  test do
    assert_match "particelle", shell_output("#{bin}/particelle --version")
  end
end
FORMULA

echo "Updated Homebrew formula at $output for ${repo} ${tag}."
