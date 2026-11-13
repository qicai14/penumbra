# Skills and tools for NASA Space Apps (solo, Claude Code)

Install = `npx skills add <id>`. Installs verified = from `npx skills find` output on 2026-11-08. Stars = from WebFetch/WebSearch summaries (unverified unless noted). Nothing installed.

## A. Research
| name | id | installs/stars | what | verdict |
|---|---|---|---|---|
| mattpocock research | mattpocock/skills@research | 777.6K (V) | research to markdown from primary sources | recommend |
| firecrawl CLI/skill | firecrawl/cli@firecrawl (also -search, -scrape, -crawl, -agent) | 104.2K (V) | web search/scrape/crawl | recommend (needs API key) |
| firecrawl research-papers | firecrawl/firecrawl-workflows@firecrawl-research-papers | 34.1K (V) | paper research workflow | maybe |
| OpenAlex lit search | google-deepmind/science-skills@literature-search-openalex | 3.2K (V), Google DeepMind source | OpenAlex search, free API | recommend |
| notebooklm-py skill | teng-lin/notebooklm-py@notebooklm | 10.1K (V); ~20.2K stars (U) | NotebookLM via unofficial API + CLI + MCP | recommend (see below) |
| PleasePrompto notebooklm | pleaseprompto/notebooklm-skill@notebooklm | 7.8K (V); ~7.8K stars (U) | browser automation; ARCHIVED | skip |
| literature-review | affaan-m/ecc@literature-review | 6.4K (V) | lit review workflow | maybe |
| k-dense citation-management | k-dense-ai/scientific-agent-skills@citation-management | 2.4K (V) | BibTeX/citations | maybe |
| pdf (openai) | openai/skills@pdf | 13.7K (V) | PDF read/extract | maybe (anthropic-skills:pdf already available) |
| claude-office pdf-extraction | claude-office-skills/skills@pdf-extraction | 9.5K (V) | PDF extraction | maybe |
| nasa-earthdata | soljourner/claude-engineering-skills@nasa-earthdata | 42 (V) | Earthdata access | maybe (low adoption; read before use) |
| semantic-scholar | wanshuiyin/auto-claude-code-research-in-sleep@semantic-scholar | 664 (V) | S2 API | maybe |
| arxiv-paper-search | ptreezh/sscisubagent-skills@arxiv-paper-search | 2 (V) | arXiv | skip |

No PubMed hit checked in depth; NASA data has no strong skill (use NASA APIs / Earthdata directly via docs).

## B. Ideation/planning
| name | id | installs | what | verdict |
|---|---|---|---|---|
| brainstorming | obra/superpowers@brainstorming | 420.5K (V) | idea to design | recommend |
| spec-driven-development | addyosmani/agent-skills@spec-driven-development | 73.3K (V) | spec first | recommend |
| prd | github/awesome-copilot@prd | 24.4K (V) | PRD writing | maybe |
| hackathon-judging | samber/dev-event-organizer-skills@hackathon-judging | 6.5K (V) | judging criteria view (organizer-oriented) | maybe |
| pitch-deck | anthropics/financial-services@pitch-deck | 2.5K (V) | pitch deck | maybe |

## C. Build
| name | id | installs | verdict |
|---|---|---|---|
| frontend-design | anthropics/skills@frontend-design | 1.1M (V) | recommend |
| impeccable (EXISTS) | pbakaus/impeccable@impeccable | 391.2K (V) | recommend (also teach-impeccable 50K, frontend-design 53.2K) |
| bun | brianlovin/agent-config@bun 2.8K; bun.sh@Bun 3.7K; affaan-m/ecc@bun-runtime 11.2K | (V) | maybe: bun.sh@Bun |
| hono | yusukebe/hono-skill@hono (author of Hono) | 12.6K (V) | recommend |
| rust | apollographql/skills@rust-best-practices 22.5K; wshobson rust-async-patterns 21.8K | (V) | maybe |
| wasm | mohitmishra786/low-level-dev-skills@wasm-emscripten | 555 (V) | skip/maybe |
| three.js | cloudai-x/threejs-skills@threejs-fundamentals (20.9K), -animation 24K, -shaders 19K | (V) | recommend (fundamentals + shaders) |
| react-three-fiber | cloudai-x/threejs-skills@threejs-react-three-fiber 4.2K; vercel-labs/json-render@react-three-fiber 2.4K | (V) | maybe |
| threlte | kjanat/skills@threlte | 23 (V) | skip (use ctx7 docs) |
| webxr | freshtechbro/claudedesignskills@aframe-webxr 2.5K; full-stack-skills/threejs-skills@threejs-webxr 408 | (V) | maybe |
| cesium | cesiumgs/cesiumjs-skills@cesiumjs-viewer-setup (official org) | 279 (V) | maybe (official, low installs) |
| maplibre | maplibre/maplibre-agent-skills@maplibre-tile-sources 1.1K, -pmtiles-patterns 823 | (V) | maybe |
| deck.gl | karlaugustnybo/deckgl-skill@deckgl | 7 (V) | skip |
| svelte | sveltejs/ai-tools@svelte-code-writer 13.9K, svelte-core-bestpractices 12K (official) | (V) | recommend if Svelte chosen |
| sqlite | secondsky/claude-skills@bun-sqlite | 527 (V) | skip |
| duckdb | duckdb/duckdb-skills@query 1.5K, @read-file 1.3K (official) | (V) | maybe |
| postgres | supabase/agent-skills@supabase-postgres-best-practices | 476.8K (V) | maybe (only if Postgres) |
| cloudflare | cloudflare/skills@workers-best-practices | 138.2K (V) | recommend if deploying to Workers |
| vercel | vercel-labs/agent-skills@deploy-to-vercel | 185K (V) | recommend if Vercel |
| fly.io | fellowship-dev/dogfooded-skills@fly-io | 200 (V) | skip |
| playwright | microsoft/playwright-cli@playwright-cli | 210K (V) | recommend |

## D. Presentation
| name | id | installs | verdict |
|---|---|---|---|
| data-storytelling | wshobson/agents@data-storytelling | 17.5K (V) | recommend |
| create-readme | github/awesome-copilot@create-readme | 19.4K (V) | recommend |
| readme-optimization | samber/developer-relations-skills@readme-optimization | 13.5K (V) | maybe |
| demo-video | alirezarezvani/claude-skills@demo-video | 728 (V) | maybe |
| hackathon-demo-video | bernieweb3/hackathon-ai-devkit@hackathon-demo-video | 59 (V) | skip |
| slides | nextlevelbuilder/ui-ux-pro-max-skill@slides | 45K (V) | maybe (pptx skill already available) |

## NotebookLM integrations (read from GitHub pages via WebFetch; stars unverified)
| item | how | auth | Windows | status | verdict |
|---|---|---|---|---|---|
| LeeJuOh/claude-code-zero notebooklm-connector | Claude-in-Chrome extension drives NotebookLM UI | Google login in Chrome + paid Claude plan | not mentioned | stars/last commit not visible; 2 issues | maybe (fragile, 40K char input cap, 3 follow-ups) |
| PleasePrompto/notebooklm-skill | Patchright browser automation, fresh session per question | one-time Google login, local data/ | implied, unconfirmed | ARCHIVED, "no longer maintained" (Sep 2026); notebooks must be shared | skip |
| teng-lin/notebooklm-py | undocumented Google APIs (batchexecute); Playwright only for login | browser login, cookie import, or master token | pip works, Windows steps in docs | ~20.2K stars, 2,580 commits; CLI + local stdio MCP + skill; note Google renamed NotebookLM to "Gemini Notebook" | recommend (can break if Google changes APIs; use a dedicated Google account) |

No official Google NotebookLM MCP/CLI found.

## Paper-search MCP servers (stars from web search, unverified)
| name | stars | verdict |
|---|---|---|
| blazickjp/arxiv-mcp-server | ~3.3K | recommend (arXiv search/read/BibTeX, local) |
| openags/paper-search-mcp | ~2.4K (older figure) | recommend (multi-source incl. Semantic Scholar/OpenAlex/PubMed) |
| semantic-scholar MCPs (akapet00 35, fastmcp 164) | small | skip, covered by above |
| OpenAlex-only MCP | none notable | skip; use the DeepMind OpenAlex skill |

## Firecrawl free tier
Sources conflict: 1,000 credits (docs rate-limits page says so) vs older 500 one-time; unclear if monthly or one-time. Free rate limits per docs: scrape 10/min, search 10/min, crawl 2/min, concurrency 2. 1 credit per page, 2 per search. Paid: Hobby $19/mo. Check firecrawl.dev/pricing. (docs.firecrawl.dev/pricing 404'd.)
