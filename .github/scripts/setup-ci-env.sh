#!/usr/bin/env bash
# CI 编译期密钥：优先使用仓库 Secrets，否则回退到公开模板占位值（仅用于 check/test，不可用于正式发布）
set -euo pipefail

PUBLIC_HEX="0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
PUBLIC_PASSWORD="change-me-before-release"

if [ -n "${TIMER_ACTIVATION_SECRET_HEX:-}" ] && [ -n "${TIMER_GENERATOR_PASSWORD:-}" ]; then
  echo "Using repository secrets for build."
  exit 0
fi

echo "::notice::Repository secrets not set; using public template placeholders for CI compile-only."
export TIMER_ACTIVATION_SECRET_HEX="${PUBLIC_HEX}"
export TIMER_GENERATOR_PASSWORD="${PUBLIC_PASSWORD}"

if [ -n "${GITHUB_ENV:-}" ]; then
  echo "TIMER_ACTIVATION_SECRET_HEX=${PUBLIC_HEX}" >> "${GITHUB_ENV}"
  echo "TIMER_GENERATOR_PASSWORD=${PUBLIC_PASSWORD}" >> "${GITHUB_ENV}"
fi
