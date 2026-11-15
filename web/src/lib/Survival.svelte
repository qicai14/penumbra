<script lang="ts">
  import type { Scenario } from "./engine";
  import { presetFields, setPresetField, type FieldKey } from "./headline";
  import type { Controls } from "./state.svelte";

  let { scenario, controls }: { scenario: Scenario; controls: Controls } = $props();

  const fields = $derived(presetFields(scenario.preset));
  let refused = $state<string | null>(null);

  function edit(key: FieldKey, label: string, e: Event) {
    const input = e.currentTarget as HTMLInputElement;
    const next = setPresetField(scenario.preset, key, input.valueAsNumber);
    if (typeof next !== "string") { controls.preset = next; refused = null; }
    else { refused = `${label}: ${next}.`; input.value = String(fields.find((f) => f.key === key)!.value); }
  }
</script>

<section class="panel" aria-labelledby="sv-h">
  <h2 id="sv-h">Lander preset</h2>
  <form class="fields" onsubmit={(e) => e.preventDefault()} aria-label="Lander preset">
    {#each fields as f (f.key)}
      <label class="row">
        <span class="name">{f.label}</span>
        <input type="number" min="0" step={f.step} max={f.key === "earth_margin_deg" ? 90 : undefined} value={f.value} data-testid="preset-{f.key}" aria-describedby={f.note ? "note-{f.key}" : undefined} onchange={(e) => edit(f.key, f.label, e)} />
        <span class="unit">{f.unit}</span>
        <span class="tag" data-testid="tag-{f.key}">{f.tag}</span>
      </label>
      {#if f.note}<p class="note" id="note-{f.key}" data-testid="note-{f.key}">{f.note}</p>{/if}
    {/each}
    <p class="row"><span class="name">Panel</span><span class="static">Sun-tracking</span></p>
    {#if refused}<p class="refusal" role="alert">{refused}</p>{/if}
  </form>
</section>

<style>
  .panel { background: var(--surface-carbon); border-radius: var(--radius-sm); padding: var(--spacing-16); margin-top: var(--spacing-24); }
  h2 { margin: 0 0 var(--spacing-16); font: var(--font-weight-regular) var(--text-body)/var(--leading-body) var(--font-mono); color: var(--color-fog); }
  .fields { display: flex; flex-direction: column; gap: var(--spacing-8); font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-ash); }
  .row { display: flex; flex-wrap: wrap; align-items: center; gap: var(--spacing-8); margin: 0; }
  .name { flex: 0 0 150px; }
  .unit { flex: 0 0 3ch; }
  .tag { margin-left: auto; border: 1px solid var(--color-slate); border-radius: var(--radius-full); padding: 0 var(--spacing-8); color: var(--color-fog); }
  .note { margin: calc(-1 * var(--spacing-8) / 2) 0 0; font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-ash); }
  .static { color: var(--color-paper); }
  input {
    width: 9ch; font: var(--text-body)/var(--leading-body) var(--font-mono); color: var(--color-paper); background: var(--surface-iron);
    border: 1px solid var(--color-slate); border-radius: var(--radius-sm); padding: 4px var(--spacing-8); color-scheme: dark;
    font-variant-numeric: tabular-nums;
  }
  input:focus-visible { outline: 1px solid var(--color-paper); outline-offset: 2px; }
  .refusal { margin: 0; padding: var(--spacing-8) var(--spacing-16); border: 1px solid var(--color-pewter); border-radius: var(--radius-sm); background: var(--surface-iron); color: var(--color-paper); }
</style>
