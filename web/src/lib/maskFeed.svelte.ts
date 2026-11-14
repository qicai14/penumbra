// One point-Sun mask feed (T22) shared by every view that paints it, so the 2D map and the 3D terrain (T23) show the same bits.
// The WASM Engine runs in a Web Worker; this class owns that worker, the coalescing and the rest-then-refine timing.
import { coalescing, MASK_STRIDE, MASK_STRIDE_COARSE, type MaskResult, type Runner } from "./mask";

export class MaskFeed {
  mask = $state.raw<MaskResult | null>(null);
  error = $state<string | null>(null);
  busy = $state(false);
  #worker: Worker | undefined;
  #calls = new Map<number, { ok: (v: any) => void; fail: (e: Error) => void }>();
  #callId = 0;

  // A crashed or unloadable worker rejects every pending call and is dropped; the next request builds a fresh one.
  #drop(why: string) {
    this.#worker?.terminate(); this.#worker = undefined;
    for (const c of this.#calls.values()) c.fail(new Error(why));
    this.#calls.clear();
  }
  #run: Runner = (req) => {
    if (!this.#worker) {
      const w = (this.#worker = new Worker(new URL("./maskWorker.ts", import.meta.url), { type: "module" }));
      w.onmessage = (e) => {
        const c = this.#calls.get(e.data.id); this.#calls.delete(e.data.id);
        if (e.data.error) c?.fail(new Error(e.data.error)); else c?.ok(e.data);
      };
      w.onerror = (e) => { if (this.#worker === w) this.#drop(e.message || "the mask worker failed to run"); };
      w.onmessageerror = () => { if (this.#worker === w) this.#drop("the mask worker sent an unreadable message"); };
    }
    return new Promise((ok, fail) => { const id = ++this.#callId; this.#calls.set(id, { ok, fail }); this.busy = true; this.#worker!.postMessage({ id, ...req }); });
  };
  #request = coalescing(this.#run,
    (r) => { this.mask = r; this.error = null; this.busy = false; },
    (e) => { this.error = String((e as Error)?.message ?? e); this.busy = false; });

  /** While the marker moves, a coarse mask (one point per 320 m) keeps the sweep going; 300 ms after it rests, the fine one replaces it. Returns the cleanup for an effect. */
  follow(regionId: string, t_unix_s: number, mastM: number): () => void {
    const req = { regionId, t_unix_s, mastM };
    this.#request({ ...req, stride: MASK_STRIDE_COARSE });
    const rest = setTimeout(() => this.#request({ ...req, stride: MASK_STRIDE }), 300);
    return () => clearTimeout(rest);
  }
  dispose() { this.#drop("disposed"); }
}
