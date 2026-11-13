// Penumbra lo-fi v2: one FAKE model drives every chart and number on these pages,
// so what a page draws always equals what it prints (the brief's hard rule).
// It mimics the engine's outputs (ARCHITECTURE §3.2): horizon, timeline columns, survival header.
// It is NOT lunar maths: shapes are invented for layout only.
(function () {
  const D2R = Math.PI / 180;
  const STEP_H = 1 / 6;                 // 10-minute steps, as the 30-day timeline
  const SPAN_D = 30, N = SPAN_D * 24 * 6;  // 4320 steps

  function rnd(seed) { let s = seed >>> 0 || 1; return () => (s = (s * 1664525 + 1013904223) >>> 0) / 4294967296; }

  // Horizon elevation (deg) at azimuth az. `pit` raises it, as pit_horizon does for the IM-2 replay.
  function horizonFn(seed, pit) {
    const r = rnd(seed), p = [0, 0, 0, 0].map(() => r() * 6.283);
    return az => {
      const a = az * D2R;
      let e = 0.15 + 0.75 * Math.sin(2 * a + p[0]) + 0.45 * Math.sin(3 * a + p[1]) + 0.22 * Math.sin(7 * a + p[2]) + 0.1 * Math.sin(13 * a + p[3]);
      return pit ? Math.max(e, pit) : e;
    };
  }
  // Sun circles the horizon once per lunar day; it is highest toward the north (az 0).
  const sunAt = (t, a0 = 300) => { const az = (a0 + 360 * t / 29.53) % 360; return { az, el: 0.35 + 1.6 * Math.cos(az * D2R) }; };
  // Earth sits near one azimuth and nods with libration.
  const earthAt = t => ({ az: 20 + 6 * Math.sin(2 * Math.PI * t / 27.2), el: 3.2 + 2.6 * Math.sin(2 * Math.PI * t / 27.2 + 1) });

  // Scenario → series + survival header. preset values are "assumed".
  function run(sc = {}) {
    const hor = horizonFn(sc.seed ?? 7, sc.pit);
    const mast = sc.mast ?? 2, margin = sc.margin ?? 2;
    const lift = (mast - 2) * 0.08 + (sc.peak ?? 0);  // a taller mast, or a site on a peak, lowers the horizon
    const P = { cap: 1500, start: 1200, min: 300, gen: 420, load: 180 };
    const t = [], ill = [], vis = [], con = [], soc = [];
    let E = P.start, lethal = NaN, opH = 0, sunH = 0, conH = 0, dark = 0, longest = 0;
    for (let k = 0; k < N; k++) {
      const td = k * STEP_H / 24, s = sunAt(td, sc.start ?? 300), e = earthAt(td);
      const h = hor(s.az) - lift;
      const f = Math.max(0, Math.min(1, (s.el - h) / 0.53 + 0.5));   // disc fraction, radius ~0.27°
      const c = e.el - (hor(e.az) - lift) > margin ? 1 : 0;
      t.push(td); ill.push(f); vis.push(f > 0 ? 1 : 0); con.push(c);
      if (f === 0) { dark += STEP_H; longest = Math.max(longest, dark); } else dark = 0;
      if (isNaN(lethal)) {
        const Es = E + (P.gen * f - P.load) * STEP_H;
        if (Es <= P.min) { lethal = td * 24 + STEP_H * (E - P.min) / (E - Es); E = P.min; }
        else E = Math.min(Es, P.cap);
        if (f > 0 && c) opH += STEP_H;
      }
      sunH += f * STEP_H; conH += c * STEP_H;
      soc.push(E);
    }
    return { hor, lift, t, ill, vis, con, soc, P, sun: t => sunAt(t, sc.start ?? 300), earth: earthAt,
      operable_d: opH / 24, lethal_h: lethal, longest_dark_h: longest,
      sunlit_pct: 100 * sunH / (SPAN_D * 24), contact_pct: 100 * conH / (SPAN_D * 24) };
  }

  const fmtLethal = h => isNaN(h) ? 'none in 30 d' : `T+${Math.floor(h / 24)}d ${Math.round(h % 24)}h`;
  const fmt = {
    operable: r => r.operable_d.toFixed(1),
    lethal: r => fmtLethal(r.lethal_h),
    dark: r => Math.round(r.longest_dark_h) + ' h',
    sunlit: r => Math.round(r.sunlit_pct) + '%',
    contact: r => Math.round(r.contact_pct) + '%',
  };

  // ---------- drawing (greyscale; Sun = solid black, Earth = dashed, always with a text label) ----------
  const el = (tag, a = {}, kids = '') => `<${tag} ${Object.entries(a).map(([k, v]) => `${k}="${v}"`).join(' ')}>${kids}</${tag}>`;

  function pano(r, o = {}) {
    const W = 720, H = o.h || 200, y = e => H - 22 - (e + 2) * ((H - 40) / 8), x = az => az * 2;
    let ridge = `M0 ${H - 20}`;
    for (let az = 0; az <= 360; az += 2) ridge += ` L${x(az)} ${y(r.hor(az)).toFixed(1)}`;
    ridge += ` L${W} ${H - 20} Z`;
    let sun = '';
    for (let az = 0; az <= 360; az += 3) sun += `${az ? 'L' : 'M'}${x(az)} ${y(0.35 + 1.6 * Math.cos(az * D2R)).toFixed(1)} `;
    let earth = '';
    for (let i = 0; i <= 60; i++) { const e = r.earth(i * 27.2 / 60); earth += `${i ? 'L' : 'M'}${x(e.az).toFixed(1)} ${y(e.el).toFixed(1)} `; }
    const now = r.sun(o.at ?? 0.8), ne = r.earth(o.at ?? 0.8);
    let g = el('rect', { width: W, height: H, fill: '#f4f4f2' });
    for (let e = -2; e <= 6; e += 2) g += el('line', { x1: 0, x2: W, y1: y(e), y2: y(e), stroke: '#ddd' }) + el('text', { x: 4, y: y(e) - 3, 'font-size': 11, fill: '#555' }, `${e}°`);
    g += el('path', { d: ridge, fill: '#b9b9b6' });
    if (o.pitLine) g += el('line', { x1: 0, x2: W, y1: y(o.pitLine), y2: y(o.pitLine), stroke: '#1d1d1d', 'stroke-width': 1.5, 'stroke-dasharray': '2 3' }) + el('text', { x: 470, y: y(o.pitLine) - 5, 'font-size': 11, fill: '#1d1d1d' }, `crater rim, pit model (${o.pitLine}°)`);
    g += el('path', { d: sun, fill: 'none', stroke: '#1d1d1d', 'stroke-width': 2.5 }) + el('text', { x: x(300), y: y(1.4) - 8, 'font-size': 12, 'font-weight': 600, fill: '#1d1d1d' }, 'Sun track');
    g += el('path', { d: earth, fill: 'none', stroke: '#555', 'stroke-width': 2, 'stroke-dasharray': '6 4' }) + el('text', { x: x(ne.az) + 22, y: y(5.5), 'font-size': 12, fill: '#333' }, 'Earth (libration loop)');
    g += el('circle', { cx: x(now.az), cy: y(now.el), r: 6, fill: '#fff', stroke: '#1d1d1d', 'stroke-width': 2.5 });
    g += el('circle', { cx: x(ne.az), cy: y(ne.el), r: 5, fill: '#555' });
    ['N', 'E', 'S', 'W', 'N'].forEach((l, i) => g += el('text', { x: Math.min(W - 10, Math.max(2, i * 180 - 4)), y: H - 6, 'font-size': 11, fill: '#555' }, l));
    return el('svg', { viewBox: `0 0 ${W} ${H}`, role: 'img', 'aria-label': 'Horizon panorama with Sun and Earth tracks' }, g);
  }

  // Timeline lanes + battery. bands = ± band at each sunset, hatched.
  function lanes(r, o = {}) {
    const W = 720, L = 92, x = td => L + (td / SPAN_D) * (W - L - 8);
    const rows = o.rows || ['ill', 'con', 'op', 'soc'];
    const rh = { ill: 34, vis: 16, con: 16, op: 16, soc: 70 };
    const name = { ill: 'Sun disc 0–1', vis: 'Sun visible', con: 'Earth > 2°', op: 'Operable', soc: 'Battery Wh' };
    let yy = 6, g = '';
    const step = 6; // draw every hour
    const lethalD = isNaN(r.lethal_h) ? null : r.lethal_h / 24;
    for (const row of rows) {
      const h = rh[row];
      g += el('text', { x: 0, y: yy + h / 2 + 4, 'font-size': 11, fill: '#555' }, name[row]);
      g += el('rect', { x: L, y: yy, width: W - L - 8, height: h, fill: '#e6e6e3' });
      if (row === 'ill') {
        let d = `M${L} ${yy + h}`;
        for (let k = 0; k < N; k += step) d += ` L${x(r.t[k]).toFixed(1)} ${(yy + h - r.ill[k] * h).toFixed(1)}`;
        g += el('path', { d: d + ` L${W - 8} ${yy + h} Z`, fill: '#555' });
      } else if (row === 'soc') {
        const sy = w => yy + h - (w / r.P.cap) * h;
        let d = '';
        for (let k = 0; k < N; k += step) d += `${k ? 'L' : 'M'}${x(r.t[k]).toFixed(1)} ${sy(r.soc[k]).toFixed(1)} `;
        g += el('line', { x1: L, x2: W - 8, y1: sy(r.P.min), y2: sy(r.P.min), stroke: '#1d1d1d', 'stroke-dasharray': '3 3' }) + el('text', { x: L + 6, y: sy(r.P.min) - 4, 'font-size': 11, fill: '#333' }, 'minimum charge');
        if (lethalD !== null) g += el('rect', { x: x(lethalD), y: yy, width: W - 8 - x(lethalD), height: h, fill: 'url(#dead)' });
        g += el('path', { d, fill: 'none', stroke: '#1d1d1d', 'stroke-width': 2 });
      } else {
        const src = row === 'con' ? r.con : row === 'vis' ? r.vis : r.ill.map((f, k) => (f > 0 && r.con[k] && (lethalD === null || r.t[k] < lethalD)) ? 1 : 0);
        let start = null;
        for (let k = 0; k <= N; k++) {
          const on = k < N && src[k];
          if (on && start === null) start = k;
          if (!on && start !== null) { g += el('rect', { x: x(r.t[start]).toFixed(1), y: yy + 2, width: Math.max(1, x(r.t[k - 1]) - x(r.t[start])).toFixed(1), height: h - 4, fill: row === 'con' ? '#777' : '#1d1d1d' }); start = null; }
        }
      }
      yy += h + 8;
    }
    // ± bands at each sunset (illumination falls to 0)
    if (o.bands !== false) for (let k = 1; k < N; k++) if (r.ill[k - 1] > 0 && r.ill[k] === 0) g += el('rect', { x: x(r.t[k]) - 4, y: 6, width: 8, height: yy - 14, fill: 'url(#band)' });
    if (lethalD !== null) g += el('line', { x1: x(lethalD), x2: x(lethalD), y1: 2, y2: yy - 6, stroke: '#1d1d1d', 'stroke-width': 2 }) + el('text', { x: lethalD > 18 ? x(lethalD) - 5 : x(lethalD) + 5, 'text-anchor': lethalD > 18 ? 'end' : 'start', y: 18, 'font-size': 12, 'font-weight': 700, fill: '#1d1d1d', stroke: '#fff', 'stroke-width': 3, 'paint-order': 'stroke' }, `first lethal shadow ${fmtLethal(r.lethal_h)} ±40 min`);
    const cur = x(o.at ?? 0.8);
    g += el('line', { x1: cur, x2: cur, y1: 0, y2: yy - 6, stroke: '#1d1d1d', 'stroke-dasharray': '1 2' });
    for (let d = 0; d <= SPAN_D; d += 5) g += el('text', { x: x(d), 'text-anchor': d === 0 ? 'start' : d === SPAN_D ? 'end' : 'middle', y: yy + 8, 'font-size': 11, fill: '#555' }, d ? `+${d}d` : 'land');
    const defs = `<defs><pattern id="band" width="4" height="4" patternUnits="userSpaceOnUse" patternTransform="rotate(45)"><rect width="2" height="4" fill="#1d1d1d" opacity=".35"/></pattern><pattern id="dead" width="8" height="8" patternUnits="userSpaceOnUse" patternTransform="rotate(-45)"><rect width="3" height="8" fill="#1d1d1d" opacity=".18"/></pattern></defs>`;
    return el('svg', { viewBox: `0 0 ${W} ${yy + 14}`, role: 'img', 'aria-label': 'Timelines and battery charge over 30 days' }, defs + g);
  }

  // Region map: a FAKE smooth "operable days" field, 6 grey bins, with an optional pin and view cone.
  function heat(o = {}) {
    const C = o.cols || 30, R = o.rows || 20, W = 600, H = W * R / C, cw = W / C;
    const v = (i, j) => 15 + 9 * Math.sin(i / 4.1) * Math.cos(j / 3.3) + 5 * Math.sin((i + j) / 6);
    const greys = ['#2a2a2a', '#555', '#808080', '#aaa', '#cfcfcf', '#f0f0f0'];
    let g = '';
    for (let j = 0; j < R; j++) for (let i = 0; i < C; i++) {
      const b = Math.max(0, Math.min(5, Math.floor(v(i, j) / 5)));
      g += el('rect', { x: (i * cw).toFixed(1), y: (j * cw).toFixed(1), width: cw.toFixed(1), height: cw.toFixed(1), fill: greys[b] });
    }
    if (o.nodata) g += el('rect', { x: W * 0.72, y: 0, width: W * 0.28, height: H * 0.35, fill: 'url(#nod)' }) + el('text', { x: W * 0.74, y: 22, 'font-size': 13, fill: '#1d1d1d', 'font-weight': 600 }, 'no DEM coverage');
    for (const p of o.pins || []) {
      const px = p.x * W, py = p.y * H;
      if (p.cone) g += el('path', { d: `M${px} ${py} L${px + 90} ${py - 60} A110 110 0 0 1 ${px + 105} ${py + 35} Z`, fill: '#fff', opacity: 0.35, stroke: '#1d1d1d', 'stroke-dasharray': '3 3' });
      g += el('circle', { cx: px, cy: py, r: 9, fill: '#fff', stroke: '#1d1d1d', 'stroke-width': 3 }) + el('text', { x: px + 13, y: py + 5, 'font-size': 14, 'font-weight': 700, fill: '#fff', stroke: '#1d1d1d', 'stroke-width': 3, 'paint-order': 'stroke' }, p.label || '');
    }
    if (o.ring) { const [rx, ry, rr] = o.ring; g += el('circle', { cx: rx * W, cy: ry * H, r: rr, fill: 'none', stroke: '#fff', 'stroke-width': 2, 'stroke-dasharray': '6 4' }); }
    const defs = `<defs><pattern id="nod" width="10" height="10" patternUnits="userSpaceOnUse" patternTransform="rotate(45)"><rect width="10" height="10" fill="#fff"/><rect width="4" height="10" fill="#1d1d1d" opacity=".25"/></pattern></defs>`;
    return el('svg', { viewBox: `0 0 ${W} ${H.toFixed(0)}`, role: 'img', 'aria-label': 'Region map shaded by operable days' }, defs + g);
  }

  // T26 readout strip: Sun and Earth elevation now, plus countdowns to the next sunset and Earthset.
  // Never blank, never negative: if the event is not in the window it says "No sunset in N d";
  // if the body is already down it counts to the next rise instead.
  function t26(r, o = {}) {
    const at = o.at ?? 0.8, k0 = Math.round(at * 24 / STEP_H), left = Math.floor(SPAN_D - at);
    const dur = h => h < 24 ? `${Math.floor(h)}:${String(Math.round((h % 1) * 60)).padStart(2, '0')}` : `${Math.floor(h / 24)}d ${String(Math.round(h % 24)).padStart(2, '0')}h`;
    const unit = h => h < 24 ? 'h:m' : 'days, hours';
    const next = (src, from) => { for (let k = k0 + 1; k < N; k++) if (!!src[k] !== from) return (k - k0) * STEP_H; return null; };
    const count = (src, down, up, none, pm) => {
      const on = !!src[k0], h = next(src, on);
      if (h === null) return { lbl: on ? `Next ${down} in` : `Next ${up} in`, num: `None`, sub: `${on ? none : `no ${up}`} in ${left} d (window ends)` };
      return { lbl: on ? `Next ${down} in` : `${down[0].toUpperCase() + down.slice(1)} now · ${up} in`, num: dur(h), sub: `${unit(h)} · ±${pm} min (assumed σ<sub>z</sub>)` };
    };
    const s = r.sun(at), e = r.earth(at), sAbove = s.el - (r.hor(s.az) - r.lift), eAbove = e.el - (r.hor(e.az) - r.lift) - 2;
    const sb = r.ill.map(f => f > 0), cells = [
      { lbl: 'Sun elevation', num: `${s.el.toFixed(2)}°`, sub: sAbove >= 0 ? `${sAbove.toFixed(2)}° above the ridge` : `${(-sAbove).toFixed(2)}° below the ridge` },
      count(sb, 'sunset', 'sunrise', 'no sunset', 40),
      { lbl: 'Earth elevation', num: `${e.el.toFixed(1)}°`, sub: eAbove >= 0 ? `${eAbove.toFixed(1)}° above the 2° margin` : `${(-eAbove).toFixed(1)}° under the 2° margin` },
      count(r.con, 'Earthset', 'Earthrise', 'no Earthset', 25),
    ];
    return `<div class="t26" role="group" aria-label="Sun and Earth now, with countdowns">${cells.map(c =>
      `<div class="box"><div class="lbl">${c.lbl}</div><span class="num">${c.num}</span><span class="${c.num === 'None' ? 'muted' : 'pm'}">${c.sub}</span></div>`).join('')}</div>`;
  }

  // "Now" values for hand-written boxes, so no page types a number the model disagrees with.
  const K_NOW = Math.round(0.8 * 24 / STEP_H);
  const nextSunset = r => { for (let k = K_NOW + 1; k < N; k++) if (r.ill[k - 1] > 0 && r.ill[k] === 0) return (k - K_NOW) * STEP_H; return null; };
  Object.assign(fmt, {
    sunEl: r => r.sun(0.8).el.toFixed(2) + '°',
    disc: r => Math.round(r.ill[K_NOW] * 100) + '%',
    lit: r => r.ill[K_NOW] > 0 ? 'yes' : 'no',
    sunset: r => { const h = nextSunset(r); return h === null ? `none in ${Math.floor(SPAN_D - 0.8)} d` : h < 24 ? `${Math.floor(h)} h ${Math.round((h % 1) * 60)} m` : `${Math.floor(h / 24)} d ${Math.round(h % 24)} h`; },
  });

  // T13 Must (PLAN-DRAFT 387–388): Limitations list + every dataset with its role and citation.
  const LIMITS = [
    'No limb darkening: the Sun disc is uniformly bright.',
    'Terrain smaller than one 20 m pixel is not in the DEM (the IM-2 crater needs the pit model).',
    'The Earth margin (2°) and the terrain height error (σ<sub>z</sub> 0.5 m) are assumed.',
    'No charge or discharge efficiency in the battery model.',
    'Lander values are assumed unless marked cited.',
    'No atmosphere: the horizon is terrain or the limb; no refraction, no twilight.',
    'Earthshine is out of scope; V5 checks sunlight only.',
  ];
  const DATASETS = [
    ['LOLA DEM, 20 m · <span class="mono">LDEM_80S_20MPP_ADJ</span>', 'Horizon, near field (first 7 km)', 'NASA PGDA · Barker et al. 2021'],
    ['LOLA DEM, 240 m · <span class="mono">LDEM_60S_240MPP_ADJ</span>', 'Horizon, far field (to 200 km)', 'NASA PGDA · Barker et al. 2021'],
    ['SPICE kernels · <span class="mono">de440.bsp</span>, Moon PA and FK', 'Sun and Earth positions', 'NASA NAIF · Park et al. 2021'],
    ['JPL Horizons API', 'V1 judge: positions', 'NASA JPL SSD'],
    ['Avg. visibility · <span class="mono">AVGVISIB_85S_060M_201608</span> + _EARTH', 'V2 judge: light and Earth over 18.6 y', 'NASA PGDA · Mazarico et al. 2011'],
    ['LROC NAC images', 'V3 judge: shadow shapes (stretch)', 'NASA LROC · Robinson et al. 2010'],
  ];
  const limits = () => `<section class="box limits"><div class="lbl">Limitations</div><ul>${LIMITS.map(l => `<li>${l}</li>`).join('')}</ul></section>`
    + `<section class="box limits"><div class="lbl">Data we use · role and citation</div><table><tr><th>Dataset</th><th>Role</th><th>Source · citation</th></tr>${DATASETS.map(d => `<tr><td>${d[0]}</td><td>${d[1]}</td><td>${d[2]}</td></tr>`).join('')}</table></section>`;

  const api = { run, pano, lanes, heat, t26, limits, fmt, fmtLethal };
  if (typeof window === 'undefined') { globalThis.PenumbraFake = api; return; }
  window.PenumbraFake = api;

  // Auto-fill: <div data-pano="key"> / data-lanes / data-heat='{"pins":[...]}', and <b data-out="key.metric">.
  // Scenarios are declared in a <script type="application/json" id="scenarios"> on each page.
  addEventListener('DOMContentLoaded', () => {
    const tag = document.getElementById('scenarios');
    const sc = tag ? JSON.parse(tag.textContent) : { main: {} };
    const runs = Object.fromEntries(Object.entries(sc).map(([k, v]) => [k, run(v)]));
    const opt = n => { try { return JSON.parse(n || '{}'); } catch { return {}; } };
    document.querySelectorAll('[data-pano]').forEach(n => n.innerHTML = pano(runs[n.dataset.pano], opt(n.dataset.opt)));
    document.querySelectorAll('[data-lanes]').forEach(n => n.innerHTML = lanes(runs[n.dataset.lanes], opt(n.dataset.opt)));
    document.querySelectorAll('[data-t26]').forEach(n => n.innerHTML = t26(runs[n.dataset.t26], opt(n.dataset.opt)));
    document.querySelectorAll('[data-limits]').forEach(n => n.innerHTML = limits());
    document.querySelectorAll('[data-heat]').forEach(n => n.innerHTML = heat(opt(n.dataset.heat)));
    document.querySelectorAll('[data-out]').forEach(n => { const [k, m] = n.dataset.out.split('.'); n.textContent = fmt[m](runs[k]); });
  });
  // After every page script has drawn its charts: keep each chart label at 11 px or more on screen.
  addEventListener('load', () => {
    const keep = svg => {
      const w = svg.getBoundingClientRect().width, vb = svg.viewBox.baseVal.width;
      if (!w || !vb) return;
      svg.querySelectorAll('text').forEach(t => {
        const base = +(t.dataset.fs || (t.dataset.fs = t.getAttribute('font-size') || 11));
        t.setAttribute('font-size', Math.max(base, 11 * vb / w).toFixed(1));
      });
    };
    const ro = new ResizeObserver(es => es.forEach(e => keep(e.target)));
    document.querySelectorAll('.svgfill svg').forEach(s => { keep(s); ro.observe(s); });
  });
})();
