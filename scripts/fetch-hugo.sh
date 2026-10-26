#!/usr/bin/env bash
# 下载并校验本平台（Linux x86_64）的 Hugo 二进制，作为 Tauri sidecar 使用。
# 用法：scripts/fetch-hugo.sh [版本]，默认读取 HUGO_VERSION 或 0.168.0。
set -euo pipefail

VERSION="${1:-${HUGO_VERSION:-0.168.0}}"
NAME="hugonic-hugo"
TRIPLE="x86_64-unknown-linux-gnu"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEST="${ROOT}/src-tauri/binaries/${NAME}-${TRIPLE}"

ARCHIVE="hugo_extended_withdeploy_${VERSION}_linux-amd64.tar.gz"
BASE="https://github.com/gohugoio/hugo/releases/download/v${VERSION}"

WORK="$(mktemp -d)"
trap 'rm -rf "${WORK}"' EXIT

echo "==> 下载 Hugo ${VERSION}（extended + withdeploy）"
curl -fsSL "${BASE}/${ARCHIVE}" -o "${WORK}/${ARCHIVE}"
curl -fsSL "${BASE}/hugo_${VERSION}_checksums.txt" -o "${WORK}/checksums.txt"

echo "==> 校验 SHA256"
(cd "${WORK}" && grep "  ${ARCHIVE}\$" checksums.txt | sha256sum -c -)

echo "==> 安装到 ${DEST}"
tar -xzf "${WORK}/${ARCHIVE}" -C "${WORK}" hugo
mkdir -p "$(dirname "${DEST}")"
install -m 755 "${WORK}/hugo" "${DEST}"

"${DEST}" version
