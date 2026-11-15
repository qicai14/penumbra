<script lang="ts">
  import Explain from "./Explain.svelte";
  // T26 (#27): Sun and Earth elevation at the time marker and the countdown to the next sunset and Earthset.
  // Everything is read from the T08 timeline columns; no new model call.
  import type { Scenario } from "./engine";
  import { fmtEl } from "./panorama";
  import { countdown } from "./timelines";

  // `prefix` keeps element ids and test ids unique when two countdowns share a page (compare view: "a-", "b-").
  let { scenario, idx, prefix = "", title = "Time marker", sticky = true }: { scenario: Scenario; idx: number; prefix?: string; title?: string; sticky?: boolean } = $props();

  const tl = $derived(scenario.timeline);
  const i = $derived(Math.min(idx, tl.t_unix_s.length - 1));
  const sunUp = (x: number) => x > 0; // disc fraction above 0
  const earthUp = (x: number) => x === 1; // the model's 0/1 contact column: already margin ≥ m_min above the terrain (§1d)
</script>

<section class="panel" class:sticky aria-labelledby="{prefix}cd-h" aria-live="off" data-testid="{prefix}countdown">
  <h2 id="{prefix}cd-h">{title}</h2>
  {#if !prefix}
    <Explain id="cd" topic="Time marker" plain="Where the Sun and Earth are at the chosen moment, and how long until each one sets.">
      <p><strong>Elevation</strong> is how high the Sun or Earth sits above a flat horizon, in degrees. Below 0° it has set.</p>
      <p><strong>In shadow</strong>: hills or crater walls block the Sun even though it is up.</p>
      <p><strong>No&nbsp;contact</strong>: Earth is hidden, so the lander cannot radio home.</p>
      <p>Move the time slider to pick a different moment.</p>
    </Explain>
  {/if}
  <div class="figs">
    <p class="fig">
      <span class="cap">Sun elevation{#if !sunUp(tl.illumination[i])}{" · in shadow"}{/if}</span>
      <strong data-testid="{prefix}cd-sun-el">{fmtEl(tl.sun_el_deg[i])}</strong>
      <span class="next" data-testid="{prefix}sunset">{countdown(tl.t_unix_s, tl.illumination, sunUp, i, "sunset")}</span>
    </p>
    <p class="fig">
      <span class="cap">Earth elevation{#if !earthUp(tl.contact[i])}{" · no contact"}{/if}</span>
      <strong data-testid="{prefix}cd-earth-el">{fmtEl(tl.earth_el_deg[i])}</strong>
      <span class="next" data-testid="{prefix}earthset">{countdown(tl.t_unix_s, tl.contact, earthUp, i, "Earthset")}</span>
    </p>
  </div>
</section>

<style>
  .panel { background: var(--surface-carbon); border-radius: var(--radius-sm); padding: var(--spacing-16); margin-top: var(--spacing-24); }
  /* "Always visible" (T26): pinned while the timelines scroll under it; on phones it scrolls to save height. */
  @media (min-width: 768px) {
    .panel.sticky { position: sticky; top: 0; z-index: 1; border: 1px solid var(--color-slate); }
  }
  h2 { margin: 0 0 var(--spacing-16); font: var(--font-weight-regular) var(--text-body)/var(--leading-body) var(--font-mono); color: var(--color-fog); }
  .figs { display: flex; flex-wrap: wrap; gap: var(--spacing-24); }
  .fig { flex: 1 1 240px; margin: 0; display: flex; flex-direction: column; }
  .cap, .next { font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-ash); }
  .next { color: var(--color-fog); font-variant-numeric: tabular-nums; }
  strong { font: var(--font-weight-regular) var(--text-display)/var(--leading-display) var(--font-mono); color: var(--color-paper); font-variant-numeric: tabular-nums; }
</style>
