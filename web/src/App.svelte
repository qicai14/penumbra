<script lang="ts">
  import { computeScenario, loadNearTile, type Inputs, type NearTile, type Scenario } from "./lib/engine";
  import Panorama from "./lib/Panorama.svelte";
  import Timelines from "./lib/Timelines.svelte";
  import Survival from "./lib/Survival.svelte";
  import Tracer from "./lib/Tracer.svelte";
  import Scoreboard from "./lib/Scoreboard.svelte";
  import About from "./lib/About.svelte";
  import SitePicker from "./lib/SitePicker.svelte";
  import { parseDate } from "./lib/site";
  import { Controls } from "./lib/state.svelte";

  const controls = new Controls();
  let tile = $state.raw<NearTile | null>(null);
  let scenario = $state.raw<Scenario | null>(null);
  let computedFor = $state.raw<Inputs | null>(null);
  let error = $state<string | null>(null);
  let idx = $state(0);
  let gen = 0;

  let tileError = $state<string | null>(null); // kept apart so a later compute success cannot hide it
  loadNearTile().then((t) => (tile = t), (e) => (tileError = String(e?.message ?? e)));

  // Any change to site, date, span or mast recomputes; a stale answer never overwrites a newer one.
  $effect(() => {
    const req: Inputs = {
      lat_deg: controls.lat, lon_deg: controls.lon, mast_m: controls.mastM,
      start_unix_s: parseDate(controls.date)!, span_days: controls.spanDays, preset: controls.preset ?? undefined, // the picker only stores dates that parse
    };
    const mine = ++gen;
    computeScenario(req).then(
      (s) => { if (mine === gen) { scenario = s; computedFor = req; error = null; } },
      (e) => { if (mine === gen) error = String(e?.message ?? e); },
    );
  });
</script>

<header class="bar">
  <h1 class="wordmark">Penumbra</h1>
</header>

<main>
  <p class="tag">~/site</p>
  {#if tile}<SitePicker {controls} {tile} />{/if}
  {#if tileError}<p class="msg" role="alert">{tileError}</p>{/if}
  {#if error}<p class="msg" role="alert">{error}</p>{/if}
  {#if scenario && computedFor}
    <p class="msg" data-testid="computed-for">
      Computed for {scenario.site.lat_deg.toFixed(4)}°, {scenario.site.lon_deg.toFixed(4)}°E · mast {scenario.site.mast_m.toFixed(1)} m · landing {controls.date} · {computedFor.span_days} d
    </p>
    <Survival {scenario} {controls} />
    <Panorama {scenario} bind:idx />
    <Timelines {scenario} bind:idx />
  {:else if !error}
    <p class="msg">Loading…</p>
  {/if}
  <Tracer />
  <Scoreboard />
  <About />
</main>

<style>
  :global(body) {
    margin: 0; background: var(--surface-void); color: var(--color-paper);
    font: var(--font-weight-regular) var(--text-body)/var(--leading-body) var(--font-mono);
  }
  .bar { padding: var(--spacing-16); display: flex; align-items: center; border-bottom: 1px solid var(--color-iron); }
  .wordmark { margin: 0; font: italic 800 var(--text-base)/1 var(--font-wordmark); font-stretch: 112%; text-transform: uppercase; }
  main { max-width: var(--page-max-width); margin: 0 auto; padding: var(--spacing-24) var(--spacing-16); }
  .tag { margin: 0 0 var(--spacing-16); font-size: var(--text-caption); line-height: var(--leading-caption); color: var(--color-ash); }
  .msg { color: var(--color-fog); }
</style>
