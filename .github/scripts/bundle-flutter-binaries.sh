#!/usr/bin/env bash
set -euo pipefail

package_dir="$1"
version="$(tr -d '[:space:]' < "$package_dir/binary_version")"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

for target in aarch64-apple-darwin x86_64-unknown-linux-musl aarch64-unknown-linux-musl; do
  archive="mitame-$target.tar.gz"
  gh release download "v$version" --repo mataku/mitame --dir "$work" --pattern "$archive" --pattern "$archive.sha256"
  if command -v sha256sum >/dev/null 2>&1; then
    (cd "$work" && sha256sum -c "$archive.sha256")
  else
    (cd "$work" && shasum -a 256 -c "$archive.sha256")
  fi
  tar -xzf "$work/$archive" -C "$work"
  mkdir -p "$package_dir/native/$target"
  install -m 755 "$work/mitame-$target/mitame" "$package_dir/native/$target/mitame"
done
