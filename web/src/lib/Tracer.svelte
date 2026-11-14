<script lang="ts">
  import { loadTracerTrack, TRACER, type Track } from "./engine";

  // shortcut: flat-sphere dip in TS, no API export yet; replace with core horizon() in T07
  const dip_deg = (-Math.acos(1737.4e3 / (1737.4e3 + TRACER.mast_m)) * 180) / Math.PI;

  let track = $state.raw<Track | null>(null);
  let error = $state<string | null>(null);
  loadTracerTrack().then((t) => (track = t), (e) => (error = String(e?.message ?? e)));

  const rows = $derived(
    track && [
      ["sun-az", "Sun azimuth", track.sun_az_deg[0]],
      ["sun-el", "Sun elevation", track.sun_el_deg[0]],
      ["earth-az", "Earth azimuth", track.earth_az_deg[0]],
      ["earth-el", "Earth elevation", track.earth_el_deg[0]],
    ] as const,
  );
</script>

<section>
  <h2>IM-2 tracer, 2026-11-14T00:00Z (computed in the browser)</h2>
  {#if error}
    <p role="alert" data-testid="tracer-error">{error}</p>
  {:else if !rows}
    <p>loading ephemeris…</p>
  {:else}
    <dl>
      {#each rows as [id, label, v] (id)}
        <dt>{label}</dt>
        <dd data-testid={id} data-value={v}>{v.toFixed(6)}°</dd>
      {/each}
      <dt>Horizon dip (mast {TRACER.mast_m} m)</dt>
      <dd data-testid="dip" data-value={dip_deg}>{dip_deg.toFixed(6)}° (no terrain yet)</dd>
      <dt>Ground height</dt>
      <dd data-testid="ground" data-value={TRACER.ground_m}>{TRACER.ground_m} m (no terrain yet)</dd>
    </dl>
  {/if}
</section>

<style>
  dl { display: grid; grid-template-columns: max-content 1fr; gap: 0.25rem 1rem; }
  dt { margin: 0; }
  dd { margin: 0; font-variant-numeric: tabular-nums; }
</style>
