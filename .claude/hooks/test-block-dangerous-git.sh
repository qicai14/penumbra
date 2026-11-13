#!/usr/bin/env bash
# Self-check for block-dangerous-git.sh: bash .claude/hooks/test-block-dangerous-git.sh
# Format: expected exit code (2 = blocked) | command. Run from a branch other than main.
cd "$(dirname "$0")" || exit 1
fail=0
while IFS='|' read -r want cmd; do
  printf '{"tool_input":{"command":"%s"}}' "$cmd" | bash block-dangerous-git.sh >/dev/null 2>&1
  got=$?
  [ "$got" = "$want" ] || { echo "FAIL want $want got $got: $cmd"; fail=1; }
done <<'CASES'
2|git reset --hard HEAD~1
2|git -C x reset --hard
0|echo reset --hard in prose
2|git clean -fd
2|git push origin main
2|git push --force origin 12-x
2|git push -f
2|git push origin +12-x
0|git push -u origin 9-x
2|git status && git push origin HEAD:main
0|cargo test
0|git commit -m 'maintain'
0|git branch -d merged
CASES
[ "$fail" = 0 ] && echo "all guardrail cases pass"
exit "$fail"
