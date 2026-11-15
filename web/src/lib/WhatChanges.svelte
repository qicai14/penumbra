<script lang="ts">
  import Explain from "./Explain.svelte";
  import { computeScenario, latlonToXy, type Inputs, type Scenario } from "./engine";
  import type { Grid } from "./grid";
  import { DATE_SHIFT_D, PIN_RADIUS_M, deltaDays, fmtDelta, withinRadius } from "./whatChanges";

  let { scenario, computedFor, grid, gridK, gridUsable }: {
    scenario: Scenario; computedFor: Inputs; grid: Grid | null; gridK: number; gridUsable: boolean;
  } = $props();

  type Run = { d: number } | { loading: true } | { note: string };
  let shifted = $state.raw<Run[]>([{ loading: true }, { loading: true }]);
  let gen = 0;
  // The two extra runs start after the main result (computedFor changes only then) and a newer input set drops older answers.
  $effect(() => {
    const base = computedFor, mine = ++gen, baseH = scenario.survival.operable_h;
    shifted = [{ loading: true }, { loading: true }];
    // One run at a time, and none once inputs move on: computeScenario is serialised, so queued extras would delay the next main run.
    (async () => {
      for (const [n, s] of [-DATE_SHIFT_D, DATE_SHIFT_D].entries()) {
        if (mine !== gen) return;
        let v: Run;
        try { v = { d: deltaDays((await computeScenario({ ...base, start_unix_s: base.start_unix_s + s * 86400 })).survival.operable_h, baseH) }; }
        catch (e: any) { v = { note: String(e?.message ?? e).replace(/^[A-Z_]+: /, "") }; }
        if (mine === gen) shifted = shifted.map((x, i) => (i === n ? v : x));
      }
    })();
  });

  const near = $derived.by(() => {
    if (!grid) return "no grid data for this region";
    if (gridK < 0) return `the grid has no layer for ${new Date(computedFor.start_unix_s * 1000).toISOString().slice(0, 10)}`;
    if (!gridUsable) return `the grid is built for the default preset, mast ${grid.meta.mast_m} m and ${grid.meta.span_days} d span`;
    const [x, y] = latlonToXy(computedFor.lat_deg, computedFor.lon_deg);
    return withinRadius(grid, gridK, x, y) ?? "this site has no grid cell";
  });

  type Bar = { id: string; label: string; v: Run };
  const bars = $derived<Bar[]>([
    { id: "date-minus", label: `Landing ${DATE_SHIFT_D} days earlier`, v: shifted[0] },
    { id: "date-plus", label: `Landing ${DATE_SHIFT_D} days later`, v: shifted[1] },
    { id: "pin-best", label: `Pin moved ${PIN_RADIUS_M} m: best cell`, v: typeof near === "string" ? { note: near } : { d: deltaDays(near.bestH, near.hereH) } },
    { id: "pin-worst", label: `Pin moved ${PIN_RADIUS_M} m: worst cell`, v: typeof near === "string" ? { note: near } : { d: deltaDays(near.worstH, near.hereH) } },
  ]);
  const scale = $derived(Math.max(1, ...bars.map((b) => ("d" in b.v ? Math.abs(b.v.d) : 0))));
  const partial = $derived(typeof near !== "string" && near.partial);
</script>

<section class="panel" aria-labelledby="wc-h">
  <h2 id="wc-h">What changes what</h2>
  <Explain id="wc" topic="What changes what" plain="Which single change helps most: a different landing date, or moving the site a little.">
    <p>Each bar is the gain or loss in working days if you change only that one thing. Bars to the right are better than your pick; longer bars matter more.</p>
    <p>Change in operable days against this pick, for each input moved on its own.</p>
  </Explain>
  <ul data-testid="what-changes">
    {#each bars as b (b.id)}
      <li data-testid={`wc-${b.id}`}>
        <span class="lab">{b.label}</span>
        {#if "loading" in b.v}
          <span class="note">Running…</span>
        {:else if "note" in b.v}
          <span class="note" data-testid={`wc-${b.id}-note`}>No bar: {b.v.note}.</span>
        {:else}
          <span class="track" aria-hidden="true"><i class:neg={b.v.d < 0} style:width={`${(Math.abs(b.v.d) / scale) * 50}%`} style:left={b.v.d < 0 ? `${50 - (Math.abs(b.v.d) / scale) * 50}%` : "50%"}></i></span>
          <strong data-testid={`wc-${b.id}-value`}>{fmtDelta(b.v.d)}</strong>
        {/if}
      </li>
    {/each}
  </ul>
  {#if partial}<p class="cap">The 300 m disc runs past the edge of the grid here, so cells beyond it are not counted.</p>{/if}
</section>

<style>
  .panel { background: var(--surface-carbon); border-radius: var(--radius-sm); padding: var(--spacing-16); margin-top: var(--spacing-24); }
  h2 { margin: 0 0 var(--spacing-8); font: var(--font-weight-regular) var(--text-body)/var(--leading-body) var(--font-mono); color: var(--color-fog); }
  .cap { margin: 0 0 var(--spacing-16); font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-ash); }
  ul { list-style: none; margin: 0; padding: 0; display: grid; gap: var(--spacing-8); }
  li { display: grid; grid-template-columns: minmax(11rem, 16rem) 1fr 5rem; align-items: center; gap: var(--spacing-16); font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-fog); }
  .track { position: relative; height: 0.75rem; border-left: 1px solid var(--color-pewter); border-right: 1px solid var(--color-iron); background: linear-gradient(var(--color-pewter), var(--color-pewter)) center / 1px 100% no-repeat; }
  i { position: absolute; top: 0; bottom: 0; background: var(--color-paper); }
  i.neg { background: var(--color-ash); }
  strong { font-weight: var(--font-weight-regular); color: var(--color-paper); text-align: right; font-variant-numeric: tabular-nums; }
  .note { grid-column: 2 / -1; color: var(--color-ash); }
  @media (max-width: 600px) { li { grid-template-columns: 1fr 5rem; } .track, .note { grid-column: 1 / -1; } strong { grid-row: 1; grid-column: 2; } }
</style>
