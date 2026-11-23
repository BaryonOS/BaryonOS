// ds-1.1 accessibility axes: structural and contrast check (task UX-A11Y-DS, 15 November 2026). Node >= 18, no dependencies.
// Usage: node docs/verification/a11y-ds-20261115/check.mjs [repo-root] [--self-test]
// Checks: (1) the three theme blocks define the same colour, place, material and elevation tokens; (2) every axis override defines the
// tokens section 21 promises, and each device-preference block equals its attribute block; (3) every var(--x) is defined; (4) no colour
// literal outside tokens.css (components.css and the gallery <style>); (5) every axis value in the section 21 table has a selector;
// (6) every listed contrast pair holds in every theme x axis combination, overlay stacks included: each foreground on each surface
// alone and under hover, hover-strong, selection and the secondary-button fill (place colours also on the pill over those), WCAG
// relative luminance from the token text, translucent colours composited over their base and quantised to 8 bits like rendered
// pixels, light intensity as sRGB channels x 0.85 (the CSS brightness() filter) then quantised; (7) colour-perception palettes keep
// the three places apart under Machado-2009 simulation (CIE76 dE >= 40); (8) every var() used by components.css resolves in each
// theme's base cascade with no axis set; (9) the contrast theme has no translucent colour under any axis (transparent = no paint
// is allowed). --self-test applies 26 mutations (21 from the repair-1 verifier, verify-0/mutate.py) and expects each to fail.
// Exit 0 only if all pass.
import { readFileSync } from 'node:fs'
import { join } from 'node:path'
import { pathToFileURL } from 'node:url'

const ROOT = process.argv.slice(2).find(a => !a.startsWith('--')) || '.'
const read = f => readFileSync(join(ROOT, f), 'utf8')

// ---------- CSS parsing ----------
const strip = s => s.replace(/\/\*[\s\S]*?\*\//g, '')
function parse(css, media = null, out = []) {
  let i = 0
  while (i < css.length) {
    const o = css.indexOf('{', i); if (o < 0) break
    const head = css.slice(i, o).trim()
    let d = 1, j = o + 1
    while (d && j < css.length) { if (css[j] === '{') d++; else if (css[j] === '}') d--; j++ }
    const body = css.slice(o + 1, j - 1)
    if (head.startsWith('@media')) parse(body, head.slice(6).trim(), out)
    else if (!head.startsWith('@')) {
      const decl = {}
      for (const m of body.matchAll(/(--[\w-]+)\s*:\s*([^;]+);?/g)) decl[m[1]] = m[2].trim()
      out.push({ sel: head, media, decl, order: out.length })
    }
    i = j
  }
  return out
}
function decls(css, out = []) { // [property, value] of every declaration, at any @-rule depth
  let i = 0
  while (i < css.length) {
    const o = css.indexOf('{', i); if (o < 0) break
    let d = 1, j = o + 1
    while (d && j < css.length) { if (css[j] === '{') d++; else if (css[j] === '}') d--; j++ }
    const body = css.slice(o + 1, j - 1)
    if (body.includes('{')) decls(body, out)
    else for (const p of body.split(';')) { const c = p.indexOf(':'); if (c > 0) out.push([p.slice(0, c).trim(), p.slice(c + 1)]) }
    i = j
  }
  return out
}
const NAMED = new Set(('aliceblue antiquewhite aqua aquamarine azure beige bisque black blanchedalmond blue blueviolet brown burlywood cadetblue chartreuse chocolate coral cornflowerblue cornsilk crimson cyan darkblue darkcyan darkgoldenrod darkgray darkgreen darkgrey darkkhaki darkmagenta darkolivegreen darkorange darkorchid darkred darksalmon darkseagreen darkslateblue darkslategray darkslategrey darkturquoise darkviolet deeppink deepskyblue dimgray dimgrey dodgerblue firebrick floralwhite forestgreen fuchsia gainsboro ghostwhite gold goldenrod gray green greenyellow grey honeydew hotpink indianred indigo ivory khaki lavender lavenderblush lawngreen lemonchiffon lightblue lightcoral lightcyan lightgoldenrodyellow lightgray lightgreen lightgrey lightpink lightsalmon lightseagreen lightskyblue lightslategray lightslategrey lightsteelblue lightyellow lime limegreen linen magenta maroon mediumaquamarine mediumblue mediumorchid mediumpurple mediumseagreen mediumslateblue mediumspringgreen mediumturquoise mediumvioletred midnightblue mintcream mistyrose moccasin navajowhite navy oldlace olive olivedrab orange orangered orchid palegoldenrod palegreen paleturquoise palevioletred papayawhip peachpuff peru pink plum powderblue purple rebeccapurple red rosybrown royalblue saddlebrown salmon sandybrown seagreen seashell sienna silver skyblue slateblue slategray slategrey snow springgreen steelblue tan teal thistle tomato turquoise violet wheat white whitesmoke yellow yellowgreen ' +
  'accentcolor accentcolortext activetext buttonborder buttonface buttontext canvas canvastext field fieldtext graytext highlight highlighttext linktext mark marktext selecteditem selecteditemtext visitedtext').split(/\s+/))
const splitTop = s => { const r = []; let d = 0, b = 0; for (let i = 0; i < s.length; i++) { const c = s[i]; if (c === '(' || c === '[') d++; else if (c === ')' || c === ']') d--; else if (c === ',' && !d) { r.push(s.slice(b, i).trim()); b = i + 1 } } r.push(s.slice(b).trim()); return r }
// One flattened element: it carries the theme and every axis attribute (ancestors and self are the same element).
// Returns specificity [a,b] of the best matching alternative, or null. Class and type selectors never match token blocks.
function match(sel, el) {
  let best = null
  for (const alt of splitTop(sel)) {
    let spec = 0, ok = true
    for (const part of alt.split(/\s+(?![^()]*\))/).filter(Boolean)) {
      let rest = part
      while (rest && ok) {
        let m
        if (rest[0] === '*') rest = rest.slice(1)
        else if ((m = rest.match(/^:root/))) { spec++; rest = rest.slice(5); ok = el.root }
        else if ((m = rest.match(/^\[([\w-]+)(?:="([^"]*)")?\]/))) { spec++; rest = rest.slice(m[0].length); ok = m[2] === undefined ? m[1] in el : el[m[1]] === m[2] }
        else if ((m = rest.match(/^:(is|not)\(/))) {
          let d = 1, j = m[0].length; while (d) { if (rest[j] === '(') d++; else if (rest[j] === ')') d--; j++ }
          const r = match(rest.slice(m[0].length, j - 1), el); rest = rest.slice(j)
          if (m[1] === 'is') { ok = r !== null; spec += r || 0 } else { ok = r === null; spec += 1 } // :not() of attribute lists counts one level
        } else { ok = false }
      }
      if (!ok) break
    }
    if (ok && (best === null || spec > best)) best = spec
  }
  return best
}
const mediaOn = (m, env) => !m || env.media.has(m.replace(/\s+/g, ' '))
function tokens(rules, el, env) {
  const hits = rules.filter(r => mediaOn(r.media, env)).map(r => ({ r, s: match(r.sel, el) })).filter(h => h.s !== null)
  hits.sort((a, b) => a.s - b.s || a.r.order - b.r.order)
  const t = {}; for (const { r } of hits) Object.assign(t, r.decl)
  const res = (v, n = 0) => n > 9 ? v : v.replace(/var\((--[\w-]+)(?:,[^)]*)?\)/g, (_, k) => res(t[k] ?? `MISSING(${k})`, n + 1))
  return Object.fromEntries(Object.entries(t).map(([k, v]) => [k, res(v)]))
}

// ---------- colour ----------
function colours(v) { // every colour in a value (gradient stops included); [r,g,b,a]
  const out = []
  for (const m of v.matchAll(/#([0-9a-f]{3,8})\b|rgba?\(([^)]*)\)|\b(transparent)\b/gi)) {
    if (m[3]) out.push([0, 0, 0, 0])
    else if (m[1]) { let h = m[1]; if (h.length <= 4) h = [...h].map(c => c + c).join(''); out.push([0, 2, 4].map(i => parseInt(h.slice(i, i + 2), 16)).concat(h.length === 8 ? parseInt(h.slice(6), 16) / 255 : 1)) }
    else { const p = m[2].split(/[\s,/]+/).filter(Boolean).map(Number); out.push([p[0], p[1], p[2], p[3] ?? 1]) }
  }
  return out
}
const over = (c, base) => c[3] >= 1 ? c : [0, 1, 2].map(i => c[i] * c[3] + base[i] * (1 - c[3])).concat(1)
const lum = c => c.slice(0, 3).map(v => { v /= 255; return v <= .04045 ? v / 12.92 : ((v + .055) / 1.055) ** 2.4 }).reduce((a, v, i) => a + v * [.2126, .7152, .0722][i], 0)
const ratio = (a, b) => { const [x, y] = [lum(a), lum(b)].sort((m, n) => n - m); return (x + .05) / (y + .05) }
const dim = (c, k) => [c[0] * k, c[1] * k, c[2] * k, 1]
const M = { protan: [[.152286, 1.052583, -.204868], [.114503, .786281, .099216], [-.003882, -.048116, 1.051998]], deutan: [[.367322, .860646, -.227968], [.280085, .672501, .047413], [-.011820, .042940, .968881]], tritan: [[1.255528, -.076749, -.178779], [-.078411, .930809, .147602], [.004733, .691367, .303900]] }
const linc = c => c.slice(0, 3).map(v => { v /= 255; return v <= .04045 ? v / 12.92 : ((v + .055) / 1.055) ** 2.4 })
const lab = c => { const X = (.4124 * c[0] + .3576 * c[1] + .1805 * c[2]) / .95047, Y = .2126 * c[0] + .7152 * c[1] + .0722 * c[2], Z = (.0193 * c[0] + .1192 * c[1] + .9505 * c[2]) / 1.08883; const f = v => v > .008856 ? Math.cbrt(v) : 7.787 * v + 16 / 116; return [116 * f(Y) - 16, 500 * (f(X) - f(Y)), 200 * (f(Y) - f(Z))] }
const simLab = (c, t) => lab(M[t].map(r => Math.min(1, Math.max(0, r[0] * c[0] + r[1] * c[1] + r[2] * c[2]))))
const dE = (a, b, t) => { const p = simLab(linc(a), t), q = simLab(linc(b), t); return Math.hypot(p[0] - q[0], p[1] - q[1], p[2] - q[2]) }

// ---------- the axis model (DESIGN-SYSTEM.md section 21) ----------
const THEMES = ['dark', 'light', 'contrast']
const AX = { 'data-surface': ['default', 'neutral'], 'data-contrast': ['standard', 'increased'], 'data-transparency': ['translucent', 'opaque'], 'data-colour-perception': ['off', 'protan', 'deutan', 'tritan'], 'data-stimulus': ['regular', 'calm', 'rich'], 'data-light-intensity': ['standard', 'dimmed'] }
const K = { standard: 1, dimmed: .85 } // components.css brightness(.85)
const PROMISE = { // tokens an axis value must define per theme (contrast theme exempt where section 21.3 says it already meets the value)
  'data-contrast=increased': { themes: ['dark', 'light'], names: ['--color-text', '--color-text-muted', '--color-text-subtle', '--color-text-disabled', '--color-line', '--color-line-strong', '--color-rim-c', '--material-glass-line', '--material-kern-tx2', '--material-kern-line', '--color-neutral-text', '--color-neutral-text-muted', '--color-neutral-text-subtle', '--color-neutral-line', '--color-neutral-line-strong'] },
  'data-transparency=opaque': { themes: ['dark', 'light'], names: ['--material-glass', '--material-orbit-bg', '--material-launch'] },
  'data-colour-perception=protan': { themes: THEMES, names: ['--place-device', '--place-nas', '--place-internet', '--place-internet-line', '--material-kern-warn-ic', '--material-kern-warn-line'] },
  'data-colour-perception=deutan': { themes: THEMES, names: ['--place-device', '--place-nas', '--place-internet', '--place-internet-line', '--material-kern-warn-ic', '--material-kern-warn-line'] },
  'data-colour-perception=tritan': { themes: THEMES, names: ['--place-device', '--place-nas', '--place-internet', '--place-internet-line', '--material-kern-warn-ic', '--material-kern-warn-line'] },
  'data-stimulus=calm': { themes: THEMES, names: ['--color-blade-glow'] },
  'data-stimulus=rich': { themes: THEMES, names: ['--color-blade-glow'] },
  'data-stimulus=calm ': { themes: ['dark', 'light'], names: ['--elev-window', '--elev-window-inactive', '--elev-orbit', '--elev-plate', '--elev-launcher', '--elev-inspector'] }, // trailing space: a second key for the same value
  'data-stimulus=rich ': { themes: ['dark', 'light'], names: ['--elev-window', '--elev-window-inactive', '--elev-orbit', '--elev-plate', '--elev-launcher', '--elev-inspector'] },
}
const DEVICE = [['data-contrast', 'increased', '(prefers-contrast: more)'], ['data-transparency', 'opaque', '(prefers-reduced-transparency: reduce)']]
const SURF = ['--color-bg', '--color-surface-panel', '--color-surface', '--color-surface-raised', '--color-surface-inset']
const OV = [null, '--color-hover', '--color-hover-strong', '--color-selection', '--color-button-secondary-bg'] // row and button states
const NSURF = ['--color-neutral-surface-panel', '--color-neutral-surface', '--color-neutral-surface-raised', '--color-neutral-surface-inset']
const stacks = (bases, ovs = OV, top = []) => bases.flatMap(b => ovs.map(o => [b, ...(o ? [o] : []), ...top]))
const ALLST = [...stacks(SURF), ...stacks(SURF, OV, ['--color-pill-bg'])]
// [foreground, background stacks (bottom first, composited over --color-bg), threshold, threshold under increased contrast or the
// contrast theme, themes (default all)]. Section 4 of DESIGN-SYSTEM.md lists the same pairs.
const PAIRS = [
  ...['--color-text', '--color-text-muted', '--color-text-subtle'].map(f => [f, stacks(SURF), 4.5, 7]),
  ['--color-text-disabled', stacks(SURF, [null]), 4.5, 4.5],
  ['--color-line-strong', stacks(SURF), 3, 3], ['--color-focus', stacks(SURF), 3, 3], ['--color-blade', stacks(SURF, [null]), 3, 3],
  ...['--place-device', '--place-nas', '--place-internet'].map(f => [f, ALLST, 3, 3]),
  ['--color-button-primary-text', [['--color-button-primary-bg'], ['--color-button-primary-bg-hover']], 4.5, 7],
  ['--material-kern-tx', [['--material-kern-bg']], 4.5, 7], ['--material-kern-tx2', [['--material-kern-bg']], 4.5, 7],
  ['--material-kern-warn-ic', [['--material-kern-warn-bg']], 3, 3],
  ['--material-kern-warn-line', [['--material-kern-bg']], 3, 3, ['contrast']],
  ['--material-tile-fg', [['--material-tile-bg']], 3, 3],
  ...['--color-text', '--color-text-muted', '--color-text-subtle'].map(f => [f, [['--material-insp-bg']], 4.5, 7]),
  ['--color-text', [['--material-glass'], ['--material-orbit-bg']], 4.5, 7],
  ...['--color-neutral-text', '--color-neutral-text-muted', '--color-neutral-text-subtle'].map(f => [f, NSURF.map(n => [n]), 4.5, 7]),
  ['--color-neutral-line-strong', NSURF.map(n => [n]), 3, 3],
  ['--color-text-muted', [['--color-surface-inset']], 3, 3], // determinate progress fill on its track (ds-1.1)
]
const LINES = ['--color-line', stacks(SURF, [null]), 3] // only promised under increased contrast
const q8 = c => [Math.round(c[0]), Math.round(c[1]), Math.round(c[2]), c[3]]

// ---------- run ----------
function run(src) {
  const fails = [], info = []
  const fail = (m) => fails.push(m)
  const tc = strip(src.tokens), cc = strip(src.components)
  const rules = parse(tc)
  // (1) theme blocks
  const sets = Object.fromEntries([[':root, [data-theme="dark"]', 'dark'], ['[data-theme="light"]', 'light'], ['[data-theme="contrast"]', 'contrast']].map(([s, n]) => {
    const r = rules.find(x => x.sel === s && !x.media); return [n, new Set(Object.keys(r?.decl || {}).filter(k => /^--(color|place|material|elev)-/.test(k)))]
  }))
  const all = new Set(Object.values(sets).flatMap(s => [...s]))
  for (const [n, s] of Object.entries(sets)) for (const k of all) if (!s.has(k)) fail(`theme ${n}: ${k} missing`)
  info.push(`themes: ${THEMES.map(t => `${t} ${sets[t].size}`).join(', ')} themed names`)
  const el = (theme, ax = {}) => ({ root: true, 'data-theme': theme, ...ax })
  const env0 = { media: new Set() }
  // (2) promised axis definitions: the axis rule must itself declare each name for each theme
  for (const [key, p] of Object.entries(PROMISE)) {
    const [a, v] = key.trim().split('=')
    for (const th of p.themes) {
      const declared = new Set(rules.filter(r => !r.media && match(r.sel, el(th, { [a]: v })) !== null && r.sel.includes(`${a}="${v}"`)).flatMap(r => Object.keys(r.decl)))
      for (const n of p.names) if (!declared.has(n)) fail(`axis ${key} in ${th}: ${n} not defined`)
    }
  }
  for (const [a, v, m] of DEVICE) for (const th of ['dark', 'light']) {
    const byAttr = tokens(rules, el(th, { [a]: v }), env0), byDev = tokens(rules, el(th), { media: new Set([m]) })
    for (const k of Object.keys(byAttr)) if (byAttr[k] !== byDev[k]) fail(`device ${m} in ${th}: ${k} = ${byDev[k]} but ${a}="${v}" gives ${byAttr[k]}`)
    const explicit = tokens(rules, el(th, { [a]: AX[a][0] }), { media: new Set([m]) }), plain = tokens(rules, el(th), env0)
    for (const k of Object.keys(plain)) if (plain[k] !== explicit[k]) { fail(`precedence: explicit ${a}="${AX[a][0]}" under ${m} in ${th} changes ${k}`); break }
  }
  const full = rules.find(r => r.sel === '[data-motion="full"]'), rootM = rules.find(r => r.sel === ':root' && r.decl['--motion-duration-fast'])
  for (const k of Object.keys(rootM?.decl || {}).filter(k => k.startsWith('--motion-duration-'))) if (full?.decl[k] !== rootM.decl[k]) fail(`data-motion="full": ${k} differs from :root`)
  const red = rules.find(r => r.sel === '[data-motion="reduce"]')
  for (const k of Object.keys(rootM?.decl || {}).filter(k => k.startsWith('--motion-duration-'))) if (red?.decl[k] !== '0ms') fail(`data-motion="reduce": ${k} is not 0ms`)
  // (3) var() defined
  const defined = new Set([...(tc + cc + src.html + src.js).matchAll(/(--[\w-]+)\s*:/g)].map(m => m[1]))
  for (const m of new Set([...(tc + cc).matchAll(/var\((--[\w-]+)/g)].map(m => m[1]))) if (!defined.has(m)) fail(`var(${m}) used but never defined`)
  // (8) every var() components.css uses resolves in each theme's base cascade (no axis), except component-local --ds-* names
  const local = new Set([...decls(cc).map(([p]) => p).filter(p => p.startsWith('--')), ...[...src.html.matchAll(/(--ds-[\w-]+)\s*:/g)].map(m => m[1])])
  for (const th of THEMES) { const t = tokens(rules, el(th), env0); for (const m of new Set([...cc.matchAll(/var\((--[\w-]+)/g)].map(m => m[1]))) if (!local.has(m) && !(m in t)) fail(`(8) var(${m}) is not defined in the ${th} base cascade`) }
  // (4) colour literals outside tokens.css: every declaration value (custom properties included) in components.css and the gallery
  // <style>; var() names, url() and strings are removed first; hex, colour functions, CSS named and system colours fail (any case)
  const style = strip((src.html.match(/<style>([\s\S]*?)<\/style>/) || [, ''])[1])
  for (const [n, t] of [['components.css', cc], ['index.html <style>', style]]) for (const [prop, val] of decls(t)) {
    const v = val.replace(/--[\w-]+/g, ' ').replace(/url\([^)]*\)|"[^"]*"|'[^']*'/g, ' ')
    const hit = v.match(/#[0-9a-f]{3,8}\b/i) || v.match(/\b(?:rgba?|hsla?|hwb|lab|lch|oklab|oklch|color|light-dark|device-cmyk)\(/i) || (v.match(/[a-z][a-z-]*/gi) || []).find(w => NAMED.has(w.toLowerCase()))
    if (hit) fail(`colour literal in ${n}: ${prop}: ${val.trim().slice(0, 60)} (${typeof hit === 'string' ? hit : hit[0]})`)
  }
  // (5) axis values promised by the document
  const tab = (src.doc.split('<!-- axes:start -->')[1] || '').split('<!-- axes:end -->')[0]
  const promised = [...new Set([...tab.matchAll(/`(data-[\w-]+)="([\w-]+)"`/g)].map(m => `[${m[1]}="${m[2]}"]`))]
  if (!promised.length) fail('section 21 axis table (<!-- axes:start --> ... <!-- axes:end -->) not found or empty')
  for (const p of promised) if (!(tc + cc).includes(p)) fail(`document promises ${p} but no CSS selector uses it`)
  info.push(`document axis values with selectors: ${promised.length}`)
  // (6) contrast pairs over every combination (+ device scenarios)
  let combos = 0, pairs = 0; const worst = {}
  const scen = []
  const keys = Object.keys(AX)
  const walk = (i, acc) => { if (i === keys.length) return scen.push(acc); for (const v of AX[keys[i]]) walk(i + 1, { ...acc, [keys[i]]: v }) }
  walk(0, {})
  const cases = []
  for (const th of THEMES) for (const s of scen) cases.push({ th, ax: s, env: env0 })
  for (const th of THEMES) for (const [, , m] of DEVICE) for (const li of AX['data-light-intensity']) cases.push({ th, ax: { 'data-light-intensity': li }, env: { media: new Set([m]) }, dev: m })
  for (const { th, ax, env, dev } of cases) {
    combos++
    const attrs = Object.fromEntries(Object.entries(ax).filter(([a, v]) => v !== AX[a][0]))
    const t = tokens(rules, el(th, attrs), env), k = K[ax['data-light-intensity'] || 'standard']
    const hi = th === 'contrast' || ax['data-contrast'] === 'increased' || dev === '(prefers-contrast: more)'
    const list = hi && th !== 'contrast' ? [...PAIRS, [...LINES, 3]] : PAIRS
    for (const [f, sts, th1, th2, only] of list) {
      if (only && !only.includes(th)) continue
      for (const st of sts) {
        if (!st.every(n => colours(t[n] || '').length) || !colours(t[f] || '').length) { fail(`unresolved pair ${f} on ${st.join('+')} in ${th} ${JSON.stringify(attrs)}`); continue }
        const tops = st.slice(1).map(n => colours(t[n])[0])
        for (const bc of colours(t[st[0]])) for (const fc of colours(t[f])) {
          if (fc[3] === 0) continue
          let B = q8(over(bc, colours(t['--color-bg'])[0])); for (const o of tops) B = q8(over(o, B))
          const F = q8(over(fc, B)), r = ratio(q8(dim(F, k)), q8(dim(B, k))), need = hi ? th2 : th1
          pairs++
          const id = `${th} ${f} on ${st.join('+')}`; if (!worst[id] || r < worst[id].r) worst[id] = { r, need, where: JSON.stringify(attrs) + (dev || '') }
          if (r + 1e-9 < need) fail(`contrast ${r.toFixed(3)} < ${need}: ${f} on ${st.join('+')} in ${th} ${JSON.stringify(attrs)}${dev ? ' ' + dev : ''}`)
        }
      }
    }
    if (th === 'contrast') for (const [n, v] of Object.entries(t)) if (/^--(color|place|material)-/.test(n) && colours(v).some(c => c[3] > 0 && c[3] < 1)) fail(`(9) contrast theme ${JSON.stringify(attrs)}: ${n} is translucent (${v})`)
  }
  info.push(`contrast: ${combos} theme x axis cases (incl. ${THEMES.length * DEVICE.length * 2} device-preference cases, each at both light intensities), ${pairs} pair evaluations`)
  const low = Object.entries(worst).sort((a, b) => (a[1].r - a[1].need) - (b[1].r - b[1].need)).slice(0, 6)
  info.push('tightest margins: ' + low.map(([id, w]) => `${id} ${w.r.toFixed(2)}/${w.need} ${w.where}`).join('; '))
  // (7) colour-perception separation
  for (const th of THEMES) for (const ty of ['protan', 'deutan', 'tritan']) {
    const t = tokens(rules, el(th, { 'data-colour-perception': ty }), env0), d = tokens(rules, el(th), env0)
    const P = n => colours(t[n])[0], D = n => colours(d[n])[0], N = ['--place-device', '--place-nas', '--place-internet']
    const mn = (G) => Math.min(dE(G(N[0]), G(N[1]), ty), dE(G(N[0]), G(N[2]), ty), dE(G(N[1]), G(N[2]), ty))
    const a = mn(P), b = mn(D)
    info.push(`${th} ${ty}: min simulated dE ${a.toFixed(1)} (default palette ${b.toFixed(1)})`)
    if (a < 40) fail(`colour-perception ${ty} in ${th}: simulated places only dE ${a.toFixed(1)} apart (< 40)`)
  }
  return { fails, info }
}

export { parse, strip, tokens, colours, over, q8, dim, ratio, dE, run, THEMES, AX }
const MAIN = import.meta.url === pathToFileURL(process.argv[1]).href
const src = !MAIN ? null : { tokens: read('ui/design/system/tokens.css'), components: read('ui/design/system/components.css'), html: read('ui/design/system/index.html'), js: read('ui/design/system/gallery.js'), doc: read('docs/design/DESIGN-SYSTEM.md') }
if (MAIN && process.argv.includes('--self-test')) { // each mutation must be caught
  // set(selector, token, value): replace the declaration inside the first top-level block with exactly this selector (asserted)
  const set = (sel, name, val) => { const css = src.tokens, i = css.indexOf(sel + ' {'); if (i < 0) throw new Error('no block ' + sel); const j = css.indexOf('\n}', i), blk = css.slice(i, j), re = new RegExp(`(\\s)${name}:[^;]*;`); if (!re.test(blk)) throw new Error(`no ${name} in ${sel}`); return { ...src, tokens: css.slice(0, i) + blk.replace(re, val === null ? '$1' : `$1${name}: ${val};`) + css.slice(j) } }
  const app = (t) => ({ ...src, components: src.components + '\n' + t + '\n' })
  const D = ':root, [data-theme="dark"]', L = '[data-theme="light"]', C = '[data-theme="contrast"]'
  const muts = [ // T, L and P: the repair-1 verifier's 21 mutations (verify-0/mutate.py), re-targeted by selector and token name
    ['T1 drop --material-glass from light', set(L, '--material-glass', null)],
    ['T2 drop --color-focus from contrast', set(C, '--color-focus', null)],
    ['T3 drop --color-hover-strong from dark', set(D, '--color-hover-strong', null)],
    ['T4 drop --type-body-line from :root', set(':root', '--type-body-line', null)],
    ['L1 hex literal', app('.ds-x { color: #fff; }')], ['L2 named colour in a border', app('.ds-x { border: 1px solid red; }')],
    ['L3 navy', app('.ds-x { color: navy; }')], ['L4 color-mix with white', app('.ds-x { background: color-mix(in srgb, var(--color-text) 50%, white); }')],
    ['L5 crimson in a box-shadow', app('.ds-x { box-shadow: 0 0 0 1px crimson; }')], ['L6 upper-case RED', app('.ds-x { color: RED; }')],
    ['L7 tomato in a gradient', app('.ds-x { background: linear-gradient(tomato, var(--color-bg)); }')], ['L8 hsl() literal', app('.ds-x { color: hsl(0 0% 50%); }')],
    ['P1 light text-muted lowered', set(L, '--color-text-muted', '#8A93A8')], ['P2 dark line-strong lowered', set(D, '--color-line-strong', '#3A4262')],
    ['P3 dark notice icon on notice', set(D, '--material-kern-warn-ic', '#4A2A30')], ['P4 dark hover-strong .32', set(D, '--color-hover-strong', 'rgba(220,230,245,.32)')],
    ['P5 dark primary hover fill darkened', set(D, '--color-button-primary-bg-hover', 'linear-gradient(180deg,#3A3F4A,#2A2E38)')],
    ['P6 dark tile glyph lowered', set(D, '--material-tile-fg', '#3A4460')], ['P7 dark inspector made light', set(D, '--material-insp-bg', 'linear-gradient(180deg,#C8CDD8,#12162B 70%)')],
    ['P8 light deutan place-device lowered', set('[data-theme="light"]:is([data-colour-perception="deutan"], [data-colour-perception="deutan"] *)', '--place-device', '#C9B040')],
    ['P9 dark neutral text-subtle lowered', set(D, '--color-neutral-text-subtle', '#4A4A4A')],
    // repair-1 additions: dimmed compensation, increased contrast at x 0.85, contrast-theme opacity, undefined var, promised selector
    ['R1 light dimmed line-strong back to the undimmed value', set('[data-theme="light"]:is([data-light-intensity="dimmed"], [data-light-intensity="dimmed"] *)', '--color-line-strong', '#6B7590')],
    ['R2 dark increased text-subtle back to #D3D8E4', { ...src, tokens: src.tokens.replaceAll('--color-text-subtle: #DADFE9;', '--color-text-subtle: #D3D8E4;') }],
    ['R3 translucent warn line in the contrast theme', set('[data-theme="contrast"]:is([data-colour-perception="tritan"], [data-colour-perception="tritan"] *)', '--material-kern-warn-line', 'rgba(255,138,128,.45)')],
    ['R4 undefined var', app('.ds-x { color: var(--color-nope); }')],
    ['R5 missing promised selector', { ...src, doc: src.doc.replace('<!-- axes:end -->', '`data-contrast="extreme"` <!-- axes:end -->') }],
  ]
  let ok = true
  for (const [n, s] of muts) { const r = run(s); console.log(`self-test ${n}: ${r.fails.length ? 'caught' : 'MISSED'} (${r.fails[0] || ''})`); ok &&= r.fails.length > 0 }
  process.exit(ok ? 0 : 1)
}
if (MAIN) {
const { fails, info } = run(src)
for (const l of info) console.log('info  ' + l)
for (const l of fails.slice(0, +(process.env.SHOW || 60))) console.log('FAIL  ' + l)
console.log(`summary: ${fails.length} failure(s)${fails.length > 60 ? ' (first 60 shown)' : ''}`)
process.exit(fails.length ? 1 : 0)
}
