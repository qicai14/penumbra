---
name: design-pass
description: Pre-event design sequence (brief, brand, wireframes, prototype, decisions), all inside design/.
argument-hint: "[step 0-4]"
disable-model-invocation: true
---

Run step $0 of the design pass (no argument: the first step whose output is missing). Every file goes in `design/`. `web/` stays untouched before Nov 14; from Nov 14 web/ imports only the `design/DESIGN.md` tokens. Use the `designer` agent for the heavy steps. After each step, stop and show the user what changed.

Before each step: say in plain words what the step will produce and which skill it uses, then wait for a yes. The user may be new to Claude Code, so end each step with the exact next prompt to type.

0. **Brief.** Write `design/BRIEF.md` from `docs/IDEA.md`:
   - users: planners first, then the public
   - the 5 screens: horizon panorama + timeline; 3D terrain with shadows; survival planner heatmap; IM-2 replay; validation scoreboard
   - hard rules: dense data UI; visuals equal the computed numbers; readable at 1° sun angles; accessible contrast (WCAG AA)
1. **Brand (once).** Uses the `brand-identity` skill, a one-time tool installed at user scope, not committed: `npx skills add fatihaydost/brand-identity-skill@brand-identity -g`. It needs `uv` for its Python scripts (dev-time only, never shipped). It is a low-trust repo: read its SKILL.md and every script before running anything. Output: name lockup, palette, type system → `design/brand/`.
2. **Lo-fi.** For each of the 5 screens, 3 structural directions as greyscale static HTML in `design/wireframes/<screen>/<a|b|c>.html`, plus `design/wireframes/index.html` showing them side by side. Ask the user to pick one per screen.
3. **Hi-fi.** Apply the brand to the picked directions in `design/prototype/` (static HTML/CSS, fake data labelled FAKE on screen). Write the tokens to `design/DESIGN.md`: type scale, palette, spacing, motion. Then run an `impeccable` critique pass and fix what it finds.
4. **Decisions.** Record every decision and open question in `design/DECISIONS.md`, for Qi Cai's review on Nov 13.

Done when the step's files exist, contrast passes AA, and the user has seen the result.
