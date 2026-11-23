// E10 journey sweep: visits every built station and state in all three themes and two tier-2 combinations,
// at 1440x900, 320x640 and 720x450 CSS px (200 % of 1440x900), and reports defect classes everywhere.
const frame = document.getElementById('sw-frame')
const only = new URLSearchParams(location.search).get('only')
const VIEWS = [[1440, 900], [320, 640], [720, 450]].filter(([w]) => !only || String(w) === only)
const THEMES = { dark: { theme: 'dark' }, light: { theme: 'light' }, contrast: { theme: 'contrast' },
  C1: { theme: 'light', 'text-spacing': 'wider', targets: '44', motion: 'reduce', 'colour-perception': 'deutan' },
  S2: { theme: 'dark', scale: '2' },
  C2: { theme: 'dark', contrast: 'increased', 'light-intensity': 'dimmed', transparency: 'opaque', 'line-length': 'short', stimulus: 'calm' } }
const FEAS = ['now', 'feasible', 'research', 'simulated']
const sleep = (ms) => new Promise((r) => setTimeout(r, ms))
const failures = []
const warnings = []
let cells = 0

const parse = (c) => {
  let m = c.match(/rgba?\(([\d.]+),\s*([\d.]+),\s*([\d.]+)(?:,\s*([\d.]+))?\)/)
  if (m) return [+m[1], +m[2], +m[3], m[4] == null ? 1 : +m[4]]
  m = c.match(/color\(srgb ([\d.]+) ([\d.]+) ([\d.]+)(?: \/ ([\d.]+))?\)/)
  if (m) return [m[1] * 255, m[2] * 255, m[3] * 255, m[4] == null ? 1 : +m[4]]
  return null
}
const lum = ([r, g, b]) => { const f = (v) => { v /= 255; return v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4 }; return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b) }
const ratio = (a, b) => { const [x, y] = [lum(a), lum(b)].sort((p, q) => q - p); return (x + 0.05) / (y + 0.05) }

async function load(w, h) {
  frame.style.width = `${w}px`; frame.style.height = `${h}px`
  frame.src = `./index.html?test=1&t=${Date.now()}`
  await new Promise((r) => { frame.onload = r })
  for (let i = 0; i < 50 && !frame.contentWindow.__journey; i++) await sleep(50)
  if (!frame.contentWindow.__journey) throw new Error(`journey did not start: ${JSON.stringify(frame.contentWindow.__errs)}`)
  // Focus-dependent rules (2.4.11, :focus-within lifts) are only valid when the document really has focus: fail loudly otherwise.
  frame.contentWindow.focus()
  if (!frame.contentDocument.hasFocus()) throw new Error('NO_FOCUS: the journey document has no focus; run the sweep in a focused window or under CDP Emulation.setFocusEmulationEnabled')
  return frame.contentWindow
}

function check(win, cell, station, state, varNames) {
  const doc = win.document
  const add = (kind, detail, list = failures) => list.push({ cell, station, state, kind, detail: String(detail).slice(0, 160) })
  const vw = doc.documentElement.clientWidth
  const theme = doc.documentElement.dataset.theme
  const visible = (el) => { if (el.closest('[hidden],[inert]')) return false; const cs = win.getComputedStyle(el); if (cs.visibility === 'hidden' || cs.display === 'none') return false; const r = el.getBoundingClientRect(); return r.width > 0 && r.height > 0 }
  const name = (el) => `${el.tagName.toLowerCase()}${el.id ? '#' + el.id : ''}.${[...el.classList].join('.')} "${(el.getAttribute('aria-label') || el.textContent || '').trim().slice(0, 40)}"`
  // Only containers designed to scroll sideways (data tables, WCAG 1.4.10 exception) excuse horizontal overflow.
  const scrollParent = (el) => el.closest('.jr-table-wrap')
  // An element scrolled out of view inside a vertical scroll box is not obscured; one inside the visible part is tested.
  const scrolledAway = (el, cx, cy) => { for (let p = el.parentElement; p && p !== doc.body; p = p.parentElement) { const cs = win.getComputedStyle(p); if (/auto|scroll/.test(cs.overflowY + cs.overflowX)) { const q = p.getBoundingClientRect(); return cx < q.left || cx > q.right || cy < q.top || cy > q.bottom } } return false }
  const textOf = (el) => (el.textContent || '').trim()
  const paints = (hit, box) => {
    for (let p = hit; p && !p.contains(box); p = p.parentElement) {
      const cs = win.getComputedStyle(p); const bg = parse(cs.backgroundColor)
      if ((bg && bg[3] > 0.3) || cs.backgroundImage !== 'none' || parseFloat(cs.borderTopWidth) > 0 || textOf(p) && p === hit || /^(IMG|svg|INPUT|BUTTON)$/i.test(p.tagName)) return true
    }
    return false
  }
  const inFlow = (el) => !/absolute|fixed/.test(win.getComputedStyle(el).position)
  // inside a clipping box that itself fits the viewport (e.g. the indeterminate progress track): not lost content
  const clipped = (el) => { for (let p = el.parentElement; p && p !== doc.body && p.id !== 'jr-os'; p = p.parentElement) { const o = win.getComputedStyle(p).overflowX; if (o === 'hidden' || o === 'clip') { const q = p.getBoundingClientRect(); return q.left >= -1 && q.right <= vw + 1 } } return false }
  for (const e of win.__errs ?? []) add('error', e)
  // 1 overflow
  if (doc.documentElement.scrollWidth > vw + 1) add('overflow', `page scrollWidth ${doc.documentElement.scrollWidth} > ${vw}`)
  const all = [...doc.body.querySelectorAll('*')].filter((el) => !el.closest('.ds-live, .ds-vh, svg') && visible(el))
  for (const el of all) {
    const r = el.getBoundingClientRect()
    if ((r.right > vw + 1 || r.left < -1) && !scrollParent(el) && !clipped(el)) { add('overflow', `${name(el)} [${Math.round(r.left)}, ${Math.round(r.right)}]`); continue }
    const cs = win.getComputedStyle(el)
    if (/auto|scroll/.test(cs.overflowX) && el.scrollWidth > el.clientWidth + 1 && !el.matches('.jr-table-wrap')) add('hscroll', `${name(el)} ${el.scrollWidth} > ${el.clientWidth}`)
    // sibling overlap: two text-bearing in-flow children of one parent whose boxes intersect (a badge over a date, a chip under a button)
    const kids = [...el.children].filter((k) => !k.matches('.ds-vh, .ds-live') && visible(k) && textOf(k) && inFlow(k))
    for (let i = 0; i < kids.length; i++) for (let j = i + 1; j < kids.length; j++) {
      // inline boxes are compared per line box (getClientRects), so text that wraps around a chip is not an overlap
      const boxes = (k) => (win.getComputedStyle(k).display === 'inline' ? [...k.getClientRects()] : [k.getBoundingClientRect()])
      let worst = null
      for (const a of boxes(kids[i])) for (const c of boxes(kids[j])) {
        const ox = Math.min(a.right, c.right) - Math.max(a.left, c.left), oy = Math.min(a.bottom, c.bottom) - Math.max(a.top, c.top)
        if (ox > 2 && oy > 2) worst = [ox, oy]
      }
      if (worst) add('overlap', `${name(kids[i])} x ${name(kids[j])} by ${Math.round(worst[0])}x${Math.round(worst[1])}`)
    }
    if (cs.overflowX === 'hidden' && cs.textOverflow !== 'ellipsis' && el.scrollWidth > el.clientWidth + 2 && el.id !== 'jr-os' && !el.matches('.jr-desk, .jr-lock, .ds-orbit, .ds-window, .ds-input-wrap, .ds-progress, .ds-skeleton, .ds-skeleton-row')) add('clip', name(el))
    // vertical clip (WCAG 1.4.10): a hidden or clipped box whose own content runs past its bottom edge and holds text or a control
    // loses that content for pointer and touch. Exempt: the OS frame and desktop (absolute layers; their own children are checked).
    if (/hidden|clip/.test(cs.overflowY) && el.scrollHeight > el.clientHeight + 2 && el.id !== 'jr-os' && !el.matches('.jr-desk, .ds-progress, .ds-skeleton, .ds-skeleton-row, .ds-window-title') && el.querySelector('a[href], button, input, select, textarea, p, h1, h2, h3, label')) add('clip-y', `${name(el)} ${el.scrollHeight} > ${el.clientHeight}`)
  }
  // 1c lost content (WCAG 1.4.10, repair 2): a control or text-bearing element whose centre lies outside an overflow-hidden or -clip
  // ancestor, with no scroll container in between, cannot be reached by pointer, touch or scrolling. This also covers the boxes
  // exempted above (#jr-os, .jr-desk, .jr-lock, .ds-window), whose own children were otherwise never checked against their edge.
  for (const el of all) {
    if (!el.matches('a[href], button, input, select, textarea, [tabindex]') && ![...el.childNodes].some((n) => n.nodeType === 3 && n.textContent.trim())) continue
    const r = el.getBoundingClientRect(), cx = r.left + r.width / 2, cy = r.top + r.height / 2
    for (let p = el.parentElement; p && p !== doc.body; p = p.parentElement) {
      const pcs = win.getComputedStyle(p)
      if (/auto|scroll/.test(pcs.overflowY + pcs.overflowX)) break
      if (!/hidden|clip/.test(pcs.overflowY + pcs.overflowX)) continue
      const q = p.getBoundingClientRect()
      if (cx < q.left - 1 || cx > q.right + 1 || cy < q.top - 1 || cy > q.bottom + 1) { add('lost', `${name(el)} outside ${name(p)} [${Math.round(r.top)}..${Math.round(r.bottom)} vs ${Math.round(q.top)}..${Math.round(q.bottom)}]`); break }
    }
  }
  // 2 feasibility labels
  for (const el of doc.querySelectorAll('[data-sim-fn]')) {
    if (!visible(el)) continue
    const scope = el.closest('[data-sim-scope]') ?? el
    if (!el.querySelector('.jr-feas') && !scope.querySelector('.jr-feas')) add('feasibility', name(el))
  }
  for (const s of win.__journey.stations()) for (const row of s.feas) if (!FEAS.includes(row[1])) add('feasibility', `${s.id}: ${row[1]}`)
  // 3 undefined var(--x)
  const rootCs = win.getComputedStyle(doc.documentElement)
  for (const v of varNames) if (!rootCs.getPropertyValue(v).trim()) add('var', v)
  // 4 focus order and 5 target size
  const tabbables = [...doc.querySelectorAll('a[href], button, input, select, textarea, [tabindex]')].filter((el) => !el.disabled && el.getAttribute('tabindex') !== '-1' && visible(el) && !(el.type === 'radio' && !el.checked && doc.querySelector(`input[name="${el.name}"]:checked`)))
  const min = { '32': 32, '44': 44 }[doc.documentElement.dataset.targets] ?? 24
  let lastRegion = -1
  const retest = []
  const regionOf = (el) => (el.closest('.jr-band, .jr-panel') ? 0 : el.closest('main') ? 1 : el.closest('dialog') ? 2 : -1)
  for (const el of tabbables) {
    const reg = regionOf(el)
    if (reg < 0) add('focus', `outside landmarks: ${name(el)}`)
    if (reg < lastRegion && reg !== 2) add('focus', `region order breaks at ${name(el)}`)
    lastRegion = Math.max(lastRegion, reg)
    const box = el.closest('.ds-radio, .ds-check, .jr-user') ?? el
    const r = box.getBoundingClientRect()
    if ((r.width < min - 0.5 || r.height < min - 0.5) && !el.closest('.jr-table td, p')) add('target', `${Math.round(r.width)}x${Math.round(r.height)} < ${min}: ${name(el)}`)
    const cx = r.left + r.width / 2, cy = r.top + r.height / 2
    if (cx > 0 && cy > 0 && cx < vw && cy < doc.documentElement.clientHeight) {
      const hit = doc.elementFromPoint(cx, cy)
      if (hit && !box.contains(hit) && !hit.contains(box) && !scrollParent(el) && !scrolledAway(el, cx, cy)) {
        // Pointer: a control may sit behind a visible surface (a window over the desktop, an open popover), never behind a layer
        // that paints nothing. Walk from the hit element up to the first common ancestor; no painted box on the way = dead control.
        if (!paints(hit, box)) { add('pointer-blocked', `${name(el)} under transparent ${name(hit)}`); continue }
        add('behind-surface', name(el), warnings)
        // WCAG 2.4.11: re-test with focus on the element (focus may lift or reveal it)
        // an inactive window is raised by focusin (journey wm); re-testing it here would mutate the journey mid-sweep
        if (el.closest('.ds-window[data-state="inactive"]')) { add('raised-on-focus', name(el), warnings); continue }
        retest.push([el, box, cx, cy, hit])
      }
    }
  }
  // 5b held key (ds-1.1 B8): an auto-repeated Enter or Space on a control must be swallowed (defaultPrevented). Synthetic events
  // never activate natively, so this checks the guard everywhere; CDP autoRepeat runs prove the real behaviour (PROTOTYPE.md section 9).
  for (const el of tabbables) {
    const keys = el.matches('input:not([type=checkbox]):not([type=radio])') ? ['Enter'] : el.matches('button, [role=button], [role=switch], [role=menuitem], [role=tab], [role=option], [role=listbox], input') ? ['Enter', ' '] : []
    for (const key of keys) {
      const ev = new win.KeyboardEvent('keydown', { key, repeat: true, bubbles: true, cancelable: true })
      el.dispatchEvent(ev)
      if (!ev.defaultPrevented) add('held-key', `${key === ' ' ? 'Space' : key} repeat not swallowed: ${name(el)}`)
    }
  }
  // 5c focused element must be visible and not covered (WCAG 2.4.11), for the focus each state leaves behind (initial focus of popovers)
  const act = doc.activeElement
  if (act && act !== doc.body && visible(act)) {
    const r = act.getBoundingClientRect(), cx = r.left + r.width / 2, cy = r.top + Math.min(r.height / 2, 12)
    const hit = cx >= 0 && cy >= 0 && cx < vw && cy < doc.documentElement.clientHeight ? doc.elementFromPoint(cx, cy) : null
    if (!hit && !scrollParent(act)) add('focus-obscured', `focused element outside the viewport: ${name(act)}`)
    else if (hit && !act.contains(hit) && !hit.contains(act) && !(act.closest('.jr-user') ?? act.closest('.ds-radio, .ds-check'))?.contains(hit)) add('focus-obscured', `${name(act)} under ${name(hit)}`)
  }
  // 6 text contrast
  const need = (el, cs) => { const px = parseFloat(cs.fontSize); const big = px >= 24 || (px >= 18.66 && +cs.fontWeight >= 700); return theme === 'contrast' ? (big ? 4.5 : 7) : big ? 3 : 4.5 }
  for (const el of all) {
    if (![...el.childNodes].some((n) => n.nodeType === 3 && n.textContent.trim())) continue
    if (el.closest('button:disabled, input:disabled, iframe')) continue
    const cs = win.getComputedStyle(el)
    const fg = parse(cs.color); if (!fg) continue
    let bg = null
    let img = false
    for (let p = el; p; p = p.parentElement) { const pcs = win.getComputedStyle(p); if (pcs.backgroundImage !== 'none') { img = true; break } const c = parse(pcs.backgroundColor); if (c && c[3] >= 0.99) { bg = c; break } }
    if (img) continue // gradient fill (primary button): measured per component by the gallery's ?measure=1
    if (!bg) bg = parse(win.getComputedStyle(doc.body).backgroundColor) ?? [0, 0, 0, 1]
    const r = ratio(fg, bg)
    if (r < need(el, cs) - 0.05) add('contrast', `${r.toFixed(2)} < ${need(el, cs)}: ${name(el)} fg ${cs.color} bg ${bg.join(',')} own ${cs.backgroundColor} ${cs.transitionDuration}`)
  }
  // 7 animations
  for (const a of doc.getAnimations()) {
    const t = a.effect?.getComputedTiming?.() ?? {}
    if (!Number.isFinite(t.iterations) || (t.endTime ?? 0) > 5000 || (t.iterations > 1 && t.duration < 1000)) add('animation', `${a.animationName ?? a.constructor.name} end ${t.endTime} it ${t.iterations}`)
  }
  // 8 live regions
  if (doc.querySelector('[aria-live="assertive"]')) add('live', 'assertive region present')
  if (doc.querySelectorAll('#jr-live[aria-live="polite"]').length !== 1) add('live', 'station live region missing')
  for (const el of doc.querySelectorAll('[role="alert"]')) if (visible(el) && !el.closest('.ds-empty[data-tone="error"]')) add('live', `role=alert outside a user-caused error: ${name(el)}`)
  // 9 stringified null or undefined in visible text or accessible names (a missing value rendered as 'null')
  for (const el of all) {
    const own = [...el.childNodes].filter((n) => n.nodeType === 3).map((n) => n.textContent).join(' ')
    const lbl = el.getAttribute('aria-label') ?? ''
    if (/(^|[^\w])(null|undefined|NaN)([^\w]|$)|nullnull/.test(`${own} ${lbl}`)) add('null-text', `${name(el)} "${own.trim().slice(0, 60)}"`)
  }
  // focus re-test last: focusing may close a popover
  for (const [el, box, cx, cy, hit] of retest) {
    if (!el.isConnected) continue
    el.focus({ preventScroll: true })
    const again = doc.elementFromPoint(cx, cy)
    if (again && !box.contains(again) && !again.contains(box)) add('obscured', `${name(el)} under ${name(again)}`); else add('covered-until-focus', name(el), warnings)
    el.blur()
  }
}

async function run() {
  const css = (await Promise.all(['./journey.css', '../system/components.css'].map((u) => fetch(u).then((r) => r.text())))).join('\n')
  const varNames = [...new Set([...css.matchAll(/var\((--[\w-]+)/g)].map((m) => m[1]))].filter((v) => !v.startsWith('--jr-dyn-') && !v.startsWith('--ds-') && !v.startsWith('--gal-'))
  for (const [w, h] of VIEWS) {
    const win = await load(w, h)
    const J = win.__journey
    const themes = Object.keys(THEMES).filter((k) => (w !== 720 || !k.startsWith('C')) && (k !== 'S2' || w === 1440))
    for (const th of themes) {
      J.setRoot(THEMES[th])
      await sleep(450)
      for (const s of J.stations()) for (const state of s.states) {
        await J.show(s.id, state)
        await sleep(state === 'success' && s.id === 'vensters' ? 50 : 120)
        // a settled state shows no loading skeleton or busy region; the error cell must show its error body
        for (let i = 0; i < 25 && state !== 'loading' && win.document.querySelector('[aria-busy="true"], .ds-skeleton'); i++) await sleep(100)
        if (state !== 'loading' && win.document.querySelector('[aria-busy="true"], .ds-skeleton')) failures.push({ cell: `${w}x${h} ${th}`, station: s.id, state, kind: 'coverage', detail: 'state did not settle: skeleton or aria-busy still shown' })
        if (s.id === 'vensters' && state === 'error' && !win.document.querySelector('[id$="-err"][data-tone="error"], .ds-empty[data-tone="error"]')) failures.push({ cell: `${w}x${h} ${th}`, station: s.id, state, kind: 'coverage', detail: 'error body not shown' })
        // headless virtual time does not advance CSS transitions: measure the end state of every finite animation
        for (const a of win.document.getAnimations()) if (Number.isFinite(a.effect?.getComputedTiming?.().endTime)) { try { a.finish() } catch {} }
        await sleep(60) // let animationend handlers run, as they would for a person
        cells++
        check(win, `${w}x${h} ${th}`, s.id, state, varNames)
        // consent coverage: the granted Tuinplan body is measured in the success cell (OQ-5, contract.js grants)
        if (s.id === 'vensters' && state === 'success' && !win.document.querySelector('#jr-note')) failures.push({ cell: `${w}x${h} ${th}`, station: s.id, state, kind: 'coverage', detail: 'granted Tuinplan body not shown' })
      }
      for (const x of J.extras()) {
        await J.extra(x)
        for (const a of win.document.getAnimations()) if (Number.isFinite(a.effect?.getComputedTiming?.().endTime)) { try { a.finish() } catch {} }
        await sleep(60)
        cells++
        check(win, `${w}x${h} ${th}`, `extra:${x}`, 'view', varNames)
        if (x === 'nogrant' && win.document.querySelector('#jr-note')) failures.push({ cell: `${w}x${h} ${th}`, station: 'extra:nogrant', state: 'view', kind: 'consent', detail: 'Tuinplan text shown without a grant' })
        await J.reset()
      }
    }
  }
  const out = { focus: 'document.hasFocus() true in every load', cells, failures, warnings: warnings.length, warningsByKind: warnings.reduce((m, f) => ((m[f.kind] = (m[f.kind] ?? 0) + 1), m), {}), warningSample: warnings.slice(0, 20), summary: Object.entries(failures.reduce((m, f) => ((m[f.kind] = (m[f.kind] ?? 0) + 1), m), {})) }
  document.getElementById('out').textContent = JSON.stringify(out, null, 1)
  document.querySelector('#sw-table tbody').replaceChildren(...failures.slice(0, 200).map((f) => { const tr = document.createElement('tr'); for (const k of ['cell', 'station', 'state', 'kind', 'detail']) { const td = document.createElement('td'); td.textContent = f[k]; tr.append(td) } return tr }))
  document.getElementById('sw-state').textContent = `done: ${cells} cells, ${failures.length} failures`
}
run().catch((e) => { document.getElementById('out').textContent = JSON.stringify({ crash: String(e && e.stack || e) }); document.getElementById('sw-state').textContent = `crashed: ${String(e && e.message || e).slice(0, 120)}` })
