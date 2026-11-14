// Screen 6 helpers: two FAKE runs on one calendar axis. Reuses PenumbraFake (v2.js).
// Fills: [data-pick=a|b] [data-links=a|b] [data-tools] [data-clocks=a|b] [data-shared='{opts}'] [data-v=name].
(function () {
  const F = () => window.PenumbraFake;
  const DAYS = 3;                       // B lands 3 days after A (FAKE scenario)
  const SC = { a: { seed: 7, start: 300 }, b: { seed: 7, start: 390 /* FAKE: layout-only phase, not date-derived */ } };
  const SITES = [['Artemis candidate regions', ['Mons Mouton', 'Mons Mouton Plateau', 'de Gerlache Rim 2', 'Peak near Cabeus B', 'Nobile Rim 1', 'Nobile Rim 2', 'Haworth', 'Malapert Massif', 'Slater Plain']],
    ['Commercial landers', ['IM-1 · Malapert A', 'IM-2 · Mons Mouton']]];
  const OOS = ['Apollo 11 · Tranquility', 'Apollo 15 · Hadley', 'Apollo 17 · Taurus-Littrow', 'Chang’e 4 · Von Kármán'];
  const sgn = (v, d = 1) => (v >= 0 ? '+' : '−') + Math.abs(v).toFixed(d);
  const hh = v => sgn(v, 0) + ' h';
  let runs;
  const R = () => runs || (runs = { a: F().run(SC.a), b: F().run(SC.b) });
  const el = (t, at, kid = '') => `<${t} ${Object.entries(at).map(([k, v]) => `${k}="${v}"`).join(' ')}>${kid}</${t}>`;

  const picker = (k, date) => `<div class="lbl">Site and landing date, ${k.toUpperCase()}</div><div class="pick"><select aria-label="Site ${k.toUpperCase()}">${SITES.map(g => `<optgroup label="${g[0]}">${g[1].map(s => `<option${s === 'Mons Mouton' ? ' selected' : ''}>${s}</option>`).join('')}</optgroup>`).join('')}<optgroup label="Apollo and other sites (disabled)">${OOS.map(s => `<option disabled>${s} · OUT_OF_COVERAGE</option>`).join('')}</optgroup></select><input type="date" aria-label="Landing date ${k.toUpperCase()}" value="${date}"></div>`;
  const links = k => `<a href="#quickmap-${k}">Open in QuickMap ↗</a><a href="#moontrek-${k}">Open in Moon Trek ↗</a>`;
  const tools = '<span class="chip">Same lander preset: Generic 1 kW ▾ (one preset, shared by A and B)</span><span>Share link holds both pairs: <code>/compare?a=-84.61,1.32@2027-01-12&amp;b=-84.61,1.32@2027-01-15&amp;preset=generic-1kw</code></span>';

  // One calendar axis from A's landing. B's lane starts DAYS later (hatched before it lands).
  function timeline(o = {}) {
    const { a, b } = R(), W = 720, L = 84, T = 30 + DAYS, x = d => L + d / T * (W - L - 8);
    const rows = o.rows || ['ill', 'con', 'op'], H = o.h || 16;
    let y = 4, g = '';
    const lethal = [];
    const lane = (k, r, off) => {
      g += el('text', { x: 0, y: y + 11, 'font-size': 12, 'font-weight': 700, fill: '#1d1d1d' }, k.toUpperCase());
      const y0 = y;
      for (const row of rows) {
        const h = row === 'ill' ? H * 1.8 : H, nm = { ill: 'Sun disc', con: 'Earth > 2°', op: 'Operable' }[row];
        g += el('text', { x: 0, y: y + h / 2 + 18, 'font-size': 11, fill: '#555' }, nm);
        g += el('rect', { x: L, y, width: W - L - 8, height: h, fill: '#e6e6e3' });
        if (off) g += el('rect', { x: L, y, width: x(off) - L, height: h, fill: 'url(#pre)' });
        const ld = isNaN(r.lethal_h) ? null : r.lethal_h / 24;
        if (row === 'ill') {
          let d = `M${x(off)} ${y + h}`;
          for (let i = 0; i < r.t.length; i += 6) d += ` L${x(r.t[i] + off).toFixed(1)} ${(y + h - r.ill[i] * h).toFixed(1)}`;
          g += el('path', { d: d + ` L${x(30 + off)} ${y + h} Z`, fill: '#555' });
        } else {
          const src = row === 'con' ? r.con : r.ill.map((f, i) => f > 0 && r.con[i] && (ld === null || r.t[i] < ld) ? 1 : 0);
          let s = null;
          for (let i = 0; i <= r.t.length; i++) {
            const on = i < r.t.length && src[i];
            if (on && s === null) s = i;
            if (!on && s !== null) { g += el('rect', { x: x(r.t[s] + off).toFixed(1), y: y + 2, width: Math.max(1, x(r.t[i - 1]) - x(r.t[s])).toFixed(1), height: h - 4, fill: row === 'con' ? '#777' : '#1d1d1d' }); s = null; }
          }
        }
        y += h + 5;
      }
      if (!isNaN(r.lethal_h)) lethal.push([k, r.lethal_h / 24 + off, y0, y]);
      y += 10;
    };
    lane('a', a, 0); lane('b', b, DAYS);
    const top = y;
    lethal.forEach(([k, d, y0, y1], i) => {
      g += el('line', { x1: x(d), x2: x(d), y1: y0, y2: y1 - 6, stroke: '#1d1d1d', 'stroke-width': 2 });
      g += el('text', { x: x(d) - 4, 'text-anchor': 'end', y: y0 + 10, 'font-size': 11, 'font-weight': 700, fill: '#1d1d1d', stroke: '#fff', 'stroke-width': 3, 'paint-order': 'stroke' }, `${k.toUpperCase()} first lethal shadow`);
    });
    for (let d = 0; d <= T; d += 3) g += el('text', { x: x(d), y: top + 8, 'text-anchor': d === 0 ? 'start' : 'middle', 'font-size': 11, fill: '#555' }, d ? `+${d}d` : 'A lands');
    g += el('line', { x1: x(DAYS), x2: x(DAYS), y1: 0, y2: top - 6, stroke: '#555', 'stroke-dasharray': '2 3' });
    const defs = '<defs><pattern id="pre" width="6" height="6" patternUnits="userSpaceOnUse" patternTransform="rotate(45)"><rect width="1.5" height="6" fill="#555" opacity=".5"/></pattern></defs>';
    return el('svg', { viewBox: `0 0 ${W} ${top + 16}`, role: 'img', 'aria-label': 'A and B timelines on one shared calendar axis' }, defs + g);
  }

  addEventListener('DOMContentLoaded', () => {
    const { a, b } = R(), f = F().fmt;
    const dOp = b.operable_d - a.operable_d, dLe = isNaN(a.lethal_h) || isNaN(b.lethal_h) ? NaN : b.lethal_h - a.lethal_h, cal = dLe + DAYS * 24;
    const q = (s, fn) => document.querySelectorAll(s).forEach(fn);
    q('[data-pick]', n => n.innerHTML = picker(n.dataset.pick, n.dataset.pick === 'a' ? '2027-01-12' : '2027-01-15'));
    q('[data-links]', n => n.innerHTML = links(n.dataset.links));
    q('[data-tools]', n => n.innerHTML = tools);
    q('[data-clocks]', n => n.innerHTML = F().t26(R()[n.dataset.clocks], { at: 0.8 }));
    q('[data-shared]', n => n.innerHTML = timeline(JSON.parse(n.dataset.shared || '{}')));
    const v = {
      opA: f.operable(a) + ' d', opB: f.operable(b) + ' d', opD: `${sgn(dOp)} d (${hh(dOp * 24)})`,
      leA: f.lethal(a), leB: f.lethal(b), leD: isNaN(dLe) ? 'n/a' : `${hh(dLe)} (calendar ${hh(cal)})`,
      darkA: f.dark(a), darkB: f.dark(b), conA: f.contact(a), conB: f.contact(b),
      sentence: `Landing ${DAYS} days later ${dOp < 0 ? 'costs' : 'gains'} ${Math.abs(Math.round(dOp * 24))} h of operable time`,
      sentence2: isNaN(dLe) ? 'No lethal shadow inside the 30 d window.' : `First lethal shadow arrives ${Math.abs(Math.round(dLe))} h ${dLe < 0 ? 'sooner' : 'later'} after landing (${Math.abs(Math.round(cal))} h ${cal < 0 ? 'sooner' : 'later'} on the calendar).`,
    };
    q('[data-v]', n => n.textContent = v[n.dataset.v]);
  });
})();
