<script lang="ts">
  // One plain-words line and three scenario buttons. Each button only calls back into the picker's own site and date setters.
  let { onim2, onbest, onslip, ready, bestReady, rankedDates }: { onim2: () => void; onbest: () => void; onslip: () => void; ready: boolean; bestReady: boolean; rankedDates: string[] } = $props();
</script>

<section class="hero" aria-label="What Penumbra answers" data-testid="hero">
  <p class="line" data-testid="hero-line">Where and when can a lander at the lunar south pole keep sunlight and Earth contact, and when does the first lethal shadow arrive? Built for Artemis III and CLPS planners.</p>
  <div class="btns" role="group" aria-label="Try a scenario">
    <button type="button" data-testid="hero-im2" onclick={onim2} disabled={!ready}>IM-2's site, 2026-11-14</button>
    <button type="button" data-testid="hero-best" onclick={onbest} disabled={!bestReady} aria-describedby={bestReady ? undefined : "hero-best-note"}>Best region on this date</button>
    <button type="button" data-testid="hero-slip" onclick={onslip} disabled={!ready}>Schedule slips 2 days</button>
  </div>
  {#if !bestReady}<p class="note" id="hero-best-note" data-testid="hero-best-note">No region ranking for this date. Ranked dates: {rankedDates.length ? rankedDates.join(", ") : "loading"}.</p>{/if}
</section>

<style>
  .hero { margin-bottom: var(--spacing-24); }
  .line { margin: 0 0 var(--spacing-16); max-width: 70ch; color: var(--color-paper); font-size: var(--text-heading-sm); line-height: var(--leading-heading); }
  .btns { display: flex; flex-wrap: wrap; gap: var(--spacing-8) var(--spacing-16); }
  .note { margin: var(--spacing-8) 0 0; font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-fog); }
  button { min-height: 44px; min-width: 44px; font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-paper); background: transparent; border: 1px solid var(--color-slate); border-radius: var(--radius-sm); padding: var(--spacing-8) var(--spacing-16); cursor: pointer; }
  button:hover:not(:disabled) { border-color: var(--color-paper); }
  button:disabled { color: var(--color-ash); cursor: not-allowed; }
  button:focus-visible { outline: 1px solid var(--color-paper); outline-offset: 2px; }
</style>
