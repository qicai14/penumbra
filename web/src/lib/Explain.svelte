<script lang="ts">
  // One plain-language line under a panel title, plus an optional "What this means" dropdown for readers who are not planners
  // (educators, the public). Planners read the numbers; the dropdown stays closed unless someone opens it.
  import type { Snippet } from "svelte";
  let { plain, id, topic, children }: { plain: string; id: string; topic: string; children?: Snippet } = $props(); // topic: the panel title, so each summary has its own name for screen readers
</script>

<p class="plain" data-testid="{id}-plain">{plain}</p>
{#if children}
  <details class="explain" data-testid="{id}-explain">
    <summary>What this means<span class="sr-only">: {topic}</span></summary>
    <div class="body">{@render children()}</div>
  </details>
{/if}

<style>
  .plain { margin: calc(-1 * var(--spacing-8)) 0 var(--spacing-8); max-width: 75ch; font: var(--text-body)/var(--leading-body) var(--font-mono); color: var(--color-paper); }
  .explain { margin: 0 0 var(--spacing-16); font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-fog); }
  .explain > summary { cursor: pointer; width: fit-content; color: var(--color-fog); text-decoration: underline; text-underline-offset: 4px; }
  .explain > summary:hover { color: var(--color-paper); }
  .explain > summary:focus-visible { outline: 1px solid var(--color-paper); outline-offset: 2px; }
  .sr-only { position: absolute; width: 1px; height: 1px; overflow: hidden; clip-path: inset(50%); white-space: nowrap; }
  .body { margin-top: var(--spacing-8); max-width: 75ch; }
  .body :global(p) { margin: 0 0 var(--spacing-8); }
  /* `font: inherit` beats a host panel's own `strong` rule (Countdown and Headline draw their big figures as <strong>). */
  .body :global(strong) { font: inherit; color: var(--color-paper); }
</style>
