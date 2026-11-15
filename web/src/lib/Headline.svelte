<script lang="ts">
  import Explain from "./Explain.svelte";
  import type { Scenario } from "./engine";
  import { headline, hourFigures, spanDays } from "./headline";

  let { scenario, nearby = null }: { scenario: Scenario; nearby?: string | null } = $props();
  const head = $derived(headline(scenario.survival, spanDays(scenario.count, scenario.step_s)));
  const hours = $derived(hourFigures(scenario.survival));
</script>

<section class="panel" aria-labelledby="hl-h">
  <h2 id="hl-h">Survival</h2>
  <Explain id="hl" topic="Survival" plain="How long a lander here can work, and when it first runs out of power.">
    <p><strong>Operable</strong>: hours with both sunlight and Earth contact, counted until the battery first runs flat.</p>
    <p><strong>First lethal shadow</strong>: when a shadow lasts long enough to drain the battery. The model stops there.</p>
    <p><strong>Sunlight</strong> and <strong>Earth contact</strong> are totals over the whole span, each counted on its own.</p>
  </Explain>
  <div class="head" data-testid="headline">
    <p class="fig"><span class="cap">Operable</span><strong data-testid="headline-operable">{head.operable}</strong></p>
    <p class="fig">
      <span class="cap">First lethal shadow in</span>
      <strong data-testid="headline-lethal">{#if !head.none}✕ {/if}{head.lethal}</strong>
    </p>
    <p class="fig"><span class="cap">Sunlight</span><strong class="sub" data-testid="headline-sun-hours">{hours.sun}</strong></p>
    <p class="fig"><span class="cap">Earth contact</span><strong class="sub" data-testid="headline-earth-hours">{hours.earth}</strong></p>
    {#if nearby}<p class="nearby" data-testid="nearest-better">{nearby}</p>{/if}
  </div>
</section>

<style>
  .panel { background: var(--surface-carbon); border-radius: var(--radius-sm); padding: var(--spacing-16); }
  h2 { margin: 0 0 var(--spacing-16); font: var(--font-weight-regular) var(--text-body)/var(--leading-body) var(--font-mono); color: var(--color-fog); }
  .head { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: var(--spacing-16); }
  .nearby { grid-column: 1 / -1; }
  .fig { margin: 0; display: flex; flex-direction: column; }
  .cap { font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-ash); }
  .nearby { margin: 0; color: var(--color-fog); font: var(--text-caption)/var(--leading-caption) var(--font-mono); }
  .sub { font-size: var(--text-heading); line-height: var(--leading-heading); }
  strong { font: var(--font-weight-regular) var(--text-display)/var(--leading-display) var(--font-mono); color: var(--color-paper); font-variant-numeric: tabular-nums; }
</style>
