#!/usr/bin/env bash
# PreToolUse(Bash) guardrail, from mattpocock-skills git-guardrails-claude-code, customised:
# blocks force-push, any push to main, reset --hard and the other destructive commands below.
INPUT=$(cat)
if command -v jq >/dev/null; then
  COMMAND=$(printf '%s' "$INPUT" | jq -r '.tool_input.command // empty')
elif command -v bun >/dev/null; then
  COMMAND=$(printf '%s' "$INPUT" | bun -e 'console.log(JSON.parse(await Bun.stdin.text()).tool_input?.command ?? "")')
else
  COMMAND=$INPUT # shortcut: no JSON parser, so match the raw payload (may over-block), install bun to fix
fi

block() {
  echo "BLOCKED: '$COMMAND' $1. The user has prevented you from doing this; ask them to run it if it is really needed." >&2
  exit 2
}

DANGEROUS_PATTERNS=(
  "reset --hard"
  "clean -f"
  "branch -D"
  "checkout \."
  "restore \."
)
for pattern in "${DANGEROUS_PATTERNS[@]}"; do
  # Same git invocation only (`git -C x reset --hard` counts; prose mentioning it does not).
  printf '%s' "$COMMAND" | grep -qE "git[^;&|\`]* $pattern" && block "matches dangerous pattern '$pattern'"
done

# Look at each `git push` segment of a compound command.
while IFS= read -r push; do
  [ -z "$push" ] && continue
  printf '%s' "$push" | grep -qE -- "(--force|--force-with-lease|--mirror|--delete|[[:space:]]-[a-zA-Z]*f|[[:space:]]\+)" && block "is a force or destructive push"
  printf '%s' "$push" | grep -qE "(^|[[:space:]:/])main([[:space:]]|$)" && block "pushes to main; open a PR instead"
  # A bare `git push` / `git push origin` pushes the current branch.
  if ! printf '%s' "$push" | grep -qE "git push([[:space:]]+-[^[:space:]]+)*[[:space:]]+[^-[:space:]]+[[:space:]]+[^-[:space:]]"; then
    [ "$(git branch --show-current 2>/dev/null)" = "main" ] && block "pushes the current branch, which is main; open a PR instead"
  fi
done < <(printf '%s' "$COMMAND" | grep -oE "git push[^;&|]*")

exit 0
