<script module lang="ts">
  const KEY = "penumbra-tour-done";
  export const tourSeen = (): boolean => { try { return localStorage.getItem(KEY) === "1"; } catch { return false; } };
</script>

<script lang="ts">
  // Three coach marks, no backdrop: a docked card plus an outline on the real element it names.
  let { open = $bindable(false) }: { open?: boolean } = $props();
  const STEPS = [
    { sel: '[data-testid="curated"]', title: "1. Pick a site", text: "Choose a curated site, type a latitude and longitude, or click the map." },
    { sel: '[data-testid="hero-answer"]', title: "2. Read the headline", text: "Operable hours, and when the first lethal shadow arrives, for that site and date." },
    { sel: 'input[aria-label="Time within the span"]', title: "3. Scrub the timeline", text: "Drag the time slider: the horizon, charts and countdown all follow it." },
  ];
  let step = $state(0);

  function close() {
    open = false; step = 0;
    try { localStorage.setItem(KEY, "1"); } catch { /* private mode: it will show again next visit */ }
  }

  $effect(() => {
    if (!open) return;
    // The target mounts once the scenario has loaded, so look again until it exists.
    let el: HTMLElement | null = null;
    const find = () => {
      if (el?.isConnected) return;
      el = document.querySelector<HTMLElement>(STEPS[step].sel);
      if (!el) return;
      el.setAttribute("data-tour-on", "");
      el.scrollIntoView({ block: "center" });
    };
    find();
    const timer = setInterval(find, 250);
    return () => { clearInterval(timer); el?.removeAttribute("data-tour-on"); };
  });

  // The card sits after main in the DOM, so give it focus or keyboard users never reach it.
  let nextBtn = $state<HTMLButtonElement>();
  $effect(() => { if (open) nextBtn?.focus(); });
</script>

<svelte:window onkeydown={(e) => { if (open && e.key === "Escape" && !e.defaultPrevented) close(); }} />

{#if open}
  <div class="tour" role="dialog" aria-label="Quick tour" aria-modal="false" data-testid="tour">
    <div aria-live="polite">
      <h2>{STEPS[step].title}</h2>
      <p>{STEPS[step].text}</p>
    </div>
    <div class="row">
      <span class="n">{step + 1} / {STEPS.length}</span>
      <button type="button" bind:this={nextBtn} onclick={() => (step === STEPS.length - 1 ? close() : step++)} data-testid="tour-next">{step === STEPS.length - 1 ? "Done" : "Next"}</button>
      <button type="button" onclick={close} aria-label="Close tour" data-testid="tour-close">✕</button>
    </div>
  </div>
{/if}

<style>
  .tour { position: fixed; z-index: 10; right: var(--spacing-16); bottom: var(--spacing-16); width: min(22rem, calc(100vw - 2 * var(--spacing-16))); box-sizing: border-box; padding: var(--spacing-16); background: var(--surface-void); border: 1px solid var(--color-paper); }
  h2 { margin: 0 0 var(--spacing-8); font: inherit; font-weight: 700; }
  p { margin: 0 0 var(--spacing-16); color: var(--color-fog); }
  .row { display: flex; align-items: center; gap: var(--spacing-16); }
  .n { margin-right: auto; color: var(--color-ash); font-size: var(--text-caption); }
  button { font: inherit; color: var(--color-paper); background: transparent; border: 1px solid var(--color-iron); padding: var(--spacing-8) var(--spacing-16); cursor: pointer; }
  button:focus-visible { outline: 1px solid var(--color-paper); outline-offset: 2px; }
  :global([data-tour-on]) { outline: 2px solid var(--color-sun); outline-offset: 4px; }
</style>
