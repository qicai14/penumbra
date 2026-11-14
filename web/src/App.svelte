<script lang="ts">
  import { loadFixture, type Scenario } from "./lib/engine";
  import Panorama from "./lib/Panorama.svelte";
  import Timelines from "./lib/Timelines.svelte";
  import Tracer from "./lib/Tracer.svelte";

  let scenario = $state.raw<Scenario | null>(null);
  let error = $state<string | null>(null);
  let idx = $state(0);

  loadFixture().then((s) => (scenario = s), (e) => (error = String(e?.message ?? e)));
</script>

<header class="bar">
  <h1 class="wordmark">Penumbra</h1>
</header>

<main>
  <p class="tag">~/site</p>
  {#if scenario}
    <Panorama {scenario} bind:idx />
    <Timelines {scenario} bind:idx />
  {:else if error}
    <p class="msg" role="alert">{error}</p>
  {:else}
    <p class="msg">Loading…</p>
  {/if}
  <Tracer />
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
