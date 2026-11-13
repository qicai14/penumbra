#!/usr/bin/env bash
# One-shot, idempotent setup for a Penumbra clone. Git Bash on Windows; also works on macOS/Linux.
# Installs what is safe to install from here; prints the exact command for the rest.
# Usage: bash scripts/bootstrap.sh
cd "$(dirname "$0")/.." || exit 1

ok() { printf '  ok    %s\n' "$1"; }
todo() { printf '  TODO  %s\n        -> %s\n' "$1" "$2"; missing=$((missing + 1)); }
opt() { printf '  opt   %s\n        -> %s\n' "$1" "$2"; }
missing=0
want_wb=$(sed -n 's/^wasm-bindgen = "=\(.*\)"/\1/p' crates/wasm/Cargo.toml)

echo "Toolchain"
if command -v rustup >/dev/null; then
  ok "rustup $(rustup --version 2>/dev/null | cut -d' ' -f2)"
  if rustup target list --installed | grep -q '^wasm32-unknown-unknown$'; then ok "wasm32 target"
  else rustup target add wasm32-unknown-unknown && ok "wasm32 target (installed)"; fi
else
  todo "rustup (Rust)" "winget install Rustlang.Rustup   (needs Visual Studio Build Tools, C++ workload), then re-run this script"
fi

have_wb=$(wasm-bindgen --version 2>/dev/null | cut -d' ' -f2)
if [ "$have_wb" = "$want_wb" ]; then ok "wasm-bindgen $want_wb"
elif command -v cargo >/dev/null; then
  echo "  ...   installing wasm-bindgen-cli $want_wb (a few minutes)"
  cargo install wasm-bindgen-cli --version "$want_wb" --locked && ok "wasm-bindgen $want_wb (installed)"
else
  todo "wasm-bindgen-cli $want_wb" "install Rust first, then re-run this script"
fi

if command -v bun >/dev/null; then ok "bun $(bun --version)"
else todo "bun" "powershell -c \"irm bun.sh/install.ps1 | iex\"   then open a new terminal and re-run"; fi

command -v typst >/dev/null && ok "typst $(typst --version | cut -d" " -f2)" || todo "typst (D1 paper)" "winget install Typst.Typst   then open a new terminal"

if command -v gh >/dev/null; then
  if gh auth status >/dev/null 2>&1; then ok "gh (logged in)"; else todo "gh login" "gh auth login"; fi
else todo "GitHub CLI" "winget install GitHub.cli   then: gh auth login"; fi

if command -v claude >/dev/null; then ok "claude $(claude --version 2>/dev/null | cut -d' ' -f1)"
else todo "Claude Code" "see https://code.claude.com/docs/en/setup, then re-run"; fi

command -v node >/dev/null && ok "node $(node --version) (ponytail plugin hooks)" || todo "Node.js (ponytail plugin hooks)" "winget install OpenJS.NodeJS.LTS   then open a new terminal"
command -v uv >/dev/null && ok "uv (for /design-pass step 1)" || opt "uv (only /design-pass step 1)" "winget install astral-sh.uv"
command -v notebooklm >/dev/null && ok "notebooklm CLI" || opt "notebooklm CLI (researcher agent only)" "uv tool install notebooklm-py   then: notebooklm login"

echo "Project"
if command -v bun >/dev/null; then
  bun install --frozen-lockfile >/dev/null && ok "bun install"
  if command -v claude >/dev/null; then
    for id in $(bun scripts/check-plugins.ts --names); do
      claude plugin install "$id" --scope project && ok "plugin $id (installed)"
    done
    [ -z "$(bun scripts/check-plugins.ts --names)" ] && ok "Claude Code plugins"
  fi
fi

git config core.hooksPath .githooks && ok "git pre-commit hook"

if command -v cargo >/dev/null && [ "$(wasm-bindgen --version 2>/dev/null | cut -d' ' -f2)" = "$want_wb" ] && command -v bun >/dev/null; then
  bun run wasm >/dev/null 2>&1 && ok "WASM built (web/src/lib/wasm)" || todo "WASM build" "bun run wasm   (read its error)"
fi
opt "Playwright browser (only for bun run smoke)" "cd web && bunx playwright install chromium-headless-shell"

echo
if [ "$missing" -eq 0 ]; then
  echo "Ready. Open Claude Code here (Opus, effort medium): /design-pass for design work, /ticket <n> for build work."
else
  echo "$missing item(s) need you (TODO above). Fix them, then run: bash scripts/bootstrap.sh"
  exit 1
fi
