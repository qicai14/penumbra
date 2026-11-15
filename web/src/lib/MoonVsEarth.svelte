<script lang="ts">
  import type { Scenario } from "./engine";
  import { ROCK_M, fmtLength, maxElevation, shadowLengthM } from "./moonVsEarth";

  let { scenario }: { scenario: Scenario } = $props();

  const peakEl = $derived(maxElevation(scenario.track.sun_el_deg));
  const shadow = $derived(peakEl === null ? null : shadowLengthM(peakEl));
</script>

<section class="card" aria-labelledby="moon-h" data-testid="moon-vs-earth">
  <h3 id="moon-h">Why the Moon is different</h3>
  {#if peakEl === null || shadow === null}
    <p data-testid="moon-sun">Here the Sun never rises in this span. On Earth it would rise every day.</p>
  {:else}
    <p data-testid="moon-sun">
      In this span the Sun climbs no higher than <strong>{peakEl.toFixed(1)}°</strong> here. The Moon's axis tilts only about 1.5° (Earth's: 23.4°), so at the pole the Sun stays low all year.
    </p>
    <p data-testid="moon-shadow">
      At that height a {ROCK_M} m rock casts a <strong>{fmtLength(shadow)}</strong> shadow. With no air to scatter light, anything in shadow stays dark and cold.
    </p>
  {/if}
  <p>Earth hangs near one spot in the sky and only wobbles slightly. Whether you can talk to it depends on the horizon, not on an Earthrise or Earthset each day.</p>
  <p class="src">Axial tilts: NASA planetary fact sheets.</p>
</section>

<style>
  .card { background: var(--surface-carbon); border: 1px solid var(--color-slate); border-radius: var(--radius-sm); padding: var(--spacing-16); margin-top: var(--spacing-16); font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-fog); }
  h3 { margin: 0 0 var(--spacing-8); font: var(--font-weight-regular) var(--text-body)/var(--leading-body) var(--font-mono); }
  p { margin: 0 0 var(--spacing-8); }
  strong { font-weight: var(--font-weight-regular); color: var(--color-paper); font-variant-numeric: tabular-nums; }
  .src { margin: 0; color: var(--color-ash); }
</style>
