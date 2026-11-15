# Handoff: start every new session here

**Paste first:** `/ticket <n>` for the issue you are on (the "C5-0 Checkpoint 5: submission" issue lists them); pull `main` first. Status as of **Sun Nov 15, ~19:00**.

Public repo `qicai14/penumbra`; the private planning repo `qicai14/penumbra-burner` stays private. Team **Fractured Toes**: Qi Cai = owner and science; Charls (`ambatuCode887`) = app and design. Live app: https://penumbra-lunar.pages.dev

## Where we are
- **Checkpoint 4 is closed:** #107–#115 merged, live at the URL above. Audits: [Nov 14](checkpoints/2026-11-14.md), [Nov 15](checkpoints/2026-11-15.md).
- **Checkpoint 5 is the "C5-0 Checkpoint 5: submission" issue.**
- **Feature code freezes at 19:30; all PRs merged by 20:00.**
- **`docs/submission/facts.md` is the single source for every number** (PDF, project page, video script).

## Plan after 20:00 (Qi Cai)
| When | Ticket | What |
|---|---|---|
| 20:00–20:30 | D1 (#29) | Project PDF: `docs/paper/penumbra.typ` to `penumbra.pdf` (`bun run paper`), numbers from `facts.md` |
| 20:30–20:40 | D6 (#34) | Final project link: logged-out check of the live app and the repo |
| 20:40–21:00 | D7 (#35) | Project page fields from `docs/submission/project-page.md`, then Publish and Submit for Judging |

- **Submit by 21:00. NSAKL26 hard deadline: 22:00.**
- **D4 (#32), the 4–5 min video, is Charls's.**
- **Done:** D2 poster; D3 30-s video (also covers D5).

## Reminders
- AI-generated images need a visible label on each image: https://github.com/nasa/spaceapps/discussions/2195
- Drive links are "Anyone with the link: Viewer" and are opened logged out before submitting.

## Rules
- Never push `burner-main` or any branch made from it to `origin`; never use `--all` or `--mirror` pushes (a hook blocks both). `/ticket` branches from `origin/main`, so it is safe.
- `main` needs checks `test` and `build`; repo auto-merge is on.
- Old process files are in [docs/archive/](archive/README.md).
