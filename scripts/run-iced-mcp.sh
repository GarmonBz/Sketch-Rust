#!/usr/bin/env bash
# Resolve a real iced-mcp binary (never an asdf/mise shim) and exec it.
# Used by the shared project `.mcp.json` so hosts do not hard-code one machine path.
set -euo pipefail

resolve_iced_mcp() {
  if command -v asdf >/dev/null 2>&1; then
    if path="$(asdf which iced-mcp 2>/dev/null)"; then
      if [[ -n "$path" && -x "$path" ]]; then
        printf '%s\n' "$path"
        return 0
      fi
    fi
  fi

  local cargo_bin="${CARGO_HOME:-$HOME/.cargo}/bin/iced-mcp"
  if [[ -x "$cargo_bin" ]]; then
    printf '%s\n' "$cargo_bin"
    return 0
  fi

  local candidate
  for candidate in "$HOME"/.asdf/installs/rust/*/bin/iced-mcp; do
    if [[ -x "$candidate" ]]; then
      printf '%s\n' "$candidate"
      return 0
    fi
  done

  if command -v mise >/dev/null 2>&1; then
    if path="$(mise which iced-mcp 2>/dev/null)"; then
      if [[ -n "$path" && -x "$path" && "$path" != *"/shims/"* ]]; then
        printf '%s\n' "$path"
        return 0
      fi
    fi
  fi

  return 1
}

if ! bin="$(resolve_iced_mcp)"; then
  echo "iced-mcp not found. Install with: cargo install iced-mcp --locked" >&2
  exit 1
fi

exec "$bin" "$@"
