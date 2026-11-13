#!/usr/bin/env bash
# SessionStart: one factual line per missing prerequisite, shown to the user and added to context.
# Silent when everything is in place. Must stay under 1 s.
cd "${CLAUDE_PROJECT_DIR:-$(dirname "$0")/../..}" || exit 0
out=()
fix="scripts/bootstrap.sh installs it"

command -v cargo >/dev/null || out+=("Rust (cargo) is not installed; scripts/bootstrap.sh prints the install command.")
if command -v rustup >/dev/null; then
  rustup target list --installed 2>/dev/null | grep -q '^wasm32-unknown-unknown$' || out+=("Rust target wasm32-unknown-unknown is not installed; $fix.")
fi
want=$(sed -n 's/^wasm-bindgen = "=\(.*\)"/\1/p' crates/wasm/Cargo.toml)
have=$(wasm-bindgen --version 2>/dev/null | cut -d' ' -f2)
[ "$have" = "$want" ] || out+=("wasm-bindgen CLI $want is not installed (found: ${have:-none}); $fix.")
if command -v bun >/dev/null; then
  [ -d node_modules ] || out+=("JS dependencies are not installed (no node_modules); scripts/bootstrap.sh runs bun install.")
  while IFS= read -r line; do [ -n "$line" ] && out+=("$line"); done < <(bun scripts/check-plugins.ts 2>/dev/null)
else
  out+=("Bun is not installed; $fix.")
fi
{ command -v gh >/dev/null && gh auth token >/dev/null 2>&1; } || out+=("GitHub CLI is missing or not logged in; /ticket needs it. Run: gh auth login.")
command -v node >/dev/null || out+=("Node.js is not installed; the ponytail plugin hooks need it. Run: winget install OpenJS.NodeJS.LTS.")
[ "$(git config core.hooksPath)" = ".githooks" ] || out+=("Git pre-commit hook is not installed in this clone; $fix.")

[ ${#out[@]} -eq 0 ] && exit 0
msg="Penumbra setup incomplete:"
for l in "${out[@]}"; do msg+="\\n- $l"; done
msg+="\\nRun: bash scripts/bootstrap.sh"
# Messages hold no double quotes or backslashes, so plain printf makes valid JSON.
printf '{"systemMessage":"%s","hookSpecificOutput":{"hookEventName":"SessionStart","additionalContext":"%s"}}\n' "$msg" "$msg"
