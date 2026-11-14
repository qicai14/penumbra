<script lang="ts">
  import { onMount } from "svelte";
  import uPlot from "uplot";
  import "uplot/dist/uPlot.min.css";
  import init, { add } from "./lib/wasm/penumbra_wasm.js";
  import Tracer from "./lib/Tracer.svelte";

  let sum = $state<number | null>(null);
  let canvas: HTMLCanvasElement;
  let chart: HTMLDivElement;

  onMount(() => {
    init().then(() => (sum = add(2, 3)));

    const plot = new uPlot({ width: 320, height: 240, series: [{}, { label: "empty" }] }, [[], []], chart);

    // Three.js loads on demand, so it stays out of the first-load bundle.
    let renderer: { dispose(): void } | undefined;
    let gone = false;
    import("three").then((THREE) => {
      if (gone) return;
      const r = new THREE.WebGLRenderer({ canvas, antialias: true });
      r.setSize(320, 240, false);
      r.render(new THREE.Scene(), new THREE.PerspectiveCamera(50, 4 / 3, 0.1, 100));
      renderer = r;
    });

    return () => {
      gone = true;
      plot.destroy();
      renderer?.dispose();
    };
  });
</script>

<main>
  <h1>Penumbra template</h1>
  <p data-testid="sum">{sum === null ? "loading WASM…" : `2 + 3 = ${sum} (from Rust/WASM)`}</p>
  <div class="row">
    <canvas bind:this={canvas} aria-label="Empty Three.js canvas"></canvas>
    <div bind:this={chart} aria-label="Empty uPlot chart"></div>
  </div>
  <Tracer />
</main>

<style>
  main { font-family: system-ui, sans-serif; padding: 1rem; }
  .row { display: flex; gap: 1rem; flex-wrap: wrap; }
  canvas { width: 320px; height: 240px; background: #111; }
</style>
