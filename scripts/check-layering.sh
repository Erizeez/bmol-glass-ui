#!/usr/bin/env bash
#
# Layering guard for the shared widget layer.
#
# This crate exists so that the window shell and the application layer can both
# consume the same glass widgets without either depending on the other. Two
# things would silently undo that, so fail the build when they happen:
#
#   1. a dependency on `bmol-window-shell`, which puts the shell *above* this
#      crate again instead of beside it;
#   2. more than one `liquid-glass-scene` source, which means two incompatible
#      `GlassMaterial` types and materials that cannot be passed across.
#
set -euo pipefail
cd "$(dirname "$0")/.."

tree="$(cargo tree -e normal)"

if grep -q 'bmol-window-shell' <<<"$tree"; then
  echo "layering: this crate must not depend on bmol-window-shell." >&2
  echo "          Window semantics belong to the shell; take them from there" >&2
  echo "          in the consumer, not from here." >&2
  exit 1
fi

scenes="$(grep -oE 'liquid-glass-scene v[0-9.]+ \([^)]*\)' <<<"$tree" | sort -u | wc -l | tr -d ' ')"
if [ "$scenes" != "1" ]; then
  echo "layering: expected exactly one liquid-glass-scene source, found $scenes:" >&2
  grep -oE 'liquid-glass-scene v[0-9.]+ \([^)]*\)' <<<"$tree" | sort -u >&2
  echo "          A second copy means two incompatible GlassMaterial types." >&2
  exit 1
fi

echo "layering ok: no bmol-window-shell dependency, single liquid-glass-scene source."
