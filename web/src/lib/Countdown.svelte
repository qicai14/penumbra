<script lang="ts">
  // T26 (#27): Sun and Earth elevation at the time marker and the countdown to the next sunset and Earthset.
  // Everything is read from the T08 timeline columns; no new model call.
  import type { Scenario } from "./engine";
  import { fmtEl } from "./panorama";
  import { countdown } from "./timelines";

  let { scenario, idx }: { scenario: Scenario; idx: number } = $props();

  const tl = $derived(scenario.timeline);
  const i = $derived(Math.min(idx, tl.t_unix_s.length - 1));
  const sunUp = (x: number) => x > 0; // disc fraction above 0
  const earthUp = (x: number) => x === 1; // the model's 0/1 contact column: already margin ≥ m_min above the terrain (§1d)
</script>

<section class="panel" aria-labelledby="cd-h" aria-live="off" data-testid="countdown">
  <h2 id="cd-h">At the time marker</h2>
  <div class="figs">
    <p class="fig">
      <span class="cap">Sun elevation{#if !sunUp(tl.illumination[i])}{" · in shadow"}{/if}</span>
      <strong data-testid="cd-sun-el">{fmtEl(tl.sun_el_deg[i])}</strong>
      <span class="next" data-testid="sunset">{countdown(tl.t_unix_s, tl.illumination, sunUp, i, "sunset")}</span>
    </p>
    <p class="fig">
      <span class="cap">Earth elevation{#if !earthUp(tl.contact[i])}{" · no contact"}{/if}</span>
      <strong data-testid="cd-earth-el">{fmtEl(tl.earth_el_deg[i])}</strong>
      <span class="next" data-testid="earthset">{countdown(tl.t_unix_s, tl.contact, earthUp, i, "Earthset")}</span>
    </p>
  </div>
</section>

<style>
  .panel { background: var(--surface-carbon); border-radius: var(--radius-sm); padding: var(--spacing-16); margin-top: var(--spacing-24); }
  /* "Always visible" (T26): pinned while the timelines scroll under it; on phones it scrolls to save height. */
  @media (min-width: 768px) {
    .panel { position: sticky; top: 0; z-index: 1; border: 1px solid var(--color-slate); }
  }
  h2 { margin: 0 0 var(--spacing-16); font: var(--font-weight-regular) var(--text-body)/var(--leading-body) var(--font-mono); color: var(--color-fog); }
  .figs { display: flex; flex-wrap: wrap; gap: var(--spacing-24); }
  .fig { flex: 1 1 240px; margin: 0; display: flex; flex-direction: column; }
  .cap, .next { font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-ash); }
  .next { color: var(--color-fog); font-variant-numeric: tabular-nums; }
  strong { font: var(--font-weight-regular) var(--text-display)/var(--leading-display) var(--font-mono); color: var(--color-paper); font-variant-numeric: tabular-nums; }
</style>
