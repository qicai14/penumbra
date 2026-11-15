<script lang="ts">
  import Explain from "./Explain.svelte";
  import * as THREE from "three";
  import { OrbitControls } from "three/addons/controls/OrbitControls.js";
  import { latlonToXy, type EphSegment, type NearTile, type Region } from "./engine";
  import { maskAt, maskGrid, MASK_STRIDE } from "./mask";
  import type { MaskFeed } from "./maskFeed.svelte";
  import type { Controls } from "./state.svelte";
  import { fmtEl, fmtUtc } from "./panorama";
  import { rgb } from "./color";
  import { EXAGGERATIONS, exaggerationLabel, gridIndices, gridPositions, groundAt, heightRange, sunDirection, toWorld, vertexColors } from "./terrain3d";

  // The same point-Sun mask the site picker paints (`feed`), at the same time marker `t`, with the Sun's azimuth and elevation
  // read from the model's track at that marker. Nothing here computes light: a vertex is ember where the mask says lit.
  let { controls, tile, feed, t, sunAz, sunEl, sunLon, idx = $bindable(0), count }: {
    controls: Controls; tile: NearTile & { regions: Region[]; eph: EphSegment[] }; feed: MaskFeed;
    t: number | null; sunAz: number | null; sunEl: number | null; sunLon: number | null; // sunLon: the longitude the track was computed for
    idx?: number; count: number; // the shared time marker: dragging here also moves the Site view
  } = $props();

  const PIN_H_M = 700, SUN_LEN_M = 4000, VIEW_DIST_M = 14000;
  let canvas: HTMLCanvasElement;
  let box: HTMLDivElement;
  let exaggeration = $state<number>(1);
  let glError = $state<string | null>(null);
  let nodata = $state(0);
  let moved = $state(false); // the camera left its home view, so Reset view is worth pressing
  let homing = false; // home() moves the camera too; that move must not count as the user's

  const m = $derived(tile.meta);
  const shown = $derived(feed.mask && feed.mask.regionId === tile.region.id ? feed.mask : null);
  const siteXy = $derived(latlonToXy(controls.lat, controls.lon));
  const siteLit = $derived(shown ? maskAt(shown, m, siteXy[0], siteXy[1]) : null);
  const range = $derived(heightRange(m, tile.data));
  const litCount = $derived(shown ? shown.lit.reduce((n, v) => n + (v ? 1 : 0), 0) : 0);

  type Entry = { geo: THREE.BufferGeometry; heights: Float32Array; colors: Float32Array; nodata: number };
  type Gl = {
    renderer: THREE.WebGLRenderer; scene: THREE.Scene; camera: THREE.PerspectiveCamera; controls: OrbitControls;
    mesh: THREE.Mesh; pin: THREE.Group; sun: THREE.Group; sunRgb: [number, number, number];
    geoms: Map<number, Entry>; key: string; render: () => void; at: THREE.Vector3 | null;
  };
  let gl: Gl | undefined; // plain, not reactive: three.js owns it; the effects below push state into it

  function freeGeoms(g: Gl) { for (const e of g.geoms.values()) e.geo.dispose(); g.geoms.clear(); }
  const disposeTree = (o: THREE.Object3D) => o.traverse((c) => { const x = c as THREE.Mesh; x.geometry?.dispose(); (x.material as THREE.Material | undefined)?.dispose(); });

  // Scene, camera, controls and teardown. Renders on demand (no animation loop), so a still view costs nothing and there is no motion to reduce.
  $effect(() => {
    let renderer: THREE.WebGLRenderer;
    try { renderer = new THREE.WebGLRenderer({ canvas, antialias: true }); }
    catch { glError = "This browser could not start WebGL, so the 3D terrain is unavailable. The 2D map above shows the same mask."; return; }
    const s = getComputedStyle(canvas);
    const c = (n: string) => s.getPropertyValue(n).trim();
    renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2));
    const scene = new THREE.Scene();
    scene.background = new THREE.Color(c("--color-carbon"));
    const camera = new THREE.PerspectiveCamera(40, 1, 50, 200000);
    const ctl = new OrbitControls(camera, canvas);
    ctl.enableDamping = false; ctl.screenSpacePanning = false;
    ctl.minDistance = 300; ctl.maxDistance = 60000; ctl.maxPolarAngle = Math.PI * 0.495;
    ctl.listenToKeyEvents(canvas);
    ctl.touches = { ONE: null as unknown as THREE.TOUCH, TWO: THREE.TOUCH.DOLLY_ROTATE }; // one finger scrolls the page; two rotate and zoom
    canvas.style.touchAction = "pan-y"; // OrbitControls sets "none", which traps the page scroll under one finger
    const keepTwoFinger = (e: TouchEvent) => { if (e.touches.length >= 2) e.preventDefault(); }; // pan-y would let a vertical two-finger drag (tilt) scroll the page
    canvas.addEventListener("touchmove", keepTwoFinger, { passive: false });

    // Unlit: the vertex colour is the whole shading, so no GPU light or shadow map can add a shadow the model did not compute.
    const mesh = new THREE.Mesh(new THREE.BufferGeometry(), new THREE.MeshBasicMaterial({ vertexColors: true }));
    mesh.frustumCulled = false;
    scene.add(mesh);
    const paper = new THREE.MeshBasicMaterial({ color: c("--color-paper") });
    const pin = new THREE.Group();
    const stem = new THREE.Mesh(new THREE.CylinderGeometry(18, 18, PIN_H_M, 8), paper); stem.position.y = PIN_H_M / 2;
    const head = new THREE.Mesh(new THREE.SphereGeometry(70, 16, 12), paper); head.position.y = PIN_H_M;
    pin.add(stem, head); scene.add(pin);
    const ember = new THREE.MeshBasicMaterial({ color: c("--color-sun") });
    const sun = new THREE.Group();
    const shaft = new THREE.Mesh(new THREE.CylinderGeometry(22, 22, SUN_LEN_M - 400, 8), ember); shaft.position.y = (SUN_LEN_M - 400) / 2;
    const tip = new THREE.Mesh(new THREE.ConeGeometry(110, 400, 12), ember); tip.position.y = SUN_LEN_M - 200;
    sun.add(shaft, tip); scene.add(sun);

    let raf = 0;
    const render = () => { if (!raf) raf = requestAnimationFrame(() => { raf = 0; renderer.render(scene, camera); }); };
    ctl.addEventListener("change", () => { render(); if (!homing) moved = true; });
    const resize = () => {
      const w = box.clientWidth, h = box.clientHeight;
      if (!w || !h) return;
      renderer.setSize(w, h, false); camera.aspect = w / h; camera.updateProjectionMatrix(); render();
    };
    const ro = new ResizeObserver(resize); ro.observe(box); resize();

    gl = { renderer, scene, camera, controls: ctl, mesh, pin, sun, sunRgb: rgb(c("--color-sun")), geoms: new Map(), key: "", render, at: null };
    return () => {
      cancelAnimationFrame(raf); ro.disconnect(); ctl.dispose(); canvas.removeEventListener("touchmove", keepTwoFinger);
      if (gl) freeGeoms(gl);
      disposeTree(scene);
      renderer.dispose(); renderer.forceContextLoss();
      gl = undefined;
    };
  });

  // The mesh: one vertex per mask point at the mask's own stride, coloured by that point's mask bit. Geometries are cached per stride
  // (the coarse and fine masks each have their own grid) and replaced when the tile or the vertical scale changes.
  $effect(() => {
    const g = gl; if (!g) return;
    const key = `${tile.region.id}|${exaggeration}`;
    if (g.key !== key) { freeGeoms(g); g.key = key; }
    const stride = shown?.stride ?? MASK_STRIDE;
    let e = g.geoms.get(stride);
    if (!e) {
      const grid = maskGrid(m, stride);
      const p = gridPositions(m, tile.data, grid, exaggeration, range.lo_m);
      const geo = new THREE.BufferGeometry();
      geo.setAttribute("position", new THREE.BufferAttribute(p.positions, 3));
      const colors = new Float32Array(grid.cols * grid.rows * 3);
      geo.setAttribute("color", new THREE.BufferAttribute(colors, 3));
      geo.setIndex(new THREE.BufferAttribute(gridIndices(grid.cols, grid.rows), 1));
      geo.computeBoundingSphere();
      e = { geo, heights: p.heights_m, colors, nodata: p.nodata };
      g.geoms.set(stride, e);
    }
    vertexColors(e.heights, range, shown ? shown.lit : null, g.sunRgb, e.colors);
    e.geo.getAttribute("color").needsUpdate = true;
    g.mesh.geometry = e.geo;
    nodata = e.nodata;
    g.render();
  });

  // The pin and the Sun arrow follow the site and the marker. A new site carries the camera with it; a new time only turns the arrow.
  function home(g: Gl, at: THREE.Vector3) {
    const el = (35 * Math.PI) / 180;
    g.controls.target.copy(at);
    g.camera.position.set(at.x, at.y + VIEW_DIST_M * Math.sin(el), at.z + VIEW_DIST_M * Math.cos(el)); // from the south, so north is away
    homing = true; g.controls.update(); homing = false; moved = false; g.render();
  }
  function resetView() { if (gl?.at) home(gl, gl.at); }
  $effect(() => {
    const g = gl; if (!g) return;
    const ground = groundAt(m, tile.data, siteXy[0], siteXy[1]);
    const [wx, wz] = toWorld(m, siteXy[0], siteXy[1]);
    const base = new THREE.Vector3(wx, (ground ?? 0) * exaggeration, wz);
    g.pin.visible = ground !== null;
    g.pin.position.copy(base);
    g.sun.position.set(wx, base.y + PIN_H_M, wz);
    const up = sunAz !== null && sunEl !== null && sunLon !== null && sunEl > 0; // the arrow is the model's Sun, for the longitude it was computed at
    g.sun.visible = up;
    if (up) g.sun.quaternion.setFromUnitVectors(new THREE.Vector3(0, 1, 0), new THREE.Vector3(...sunDirection(sunAz, sunEl, sunLon)));
    if (!g.at) home(g, base);
    else if (!g.at.equals(base)) { const d = base.clone().sub(g.at); g.camera.position.add(d); g.controls.target.add(d); g.controls.update(); }
    g.at = base;
    g.render();
  });
</script>

<section class="panel" aria-labelledby="t3-h">
  <header>
    <h2 id="t3-h">3D terrain</h2>
  </header>
  <Explain id="t3" topic="3D terrain" plain="The ground around the site in 3D, with sunlit and shadowed ground at the chosen time.">
    <p><strong>Orange</strong> ground has the Sun's centre above the terrain; <strong>dark</strong> ground has it hidden behind a hill. The pin is the site and the arrow points to the Sun.</p>
    <p>Drag to turn the view and scroll to zoom. Move the time slider to watch shadows sweep across the ground.</p>
  </Explain>

  <figure data-testid="terrain-3d" data-t={shown?.t_unix_s ?? ""} data-stride={shown?.stride ?? ""} data-lit={litCount} data-vertices={shown ? shown.cols * shown.rows : ""} data-exaggeration={exaggeration} data-gl={glError ? "off" : "on"}>
    <div class="stage" bind:this={box}>
      <canvas
        bind:this={canvas}
        tabindex="0"
        aria-describedby="t3-note t3-caveat"
        aria-label="3D height mesh of the {tile.region.name} near tile at {shown ? fmtUtc(shown.t_unix_s) : t === null ? 'the time marker' : fmtUtc(t)}. Each mesh point is ember where the Sun's centre is above the terrain and dark where it is below. The pin is the site; the ember arrow points to the Sun. Drag to orbit, scroll to zoom, arrow keys to pan."
      ></canvas>
    </div>
    <label class="time">
      <span>Time</span>
      <input data-testid="t3-time" type="range" min="0" max={count - 1} step="1" bind:value={idx} aria-label="Time within the span" />
      <output>{t === null ? "…" : fmtUtc(t)}</output>
    </label>
    {#if glError}<p class="refusal" role="alert">{glError}</p>{/if}
    {#if feed.error}<p class="refusal" role="alert">Mask unavailable. {feed.error}</p>{/if}
    <div class="legend">
      <p class="sun" id="t3-note">
        Sun (model): azimuth {sunAz === null ? "…" : `${sunAz.toFixed(1)}°`}, elevation {sunEl === null ? "…" : fmtEl(sunEl)}{#if sunEl !== null && sunEl <= 0}{" · Sun below the horizon"}{/if}
        · pin: {siteLit === null ? (feed.error ? "no mask yet" : "computing") : siteLit ? "lit" : "in shadow"}{#if shown && t !== null && shown.t_unix_s !== t}{" (mask for "}{fmtUtc(shown.t_unix_s)}{")"}{/if}
      </p>
      <p class="note" id="t3-caveat" data-testid="t3-caveat">Point-Sun mask, not illumination; points near the tile edge can read lit.</p>
      {#if nodata > 0}<p class="note">{nodata} mesh points have no terrain data and sit at the tile's lowest height.</p>{/if}
    </div>
    <figcaption>
      <label>Vertical scale
        <select data-testid="t3-exaggeration" bind:value={exaggeration}>
          {#each EXAGGERATIONS as k (k)}<option value={k}>{exaggerationLabel(k)}</option>{/each}
        </select>
      </label>
      <button type="button" class={{ moved }} data-testid="t3-reset" data-moved={moved} onclick={resetView}>Reset view</button>
      {#if exaggeration !== 1}<span class="warn" role="note">Heights drawn ×{exaggeration}: slopes look steeper than they are.</span>{/if}
    </figcaption>
  </figure>
</section>

<style>
  .panel { background: var(--surface-carbon); border-radius: var(--radius-sm); padding: var(--spacing-16); margin-bottom: var(--spacing-16); }
  header { display: flex; justify-content: space-between; align-items: center; gap: var(--spacing-16); margin-bottom: var(--spacing-16); }
  h2 { margin: 0; font: var(--font-weight-regular) var(--text-body)/var(--leading-body) var(--font-mono); color: var(--color-fog); }
  figure { margin: 0; }
  .time { display: flex; flex-wrap: wrap; align-items: center; gap: var(--spacing-8); margin-top: var(--spacing-8); font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-ash); }
  .time input { flex: 1 1 200px; accent-color: var(--color-fog); }
  .time output { color: var(--color-paper); font-variant-numeric: tabular-nums; }
  .stage { position: relative; width: 100%; height: clamp(280px, 52vw, 520px); border-radius: var(--radius-sm); overflow: hidden; background: var(--color-carbon); }
  canvas { display: block; width: 100%; height: 100%; cursor: grab; touch-action: pan-y; }
  canvas:active { cursor: grabbing; }
  canvas:focus-visible { outline: 1px solid var(--color-paper); outline-offset: 2px; }
  .legend { margin-top: var(--spacing-8); font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-fog); }
  .legend p { margin: 0 0 4px; max-width: 75ch; }
  .legend .sun { font-variant-numeric: tabular-nums; }
  .legend .note { color: var(--color-ash); }
  figcaption { margin-top: var(--spacing-8); display: flex; flex-wrap: wrap; align-items: center; gap: var(--spacing-16); font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-ash); }
  figcaption label { display: flex; align-items: center; gap: var(--spacing-8); }
  .warn { color: var(--color-paper); }
  select {
    font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-paper); background: var(--surface-iron);
    border: 1px solid var(--color-slate); border-radius: var(--radius-sm); padding: 4px var(--spacing-8); color-scheme: dark;
  }
  button {
    font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-fog); background: transparent;
    border: 1px solid var(--color-slate); border-radius: var(--radius-sm); padding: var(--spacing-8) var(--spacing-16); cursor: pointer;
  }
  button.moved { color: var(--color-void); background: var(--color-paper); border-color: var(--color-paper); font-weight: var(--font-weight-bold); } /* Axiom colours inverted: paper box, void text */
  button:focus-visible, select:focus-visible { outline: 1px solid var(--color-paper); outline-offset: 2px; }
  .refusal { margin: var(--spacing-8) 0 0; padding: var(--spacing-8) var(--spacing-16); border: 1px solid var(--color-pewter); border-radius: var(--radius-sm); background: var(--surface-iron); color: var(--color-paper); }
</style>
