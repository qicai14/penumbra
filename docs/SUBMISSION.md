# Submission plan — NSAKL26 (Kuala Lumpur local event)

Source: *NSAKL26 Project Execution Guidebook* (local copy: `D:\DownloadHDD\yogeswaran1livecom_NASASPACEAPPSKUALALUMPUR2026PROJECTEXECUTIONGUIDEBOOK1793029136.pdf`; copyrighted, not committed). Team: **Fractured Toes** = **Qi Cai** (owner, science core) and **Charls** (`ambatuCode887`; app and design). Project: **Penumbra**. Only the deploy URL is `penumbra-lunar.pages.dev`.

## Deliverables (all in English; uploaded via Google Drive, plus the Space Apps project page)
| # | Deliverable | Spec from the guidebook | Owner | Tool |
|---|---|---|---|---|
| D1 | **Project document** (PDF) | Any format. The research-paper outline is suggested: Introduction, Methodology, Results, Discussion, Conclusion, References. NASA and partner data must be cited. | Qi Cai (science sections), Charls (figures, layout) | **Typst** → PDF (see below) |
| D2 | **Team poster** | "Project Name with TEAM POSTER" is item 1 of the outline. Project name "Penumbra", team name "Fractured Toes", both member names | Charls | Canva |
| D3 | **30-second demo video** (mp4, compulsory) | Beats: 0:00 intro → 0:02 challenge → 0:05 name and tagline → 0:10 how it works and the NASA data → 0:20 the future → 0:30 end | Charls | screen capture + Canva video |
| D4 | **3–5 minute project video** (mp4, compulsory) | Solution walkthrough. Note: one page says 3–7 min; aim for 4:30 so it's safe either way | Charls (edit), Qi Cai (science voice-over) | screen capture + Canva video |
| D5 | **Global demo link**: a 7-slide deck (the title slide counts) OR the 30-second video | Slides need team and member names (Fractured Toes: Qi Cai, Charls), the challenge, a project image, the user experience, and how space agency data makes it possible | Charls | Canva → PDF |
| D6 | **Final project link** | One public link that works without login: the deployed app, `https://penumbra-lunar.pages.dev` (and GitHub) | Qi Cai | Cloudflare Pages |
| D7 | **Project page fields** | High-level summary, demo, final project, details (tools and languages), NASA data and partner data with citations, references (Qi Cai, Nov 11: item 6 is References; no separate Use-of-AI write-up, answer only if the form asks) | Qi Cai | `docs/submission/*.md` drafts, pasted in |

**Access:** Drive links must open for logged-out viewers. The guidebook contradicts itself ("editor mode" vs "view option; fail = disqualified"): use **"Anyone with the link: Viewer"** and confirm with the local lead. The project page's "Edit Details" button appears **Nov 15, 09:00 local**. Lock the page while editing, "Save Draft" often, "Publish", then "Submit for Judging".

## Local rubric: 10 criteria × 10 points (Gold ≥ 90)
| # | Criterion | Penumbra's answer | Gap / action |
|---|---|---|---|
| 1 | Relevance to theme ("The Next Frontier") | Artemis/CLPS south-pole landing survival | Say "Next Frontier" in the hook |
| 2 | Originality | Joint Sun∧Earth operable window + time to first lethal shadow; IM-2 replay | Comparison table against Moon Trek / QuickMap / Horizons |
| 3 | Construction technique | Rust→WASM, one model for native and browser, test-first | Show the architecture diagram in the doc and video |
| 4 | Diversity of material used | LOLA DEMs (20 m + 240 m), PGDA AVGVISIB, NAIF kernels, Horizons, LROC NAC, mission reports | List every dataset with its role |
| 5 | Demonstrated understanding | SCIENCE-SPEC equations and pitfalls | Methodology section with the equations |
| 6 | Practicality | Planner question answered per site and date | Real user story: the IM-2 replay |
| 7 | **Commercialization** | Not covered yet | **Add**: users (CLPS vendors, Artemis ops, rover teams), the open API plus a paid "mission planning service" model, cost of a failed lander vs the cost of the tool |
| 8 | Materials submitted | D1–D7 complete | Checklist below |
| 9 | Presentation style | Demo story (IM-2 → fix → ranking → scoreboard) | 30-s and 4:30 scripts written before Sat 13:00 |
| 10 | Use of free NASA resources | All data is NASA's (LOLA, PGDA, NAIF, Horizons, LROC) | Citation table in the doc |

Global categories to target: **Best Use of Science** and **Best Use of Data** (the validation scoreboard).

## Global judging: Space Apps *Judging & Awards Guide* (dated Nov 13 2026, read Nov 14)
Local judges pick the Global Nominees using these same criteria. Every level scores **5 criteria, each 1–5**:

| Criterion | What a 5 needs | Penumbra's answer |
|---|---|---|
| Impact | a significant problem, a wide reach | IM-1/IM-2 losses; every south-pole CLPS and Artemis lander |
| Creativity | a new approach, not done before | joint Sun∧Earth operable window + time to first lethal shadow |
| Validity | scientifically sound, feasible, **usable** | V1–V5 scoreboard; usability counts here too |
| Relevance | answers the challenge; **NASA data is core** | LOLA, PGDA, NAIF, Horizons; site and date comparison (T27) |
| Presentation | judged on the demo **and** the full project | 30-s video, 4:30 video, D1 |

Rules in the guide that change our deliverables:
- **References field** must list *every* outside resource, including open-source code, text and images: Rust crates (ANISE, wasm-bindgen…), Svelte, Vite, Three.js, uPlot, IBM Plex Mono, Inter, Archivo, Typst, NASA images. A checkbox confirms it.
- The demo is 7 slides max (title counts) **or** a 30-s video. A video spoken in another language needs **English subtitles**.
- Every member must be registered for the same local event and be on the team page's Members tab; members can't be added after submission closes.
- The guide says 23:59 local, but **NSAKL26 closes at Sun Nov 15, 22:00** (announced in the NSAKL26 Telegram channel, Nov 14). Submit by 22:00. Global Nominees Dec 3, Finalists Dec 17, Winners Jan 26 2027.

## Event-day checkpoints (from the guidebook timeline)
- Sat 14, 13:00: "Phase 2: video footage review", so capture app footage early and keep it on the shared Drive.
- Sun 15, 08:30: "Phase 3: final project review". 13:00: "Phase 4: proof-read all data, documents and video". **By 22:00 (NSAKL26 deadline, Telegram Nov 14; was 23:59): upload and submit everything.**

## Why these tools
- **Typst for D1, instead of Word (docx) or Canva:** a single binary with no Python. It has real equation typesetting for the science section, and its figures and numbers can be pulled from repo files (scoreboard.json) so the PDF never drifts from the app. Plain text in git means agents can draft it and both of us can review it in PRs. docx is binary, so it can't be diffed or reviewed, and agents edit it poorly. Canva is weak at long text with equations and citations.
- **Canva (MCP connector) for D2, D3–D4 editing and D5:** visual layouts, brand kit reuse, and export to PDF/mp4. Use `design/DESIGN.md` tokens as the Canva brand kit, so the app, poster and slides match.

## Pre-event prep (allowed: templates and planning, no project results)
- Charls (with /design-pass): the brand kit in Canva; poster and 7-slide **templates** with placeholder content; the video storyboard for D3/D4.
- Qi Cai: the Typst paper skeleton (section headings, citation file from `docs/research/`, a figure slot for each V-test); drafts of the Use-of-AI and data-citation fields.
