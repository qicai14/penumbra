<script lang="ts">
  import { loadScoreboard, rowStatus, type Row } from "./scoreboard";

  let { onopen }: { onopen: () => void } = $props();
  let rows = $state.raw<Row[] | null>(null);
  let error = $state<string | null>(null);
  loadScoreboard().then((r) => (rows = r), (e) => (error = String(e?.message ?? e)));

  // "V1, V2 pass · V2b fails · V3 not run": ids grouped by status, every id from scoreboard.json.
  const text = $derived.by(() => {
    if (!rows) return "";
    const by = (s: string) => rows!.filter((r) => rowStatus(r) === s).map((r) => r.id).join(", ");
    return ([["pass", "pass"], ["miss", "fails"], ["not run", "not run"]] as const)
      .map(([s, word]) => (by(s) ? `${by(s)} ${word}` : ""))
      .filter(Boolean)
      .join(" · ");
  });
</script>

{#if error}
  <p class="strip" role="alert" data-testid="verdict-strip-error">{error}</p>
{:else if rows}
  <p class="strip">
    <span data-testid="verdict-strip">{text}</span>
    <button type="button" onclick={onopen}>Validation details</button>
  </p>
{/if}

<style>
  .strip { margin: 0 0 var(--spacing-16); color: var(--color-fog); font-size: var(--text-caption); line-height: var(--leading-caption); }
  button { font: inherit; color: var(--color-paper); background: transparent; border: 0; padding: 0; margin-left: var(--spacing-8); text-decoration: underline; cursor: pointer; }
  button:focus-visible { outline: 1px solid var(--color-paper); outline-offset: 2px; }
</style>
