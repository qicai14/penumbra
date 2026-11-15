# Evidence rules

- NotebookLM is a locator, not a source of values: use it to find where a fact lives, then read the paper.
- Limit asks to the relevant sources with `notebooklm ask -s <source-id> ...`.
- The corpus (`docs/research/lunar-landing/corpus/`) is copyrighted paper text: it is committed only in the private `penumbra-burner` and is gitignored in the public `penumbra`. A clone of `penumbra` has no corpus; use NotebookLM as the locator there, or ask Qi Cai for a quote.
- Every number needs a verbatim quote from the paper or the local corpus (`docs/research/lunar-landing/corpus/`, INDEX.tsv maps ids to titles), cited as `<source-id>:<line>`.
- Tag each claim [V] (verified against a quote) or [U] (unverified); [U] values never ship as fact.

## Commercialization sources

Used by `docs/submission/commercialization.md` (#115). [V] = page fetched and quote checked; the quotes below are short extracts.

| id | Source | Claim used | Quote | Tag |
|---|---|---|---|---|
| C1 | NASA OIG IG-24-013, "NASA's Commercial Lunar Payload Services Initiative", 2024, Table 2 (as of Feb 2024). https://oig.nasa.gov/wp-content/uploads/2024/06/final-report-ig-24-013-nasas-commercial-lunar-payload-services-initiative.pdf | IM-1 $81.7M → $122.0M; PRIME-1 (IM-2) $47.0M → $50.0M; Astrobotic TO-20A $199.5M → $322.8M; VIPER $505.4M | "Intuitive Machines May 2019 81.7 122.0"; "October 2020 47.0 50.0 … (PRIME-1)"; "Astrobotic (TO-20A) June 2020 199.5 322.8"; "NASA's $505.4-million Volatiles Investigating Polar Exploration Rover (VIPER)" | [V] |
| C2 | NASA release, 2025-03-07, "NASA Receives Some Data Before Intuitive Machines Ends Lunar Mission". https://www.nasa.gov/news-release/nasa-receives-some-data-before-intuitive-machines-ends-lunar-mission/ | IM-2 on its side, early end before batteries depleted | "the lander was on its side"; "before its batteries were depleted" | [V] |
| C3 | NASA CLPS page. https://www.nasa.gov/clps | $2.6B cap through Nov 2028 | "combined maximum contract value of $2.6 billion through November 2028" | [V] |
| C4 | NASA release, 2026-06-09, Artemis III crew. https://www.nasa.gov/news-release/nasa-marches-toward-artemis-iii-mission-in-2027-names-crew-members/ ; also https://www.nasa.gov/news-release/nasa-adds-mission-to-artemis-lunar-program-updates-architecture/ (2026-02-27) | Artemis III 2027 in Earth orbit; first crewed south-pole landing Artemis IV 2028 | "the first planned crewed mission to the lunar South Pole in 2028" | [V]* |
| C5 | NASA event page, Blue Origin Blue Moon MK1 (updated 2026-10-21). https://www.nasa.gov/event/clps-flight-blue-origins-blue-moon-mark-1/ | MK1 lands 2027, Shackleton connecting ridge | "The mission will land on the Shackleton Connecting Ridge." | [V]* |
| C6 | NASA Science, CLPS deliveries table (updated 2026-10-23). https://science.nasa.gov/lunar-science/clps-deliveries/ | South-pole sites for IM Nova-C (Mons Mouton), IM Nova-D, Firefly Blue Ghost 4, Astrobotic Griffin; no dates used | site column "Lunar South Pole" / "Mons Mouton" | [V]* |
| C7 | Xinhua via english.www.gov.cn, 2026-05-23. https://english.www.gov.cn/news/202605/23/content_WS6a118ca1c6d00ca5f9a0b33c.html | Chang'e-7 launch H2 2026, south-pole survey | "scheduled for launch in the second half of 2026" | [V]* |

\* Fetched through a summarising web tool, so the quote is not byte-checked against the raw page; re-open the URL before quoting it verbatim in the paper.

Not used, because no primary source was found: an IM-1 dollar figure on nasa.gov (OIG's text says $81.3M where its table says $81.7M), IM-3/IM-4 and Griffin launch dates, LUPEX dates, and any IM statement that cold or shadow ended IM-2.
