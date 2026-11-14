<script lang="ts">
  import { headline, loadScoreboard, rowStatus, tally, type Row } from "./scoreboard";

  let rows = $state.raw<Row[] | null>(null);
  let error = $state<string | null>(null);
  loadScoreboard().then((r) => (rows = r), (e) => (error = String(e?.message ?? e)));

  const t = $derived(rows && tally(rows));
  const mark = { pass: "✓ pass", miss: "✗ miss", "not run": "not run" } as const;
  const num = (v: number | null) => (v === null ? "—" : String(v));
</script>

<section aria-labelledby="scoreboard-h">
  <h2 id="scoreboard-h">Validation scoreboard</h2>
  {#if error}
    <p role="alert" data-testid="scoreboard-error">{error}</p>
  {:else if !rows || !t}
    <p class="msg" role="status">Loading…</p>
  {:else}
    <p class="verdict" data-testid="verdict">{headline(t)}</p>
    <ul class="strip" aria-label="Checks at a glance">
      {#each rows as r (r.id)}
        {@const s = rowStatus(r)}
        <li class="box" class:off={s === "not run"} data-status={s}>
          <b>{r.id}</b>
          <span>{mark[s]}</span>
        </li>
      {/each}
    </ul>
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
              <td>{r.metric}</td>
              <td class="n">{num(r.threshold)}</td>
              <td class="n">{num(r.value)}</td>
              <td class="n">{num(r.sample_size)}</td>
              <td class="verdict-cell" class:off={s === "not run"}>{mark[s]}</td>
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
  .strip { list-style: none; margin: 0 0 var(--spacing-24); padding: 0; display: grid; grid-template-columns: repeat(5, 1fr); gap: var(--spacing-8); }
  .box { border: 1px solid var(--color-slate); padding: var(--spacing-8); text-align: center; background: var(--surface-carbon); }
  .box.off { border-style: dashed; color: var(--color-fog); }
  .box b { display: block; font-size: var(--text-heading-sm); }
  .scroll { overflow-x: auto; }
  table { width: 100%; border-collapse: collapse; font-size: var(--text-sm); line-height: var(--leading-sm); }
  caption { text-align: left; color: var(--color-ash); padding-bottom: var(--spacing-8); font-size: var(--text-caption); }
  th, td { border: 1px solid var(--color-iron); padding: var(--spacing-8); text-align: left; vertical-align: top; }
  thead th { color: var(--color-fog); font-weight: var(--font-weight-regular); background: var(--surface-carbon); }
  .n { text-align: right; font-variant-numeric: tabular-nums; }
  .off { color: var(--color-fog); }
  @media (max-width: 600px) { .strip { grid-template-columns: repeat(3, 1fr); } }
</style>
