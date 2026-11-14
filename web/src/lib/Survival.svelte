<script lang="ts">
  import type { Scenario } from "./engine";
  import { headline, presetFields, setPresetField, spanDays, type FieldKey } from "./headline";
  import type { Controls } from "./state.svelte";

  let { scenario, controls }: { scenario: Scenario; controls: Controls } = $props();

  const head = $derived(headline(scenario.survival, spanDays(scenario.count, scenario.step_s)));
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
  <header>
    <h2 id="sv-h">Survival · lander preset</h2>
    {#if scenario.fake}<span class="pill" data-testid="fake-badge-survival">FAKE DATA</span>{/if}
  </header>

  <div class="body">
    <div class="head" data-testid="headline">
      <p class="fig"><span class="cap">Operable</span><strong data-testid="headline-operable">{head.operable}</strong></p>
      <p class="fig">
        <span class="cap">First lethal shadow in</span>
        <strong data-testid="headline-lethal">{#if !head.none}✕ {/if}{head.lethal}</strong>
      </p>
    </div>

    <form class="fields" onsubmit={(e) => e.preventDefault()} aria-label="Lander preset">
      {#each fields as f (f.key)}
        <label class="row">
          <span class="name">{f.label}</span>
          <input type="number" min="0" step={f.step} max={f.key === "earth_margin_deg" ? 90 : undefined} value={f.value} data-testid="preset-{f.key}" onchange={(e) => edit(f.key, f.label, e)} />
          <span class="unit">{f.unit}</span>
          <span class="tag" data-testid="tag-{f.key}">{f.tag}</span>
        </label>
      {/each}
      <p class="row"><span class="name">Panel</span><span class="static">Sun-tracking</span></p>
      {#if refused}<p class="refusal" role="alert">{refused}</p>{/if}
    </form>
  </div>
</section>

<style>
  .panel { background: var(--surface-carbon); border-radius: var(--radius-sm); padding: var(--spacing-16); margin-top: var(--spacing-24); }
  header { display: flex; justify-content: space-between; align-items: center; gap: var(--spacing-16); margin-bottom: var(--spacing-16); }
  h2 { margin: 0; font: var(--font-weight-regular) var(--text-body)/var(--leading-body) var(--font-mono); color: var(--color-fog); }
  .pill {
    border: 1px solid var(--color-slate); border-radius: var(--radius-full); padding: 0 var(--spacing-8);
    font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-ash); text-transform: uppercase; white-space: nowrap;
  }
  .body { display: flex; flex-wrap: wrap; gap: var(--spacing-24); align-items: flex-start; }
  .head { flex: 1 1 280px; display: flex; flex-direction: column; gap: var(--spacing-16); }
  .fig { margin: 0; display: flex; flex-direction: column; }
  .cap { font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-ash); }
  strong { font: var(--font-weight-regular) var(--text-display)/var(--leading-display) var(--font-mono); color: var(--color-paper); font-variant-numeric: tabular-nums; }
  .fields { flex: 1 1 320px; display: flex; flex-direction: column; gap: var(--spacing-8); font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-ash); }
  .row { display: flex; align-items: center; gap: var(--spacing-8); margin: 0; }
  .name { flex: 0 0 150px; }
  .unit { flex: 0 0 3ch; }
  .tag { margin-left: auto; border: 1px solid var(--color-slate); border-radius: var(--radius-full); padding: 0 var(--spacing-8); color: var(--color-fog); }
  .static { color: var(--color-paper); }
  input {
    width: 9ch; font: var(--text-body)/var(--leading-body) var(--font-mono); color: var(--color-paper); background: var(--surface-iron);
    border: 1px solid var(--color-slate); border-radius: var(--radius-sm); padding: 4px var(--spacing-8); color-scheme: dark;
    font-variant-numeric: tabular-nums;
  }
  input:focus-visible { outline: 1px solid var(--color-paper); outline-offset: 2px; }
  .refusal { margin: 0; padding: var(--spacing-8) var(--spacing-16); border: 1px solid var(--color-pewter); border-radius: var(--radius-sm); background: var(--surface-iron); color: var(--color-paper); }
</style>
