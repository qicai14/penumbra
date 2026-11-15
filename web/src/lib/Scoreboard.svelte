<script lang="ts">
  import Explain from "./Explain.svelte";
  import { headline, loadScoreboard, rowStatus, tally, type Row } from "./scoreboard";

  let rows = $state.raw<Row[] | null>(null);
  let error = $state<string | null>(null);
  loadScoreboard().then((r) => (rows = r), (e) => (error = String(e?.message ?? e)));

  const t = $derived(rows && tally(rows));
  const mark = { pass: "✓ pass", miss: "✗ miss", "not run": "not run" } as const;
  // Display only. V2b: all 7 Barker RoIs pass; the miss is Mazarico SP-01.
  const note: Record<string, string> = { V2b: "7 of 7 Barker RoIs pass. The miss is Mazarico SP-01: our shortest night is 2.96 d vs his 2.75 d." };
  // A pass with no threshold is a reconstruction, not a validation (V4).
  const recon = (r: Row) => r.pass === true && r.threshold === null;
  const num =(v: number | null) => (v === null ? "—" : String(v));
</script>

<section aria-labelledby="scoreboard-h">
  <h2 id="scoreboard-h">Validation scoreboard</h2>
  <Explain id="sb" topic="Validation scoreboard" plain="How closely the model matches outside sources: JPL and NASA data, and real missions.">
    <p>Each row checks the model against an outside judge, such as JPL Horizons for where the Sun and Earth are, or NASA maps of which ground is lit. <strong>Pass</strong> means our error is inside the threshold; <strong>not run</strong> means the check is still waiting on data.</p>
  </Explain>
  {#if error}
    <p role="alert" data-testid="scoreboard-error">{error}</p>
  {:else if !rows || !t}
    <p class="msg" role="status">Loading…</p>
  {:else}
    <p class="verdict" data-testid="verdict">{headline(t)}</p>
    <div class="scroll">
      <table data-testid="scoreboard">
        <caption>One row per check: the outside judge, the metric compared and our number.</caption>
        <thead>
          <tr>
            <th scope="col">Check</th><th scope="col">Judge</th><th scope="col">Metric compared</th>
            <th scope="col">Threshold</th><th scope="col">Our value</th><th scope="col">Sample size</th>
            <th scope="col">Verdict</th>
          </tr>
        </thead>
        <tbody>
          {#each rows as r (r.id)}
            {@const s = rowStatus(r)}
            <tr data-testid="score-row" data-status={s}>
              <th scope="row">{r.id}</th>
              <td>{r.judge}</td>
              <td>{r.metric}{#if note[r.id]}<span class="note">{note[r.id]}</span>{/if}</td>
              <td class="n">{num(r.threshold)}</td>
              <td class="n">{num(r.value)}</td>
              <td class="n">{num(r.sample_size)}</td>
              <td class="verdict-cell" class:off={s === "not run" || recon(r)}>{recon(r) ? "reconstruction" : mark[s]}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</section>

<style>
  section { margin-top: var(--section-gap); }
  h2 { margin: 0 0 var(--spacing-16); font: var(--font-weight-bold) var(--text-heading)/var(--leading-heading) var(--font-mono); }
  .msg { color: var(--color-fog); }
  .verdict { margin: 0 0 var(--spacing-16); font: var(--font-weight-bold) var(--text-heading-lg)/var(--leading-heading-lg) var(--font-mono); }
  .scroll { overflow-x: auto; }
  table { width: 100%; border-collapse: collapse; font-size: var(--text-sm); line-height: var(--leading-sm); }
  caption { text-align: left; color: var(--color-ash); padding-bottom: var(--spacing-8); font-size: var(--text-caption); }
  th, td { border: 1px solid var(--color-iron); padding: var(--spacing-8); text-align: left; vertical-align: top; }
  thead th { color: var(--color-fog); font-weight: var(--font-weight-regular); background: var(--surface-carbon); }
  .n { text-align: right; font-variant-numeric: tabular-nums; }
  .note { display: block; margin-top: var(--spacing-8); color: var(--color-fog); font-size: var(--text-caption); }
  .off { color: var(--color-fog); }
</style>
