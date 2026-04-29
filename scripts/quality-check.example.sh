#!/usr/bin/env bash
set -euo pipefail

echo "Git status"
git status --short

echo "Frontend checks"
if [ -f package.json ]; then
  npm run lint || true
  npm run test || true
  npm run build || true
fi

echo "Rust checks"
if [ -d src-tauri ]; then
  cd src-tauri
  cargo fmt --check || true
  cargo clippy --all-targets --all-features || true
  cargo test || true
fi
