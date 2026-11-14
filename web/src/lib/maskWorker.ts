// Web Worker (T22): owns its own WASM Engine with the shipped ephemeris and the near tile, so the mask never runs on the page thread.
import { loadMaskEngine } from "./engine";
import { checkMask, maskGrid } from "./mask";
import type { MaskRequest } from "./mask";

self.onmessage = async (e: MessageEvent<MaskRequest & { id: number }>) => {
  const { id, regionId, t_unix_s, mastM, stride } = e.data;
  try {
    const { engine, meta } = await loadMaskEngine(regionId);
    const lit = engine.point_sun_mask(t_unix_s, mastM, stride);
    const g = maskGrid(meta, stride);
    checkMask(lit, g);
    (self as unknown as Worker).postMessage({ id, lit, cols: g.cols, rows: g.rows }, [lit.buffer]);
  } catch (err: any) {
    (self as unknown as Worker).postMessage({ id, error: String(err?.message ?? err) });
  }
};
