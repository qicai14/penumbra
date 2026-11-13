# Evidence rules

- NotebookLM is a locator, not a source of values: use it to find where a fact lives, then read the paper.
- Limit asks to the relevant sources with `notebooklm ask -s <source-id> ...`.
- The corpus (`docs/research/lunar-landing/corpus/`) is copyrighted paper text: it is committed only in the private `penumbra-burner` and is gitignored in the public `penumbra`. A clone of `penumbra` has no corpus; use NotebookLM as the locator there, or ask Qi Cai for a quote.
- Every number needs a verbatim quote from the paper or the local corpus (`docs/research/lunar-landing/corpus/`, INDEX.tsv maps ids to titles), cited as `<source-id>:<line>`.
- Tag each claim [V] (verified against a quote) or [U] (unverified); [U] values never ship as fact.
