<script lang="ts">
  import { untrack } from "svelte";
  // Advances the shared time marker in real time: `speed` days of scenario time per second.
  // Only `idx` is written per frame; the float position lives in a plain variable.
  let { idx = $bindable(0), t }: { idx: number; t: ArrayLike<number> } = $props();
  let playing = $state(false);
  let speed = $state(1); // days per second

  $effect(() => {
    if (!playing) return;
    const last = t.length - 1;
    if (last < 1) { playing = false; return; }
    const perDay = (86400 * last) / (t[last] - t[0]); // samples per scenario day
    const start = untrack(() => idx); // untracked: our own per-frame writes must not restart the loop
    let pos = start >= last ? 0 : start; // play at the end restarts
    let prev = performance.now();
    let raf = requestAnimationFrame(function tick(now) {
      pos = Math.min(last, pos + ((now - prev) / 1000) * speed * perDay);
      prev = now;
      const i = Math.floor(pos);
      if (i !== idx) idx = i;
      if (pos >= last) playing = false;
      else raf = requestAnimationFrame(tick);
    });
    return () => cancelAnimationFrame(raf); // pause, new scenario (t changes) or destroy
  });
</script>

<span class="play">
  <button type="button" data-testid="play" aria-pressed={playing} onclick={() => (playing = !playing)}>{playing ? "Pause" : "Play"}</button>
  <button type="button" data-testid="play-speed" aria-label="Playback speed, days per second" onclick={() => (speed = speed === 1 ? 5 : 1)}>{speed} d/s</button>
</span>

<style>
  .play { display: inline-flex; }
  button {
    font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-fog); background: transparent;
    border: 1px solid var(--color-slate); padding: var(--spacing-8) var(--spacing-16); margin-left: -1px; cursor: pointer;
  }
  button:first-child { margin-left: 0; border-radius: var(--radius-sm) 0 0 var(--radius-sm); }
  button:last-child { border-radius: 0 var(--radius-sm) var(--radius-sm) 0; }
  button[aria-pressed="true"] { color: var(--color-paper); background: var(--surface-iron); border-color: var(--color-pewter); }
  button:focus-visible { outline: 1px solid var(--color-paper); outline-offset: 2px; }
</style>
