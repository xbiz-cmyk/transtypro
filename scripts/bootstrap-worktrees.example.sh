#!/usr/bin/env bash
set -euo pipefail

# Run after Phase 0 has created the baseline project and main branch is clean.

git status

git branch phase/01-ui-shell || true
git branch phase/02-storage-settings || true
git branch phase/03-audio-stt || true
git branch phase/06-privacy-diagnostics || true

git worktree add ../transtypro-ui phase/01-ui-shell
git worktree add ../transtypro-storage phase/02-storage-settings
git worktree add ../transtypro-audio phase/03-audio-stt
git worktree add ../transtypro-qa phase/06-privacy-diagnostics
