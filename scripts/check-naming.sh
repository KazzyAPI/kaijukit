#!/bin/sh
# Fails if a banned term appears outside the allowed doc paths.
cd "$(dirname "$0")/.." || exit 1
bad=$(git ls-files | grep -Ev '^(docs/|conformance/|\.github/|README\.md|AGENTS\.md|LICENSE|scripts/)' \
  | xargs -r grep -Iinf scripts/banned-terms.txt 2>/dev/null | grep -Ev '[Aa]ccount_?[Tt]ype')
[ -z "$bad" ] || { echo "Banned names found:"; echo "$bad"; exit 1; }
