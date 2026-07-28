#!/usr/bin/env bash
set -euo pipefail

REPO="Y-Square-T3/oh-my-codes"
INSTALL_DIR="/usr/local/bin"
VERSION="${1:-}"

detect_platform() {
  local os arch

  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Darwin)
      os="darwin"
      ;;
    Linux)
      os="linux"
      ;;
    *)
      echo "Unsupported OS: $os"
      exit 1
      ;;
  esac

  case "$arch" in
    arm64|aarch64)
      arch="arm64"
      ;;
    x86_64|x64)
      arch="x64"
      ;;
    *)
      echo "Unsupported architecture: $arch"
      exit 1
      ;;
  esac

  echo "${os}-${arch}"
}

get_latest_version() {
  local response
  response="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest")"
  echo "$response" | grep -o '"tag_name": *"[^"]*"' | head -1 | cut -d'"' -f4 | sed 's/^v//'
}

download_and_install() {
  local platform="$1"
  local version="$2"
  local archive_name="omc-${platform}.tar.gz"
  local download_url="https://github.com/${REPO}/releases/download/v${version}/${archive_name}"
  local temp_dir

  echo "Downloading omc v${version} for ${platform}..."
  temp_dir="$(mktemp -d)"
  trap 'rm -rf "$temp_dir"' EXIT

  if ! curl -fsSL "$download_url" -o "${temp_dir}/${archive_name}"; then
    echo "Failed to download from ${download_url}"
    exit 1
  fi

  echo "Extracting..."
  tar -xzf "${temp_dir}/${archive_name}" -C "$temp_dir"

  echo "Installing to ${INSTALL_DIR}..."
  if [ -w "$INSTALL_DIR" ]; then
    cp "${temp_dir}/omc" "${INSTALL_DIR}/omc"
    cp "${temp_dir}/omcd" "${INSTALL_DIR}/omcd"
    chmod +x "${INSTALL_DIR}/omc" "${INSTALL_DIR}/omcd"
  else
    sudo cp "${temp_dir}/omc" "${INSTALL_DIR}/omc"
    sudo cp "${temp_dir}/omcd" "${INSTALL_DIR}/omcd"
    sudo chmod +x "${INSTALL_DIR}/omc" "${INSTALL_DIR}/omcd"
  fi

  echo "✓ omc and omcd installed to ${INSTALL_DIR}"
}

main() {
  local platform version

  platform="$(detect_platform)"
  echo "Detected platform: ${platform}"

  if [ -z "$VERSION" ]; then
    version="$(get_latest_version)"
    if [ -z "$version" ]; then
      echo "Failed to fetch latest version"
      exit 1
    fi
    echo "Latest version: ${version}"
  else
    version="${VERSION#v}"
    echo "Installing version: ${version}"
  fi

  download_and_install "$platform" "$version"

  echo ""
  echo "Installation complete! Run 'omc --help' to get started."
}

main
