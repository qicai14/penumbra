<script lang="ts">
  import type { Scenario } from "./engine";
  import { headline, spanDays } from "./headline";

  let { scenario, nearby = null, onmove }: { scenario: Scenario; nearby?: string | null; onmove?: () => void } = $props();
  const head = $derived(headline(scenario.survival, spanDays(scenario.count, scenario.step_s)));
</script>

<section class="panel" aria-labelledby="hl-h">
  <h2 id="hl-h">Survival</h2>
  <div class="head" data-testid="headline">
    <p class="fig"><span class="cap">Operable</span><strong data-testid="headline-operable">{head.operable}</strong></p>
    <p class="fig">
      <span class="cap">First lethal shadow in</span>
      <strong data-testid="headline-lethal">{#if !head.none}✕ {/if}{head.lethal}</strong>
    </p>
    {#if nearby}<p class="nearby" data-testid="nearest-better">{nearby}</p>{/if}
    {#if onmove}<button type="button" data-testid="move-pin" onclick={onmove}>Move pin there</button>{/if}
  </div>
</section>

<style>
  .panel { background: var(--surface-carbon); border-radius: var(--radius-sm); padding: var(--spacing-16); }
  h2 { margin: 0 0 var(--spacing-16); font: var(--font-weight-regular) var(--text-body)/var(--leading-body) var(--font-mono); color: var(--color-fog); }
  .head { display: flex; flex-direction: column; gap: var(--spacing-16); }
  .fig { margin: 0; display: flex; flex-direction: column; }
  .cap { font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-ash); }
  .nearby { margin: 0; color: var(--color-fog); font: var(--text-caption)/var(--leading-caption) var(--font-mono); }
  button { align-self: flex-start; min-height: 44px; font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-fog); background: transparent; border: 1px solid var(--color-slate); border-radius: var(--radius-sm); padding: var(--spacing-8) var(--spacing-16); cursor: pointer; }
  button:focus-visible { outline: 1px solid var(--color-paper); outline-offset: 2px; }
  strong { font: var(--font-weight-regular) var(--text-display)/var(--leading-display) var(--font-mono); color: var(--color-paper); font-variant-numeric: tabular-nums; }
</style>
