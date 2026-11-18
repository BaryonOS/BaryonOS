// BaryonOS design system ds-1.1: component gallery behaviour. DESIGN ARTEFACT, 14 November 2026 (ds-1.1 15 November 2026); not a working OS.
// Builds one specimen set per theme from <template id="tpl-panel"> ({p} = id prefix, {k} = Kern theme, the opposite
// theme per MF1), renders icons with contract.js icon(), lists token swatches, and wires keyboard behaviour:
// Tab/Shift+Tab (native), Space/Enter (native buttons), arrows for tabs, listbox rows and menu (radios are native),
// Escape closes the menu, dialogs and tooltips. Query: ?only=dark|light|contrast|a11y|combos|scale|ref ?motion=reduce
// ?theme=dark|light|contrast ?measure=1. ds-1.1: the Toegankelijkheid settings group drives #a11y-preview; #combo-host renders a pairwise set.
import { icon, T, ask, setPrompt, grants, files, bus, destination } from '../prototype/contract.js'
import { ICONS } from './icons.js'

const q = new URLSearchParams(location.search)
const root = document.documentElement
const $ = (s, el = document) => el.querySelector(s)
const $$ = (s, el = document) => [...el.querySelectorAll(s)]
const THEMES = [['dark', 'Donker', 'light'], ['light', 'Licht', 'dark'], ['contrast', 'Hoog contrast', 'contrast']]
const KERN = Object.fromEntries(THEMES.map(([p, , k]) => [p, k]))
if (q.get('motion') === 'reduce') root.dataset.motion = 'reduce'
// polarity follows the device (prefers-color-scheme) unless the URL names a theme (ds-1.1 section 21.4)
const deviceTheme = () => matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark'
root.dataset.theme = ['dark', 'light', 'contrast'].includes(q.get('theme')) ? q.get('theme') : deviceTheme()

const fill = (id, p, k) => document.getElementById(id).innerHTML.replaceAll('{p}', p).replaceAll('{k}', k)
function hydrate(scope) {
  for (const ph of $$('i[data-icon]', scope)) {
    const svg = icon(ph.dataset.icon)
    if (ph.className) svg.classList.add(...ph.className.split(/\s+/).filter(Boolean))
    ph.replaceWith(svg)
  }
}

// ---------- build ----------
const host = $('#panels')
for (const [p, name, k] of THEMES) {
  const sec = document.createElement('section')
  Object.assign(sec, { className: 'gal-panel', id: `panel-${p}` })
  sec.dataset.theme = p
  sec.setAttribute('aria-labelledby', `h-${p}`)
  sec.innerHTML = `<h2 id="h-${p}">${name} <small>data-theme="${p}" · Kern in "${k}"</small></h2>` + fill('tpl-panel', p, k)
  host.append(sec)
}
$('#scale-host').innerHTML = fill('tpl-scale', 's', 'light')
const AB = { ongoing: 'ao', action: 'aa' }
for (const h of $$('[data-ask]')) {
  const p = h.closest('#scale-host') ? 's' : h.closest('.gal-panel').dataset.theme
  h.innerHTML = fill(`tpl-ask-${h.dataset.ask}`, p, p === 's' ? 'light' : KERN[p])
}
// ---------- ds-1.1 accessibility: preview, settings group and pairwise combinations (DESIGN-SYSTEM.md section 21) ----------
const prev = $('#a11y-preview')
prev.innerHTML = fill('tpl-preview', 'v', '')
for (const c of prev.children) c.inert = true // illustrative: no Tab stops after the settings (repair-1); the group keeps its label
const AXES = [['data-contrast', ['', 'increased']], ['data-transparency', ['', 'opaque']], ['data-light-intensity', ['', 'dimmed']], ['data-colour-perception', ['', 'protan', 'deutan', 'tritan']], ['data-stimulus', ['', 'calm', 'rich']], ['data-text-spacing', ['', 'wider']], ['data-line-length', ['', 'short']], ['data-targets', ['', '32', '44']], ['data-motion', ['', 'reduce']]]
function pairwise() { // greedy covering array: every value pair of every two axes appears in at least one row; row 0 is the default
  const key = (i, x, j, y) => i < j ? `${i}:${x}|${j}:${y}` : `${j}:${y}|${i}:${x}`
  const need = new Set(); AXES.forEach(([, a], i) => AXES.forEach(([, b], j) => { if (j > i) a.forEach((_, x) => b.forEach((_, y) => need.add(key(i, x, j, y)))) }))
  const total = need.size, rows = []
  const cover = r => { rows.push(r); r.forEach((x, i) => r.forEach((y, j) => { if (j > i) need.delete(key(i, x, j, y)) })) }
  cover(AXES.map(() => 0))
  while (need.size) {
    const [[i, x], [j, y]] = [...need][0].split('|').map(t => t.split(':').map(Number)), r = AXES.map(() => null); r[i] = x; r[j] = y
    for (let k = 0; k < AXES.length; k++) if (r[k] === null) {
      let best = 0, bg = -1
      AXES[k][1].forEach((_, v) => { const g = r.filter((w, m) => w !== null && need.has(key(m, w, k, v))).length; if (g > bg) { bg = g; best = v } })
      r[k] = best
    }
    cover(r)
  }
  return { rows, total, uncovered: need.size }
}
const PW = pairwise()
$('#combo-host').innerHTML = PW.rows.map((r, n) => {
  const attrs = r.map((v, k) => AXES[k][1][v] ? `${AXES[k][0]}="${AXES[k][1][v]}"` : '').filter(Boolean)
  return `<div class="gal-combo-row"><p class="gal-tag">${n ? `${n}. ` + attrs.join(' · ') : '0. standaard (geen attribuut)'}</p>` +
    THEMES.map(([p, name, k]) => `<div data-theme="${p}" ${attrs.join(' ')} inert aria-label="${name}">${fill('tpl-combo', p, k)}</div>`).join('') +
    THEMES.map(([p, name, k]) => `<div class="gal-combo-read" data-theme="${p}" ${attrs.join(' ')} inert aria-label="${name}, leestekst">${fill('tpl-combo-read', p, k)}</div>`).join('') + '</div>'
}).join('')
const setKern = () => { $('[data-kern]', prev).dataset.theme = KERN[prev.dataset.theme] }
const form = $('#a11y-form'), panel = $('#panel-a11y'), chips = $('#a11y-chips')
function applyAxis(axis, v) {
  if (axis === 'theme') prev.dataset.theme = v || deviceTheme()
  else if (v) prev.setAttribute(`data-${axis}`, v); else prev.removeAttribute(`data-${axis}`)
  if (axis === 'theme') { // the settings surface follows the chosen theme too (repair-2); each tile sample shows its own value in that theme
    setKern(); panel.dataset.theme = prev.dataset.theme
    for (const s of $$('.gal-sample:not([data-sample="theme"])', form)) s.dataset.theme = prev.dataset.theme
    $('.gal-sample[data-follow]', form).dataset.theme = deviceTheme()
  }
  const inert = prev.dataset.theme === 'contrast' // these axes change nothing (contrast: only 2 px edges) in the contrast theme: say so where the choice is made
  for (const n of $$('.gal-inert', form)) { n.hidden = !inert; const f = n.closest('fieldset'); if (inert) f.setAttribute('aria-describedby', n.id); else f.removeAttribute('aria-describedby') } // a hidden description would still be read
}
// Photosensitive safety (B8, WCAG 2.3.1): the radio selection follows focus at once, but the preview changes at most once per 400 ms,
// so holding an arrow key (about 33 Hz auto-repeat) makes at most 3 changes, 1.5 flash pairs, in any second (the panel and preview).
const GAP = 400, queued = new Map(); let lastApply = -Infinity, timer = 0
function flush() { timer = 0; lastApply = performance.now(); for (const [a, v] of queued) applyAxis(a, v); queued.clear() }
function queueAxis(axis, v) { queued.set(axis, v); if (timer) return; const wait = lastApply + GAP - performance.now(); if (wait <= 0) flush(); else timer = setTimeout(flush, wait) }
form.addEventListener('change', (e) => { if (e.target.name?.startsWith('a11y-')) { queueAxis(e.target.name.slice(5), e.target.value); renderChips() } })
// Actief (repair-2): one chip per axis that is not on its first value; a chip returns that axis alone, focus moves to the next chip
const optText = (inp) => [...inp.parentElement.childNodes].filter(n => n.nodeType === 3).map(n => n.textContent).join('').trim()
function renderChips() {
  const on = $$('fieldset[data-axis]', form).filter(f => $('input:checked', f).value !== '')
  const el = (tag, attrs, text) => { const n = document.createElement(tag); for (const [k, v] of attrs) n.setAttribute(k, v); if (text) n.textContent = text; return n }
  // DOM text goes in as text, never as HTML (CodeQL js/xss-through-dom); attribute order matches the former markup
  chips.replaceChildren(...(on.length ? on.map(f => {
    const leg = $('legend', f).textContent, v = optText($('input:checked', f)), d = optText($('input', f))
    const b = el('button', [['class', 'ds-btn'], ['data-variant', 'secondary'], ['type', 'button'], ['data-reset', f.dataset.axis], ['aria-label', `${leg}: ${v}. Terug naar ${d}`]], `${leg}: ${v}`)
    b.append(el('i', [['data-icon', 'close']])); const li = el('li', []); li.append(b); return li
  }) : [el('li', [['class', 'gal-tag']], 'Niets aangepast: je ziet BaryonOS zoals ontworpen, met de voorkeuren van je apparaat.')]))
  hydrate(chips)
}
chips.addEventListener('click', (e) => {
  const b = e.target.closest('[data-reset]'); if (!b) return
  const f = $(`fieldset[data-axis="${b.dataset.reset}"]`, form), first = $('input', f), i = $$('[data-reset]', chips).indexOf(b)
  first.checked = true; queueAxis(f.dataset.axis, ''); renderChips()
  const rest = $$('[data-reset]', chips); (rest[Math.min(i, rest.length - 1)] || $('#a11y-reset')).focus()
  $('#a11y-live').textContent = `${$('legend', f).textContent} staat weer op ${optText(first)}.`
})
matchMedia('(prefers-color-scheme: light)').addEventListener('change', () => { if (!qt) root.dataset.theme = deviceTheme(); $('.gal-sample[data-follow]', form).dataset.theme = deviceTheme(); if ($('input[name="a11y-theme"]:checked', form)?.value === '') queueAxis('theme', '') }) // the page, the Volg apparaat sample (always) and the preview (only while Volg apparaat is chosen) follow the device live; ?theme= pins the page (qt is set before any event runs)
// liveness of a long busy state after its finite loops end: elapsed time as text every 5 s (not a live region, not motion)
const t0 = performance.now()
setInterval(() => { const s = Math.floor((performance.now() - t0) / 5000) * 5; for (const n of $$('[data-elapsed]')) n.textContent = `Nog bezig · ${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}` }, 5000)
let undoState = null
const formState = () => Object.fromEntries($$('fieldset[data-axis]', form).map(f => [f.dataset.axis, $('input:checked', f).value]))
const setForm = (st) => { for (const [a, v] of Object.entries(st)) { $(`input[name="a11y-${a}"][value="${v}"]`, form).checked = true; queueAxis(a, v) } renderChips() }
$('#a11y-reset').addEventListener('click', () => {
  undoState = formState(); setForm(Object.fromEntries(Object.keys(undoState).map(a => [a, ''])))
  $('#a11y-live').textContent = 'Alle toegankelijkheidsinstellingen staan weer op standaard; je apparaatvoorkeuren gelden weer.'
  $('#a11y-undo').hidden = false
})
$('#a11y-undo').addEventListener('click', () => {
  if (undoState) setForm(undoState)
  undoState = null
  $('#a11y-live').textContent = 'Je vorige toegankelijkheidsinstellingen zijn terug.'; $('#a11y-undo').hidden = true; $('#a11y-reset').focus()
})
const qt = ['dark', 'light', 'contrast'].includes(q.get('theme')) ? q.get('theme') : '' // ?theme= preselects Thema, so the group itself renders in that theme
$(`input[name="a11y-theme"][value="${qt}"]`, form).checked = true
applyAxis('theme', qt); renderChips()
hydrate(document)

// ---------- permission model (contract.js ask/grants; owner decision of 14 November 2026, DESIGN-SYSTEM.md §11) ----------
const APP = { browser: T.apps.webvenster, notities: T.apps.notities }
const APP_ICON = { browser: 'browser', notities: 'notities' }
const REQ = {
  ongoing: { kind: 'ongoing', app: 'browser', resource: 'tuinplanner.example', right: 'connect', scope: 'domain' },
  action: { kind: 'action', app: 'browser', resource: 'tuinplan', right: 'send-copy', scope: 'document', to: 'tuinplanner.example' },
}
// gallery sample data (not strings.js copy; O-D1): the two requests' "what" and "done" fragments
const WHAT ={ ongoing: 'verbinding maken met tuinplanner.example', action: 'een kopie van Tuinplan naar tuinplanner.example sturen' }
const resultText = (kind, c) => {
  const R = T.trusted.result
  if (c === 'deny') return R.deny(APP.browser)
  if (c === 'once') return R.once(APP.browser, 'een kopie van Tuinplan naar tuinplanner.example gestuurd')
  return R[c](APP.browser, WHAT[kind])
}
// labels and durations come from strings.js (each defined once); the template text is the no-script fallback
function syncAsk(scope) {
  for (const a of $$('.ds-ask[data-kind]', scope)) {
    const kind = a.dataset.kind, terms = $$('.ds-ask-terms > li', a)
    $$('[data-choice]', a).forEach((b, i) => {
      const c = b.dataset.choice, d = T.trusted.duration[c]
      b.textContent = T.trusted.choices[kind][c]
      terms[i].innerHTML = `<strong>${b.textContent}.</strong> ${typeof d === 'function' ? d(APP.browser) : d}`
    })
  }
}
const collName = (id) => files.collections().find((c) => c.id === id)?.name
const resName = (id) => files.get(id)?.name ?? (collName(id) ? `collectie ${collName(id)}` : id)
// reach of a grant (design proposal for UX-E): the destination for a sending or connecting right, else every place where
// the resource is kept (a collection: the union of its files). It names reach, not a promise: read + internet grants can
// combine (LOCAL-PRIVACY-CONTRACT §4).
const KEPT = { local: ['device'], nas: ['nas'], both: ['device', 'nas'] }
const badge = (place, label) => `<span class="ds-place" data-place="${place}"><i class="ds-node" data-shape="${place}" aria-hidden="true"></i><i data-icon="${place}"></i>${label}</span>`
function reachBadge(g) {
  const to = destination(g)
  if (to) return badge('internet', `${T.instellingen.privacy.internet} · ${to}`)
  const kept = files.get(g.resource) ? [files.get(g.resource)] : files.list({ collection: g.resource })
  const places = [...new Set(kept.flatMap((f) => KEPT[f.storage] ?? ['device']))]
  return (places.length ? places : ['device']).map((p) => badge(p, T.instellingen.opslag[p])).join('')
}
// shape + icon + text: dashed chip + klok = ends at closing, solid chip + altijd (loop) = until revoked; the lock stays
// reserved for the Kern band (C1 repair-2)
const UNTIL_ICON = { close: 'klok', always: 'altijd' }
// one grant's labels; Intrekken and the revocation message name all four key fields (strings.js trusted.grant.revokeName)
function labels(g) {
  const G = T.trusted.grant, app = APP[g.app] ?? g.app, res = resName(g.resource)
  return { app, res, key: [app, G.target(res, g.to), G.rights[g.right] ?? g.right, G.scopes[g.scope] ?? g.scope] }
}
function renderGrants() {
  const G = T.trusted.grant, list = grants.all()
  for (const host of $$('[data-grants]')) {
    host.innerHTML = list.length ? list.map((g, i) => {
      const { app, res, key } = labels(g)
      const until = g.until === 'always' ? G.until.always : G.until.close(app)
      return `<li class="ds-grant" data-until="${g.until}"><span class="ds-grant-app" aria-hidden="true"><i data-icon="${APP_ICON[g.app] ?? 'dot'}"></i></span>`
        + `<div><p class="ds-grant-name">${app} · ${res}</p><p class="ds-grant-meta">${key[2]} · ${key[3]}</p>`
        + `<p class="ds-grant-reach">${reachBadge(g)}<span class="ds-grant-until" data-until="${g.until}"><i data-icon="${UNTIL_ICON[g.until]}"></i>${until}</span></p></div>`
        + `<button class="ds-btn" data-variant="secondary" data-revoke="${i}" aria-label="${G.revokeName(...key)}">${G.revoke}</button></li>`
    }).join('') : `<li><p class="ds-grant-empty" tabindex="-1">${G.empty}</p></li>`
    hydrate(host)
  }
}
// The open prompt's resolver. A choice resolves it at click time; close or Escape resolves 'deny' only while it is still
// unanswered, and a late 'close' event of an earlier prompt (the dialog is open again) is ignored (verify-0 minor).
let pending = null, askPanel = null
const answer = (c) => { const r = pending; pending = null; r?.(c) }
// gallery specimens: earlier answers so the grant rows show every duration and place, including a send grant for another
// destination (zaadruil.example) that does not cover the action prompt's tuinplanner.example
let specimenAnswer = null
setPrompt((req) => {
  if (specimenAnswer) { const c = specimenAnswer; specimenAnswer = null; return Promise.resolve(c) }
  return new Promise((resolve) => { answer('deny'); pending = resolve; openAskDialog(req.kind) })
})
for (const [req, c] of [
  [{ kind: 'ongoing', app: 'notities', resource: 'tuin', right: 'read', scope: 'collection' }, 'always'],
  [{ kind: 'ongoing', app: 'notities', resource: 'lofoten', right: 'read', scope: 'document' }, 'untilClose'],
  [{ ...REQ.action, to: 'zaadruil.example' }, 'always'],
]) { specimenAnswer = c; await ask(req) }
renderGrants()
bus.on('grants', renderGrants)
syncAsk(document)

const SW = {
  'Neutrale rollen': ['bg', 'surface-panel', 'surface', 'surface-raised', 'surface-inset', 'line', 'line-strong', 'text', 'text-muted', 'text-subtle', 'text-disabled', 'focus', 'selection', 'hover', 'hover-strong', 'blade', 'blade-glow', 'well', 'scrim', 'silver', 'rim-a', 'rim-b', 'rim-c', 'button-primary-bg', 'button-primary-bg-hover', 'button-primary-text', 'button-secondary-bg', 'pill-bg'].map(n => `--color-${n}`),
  'Status (S0 D1: tekstkleur, geen tint)': ['danger', 'warning', 'success'].map(n => `--color-${n}`),
  'Materialen': ['kern-bg', 'kern-tx', 'kern-tx2', 'kern-line', 'kern-rim', 'kern-warn-bg', 'kern-warn-line', 'kern-warn-ic', 'glass', 'glass-line', 'launch', 'launch-opaque', 'insp-bg', 'orbit-bg', 'tile-bg', 'tile-fg', 'tile-pc'].map(n => `--material-${n}`),
}
const PLACE = [['--place-device', 'Dit apparaat · groen'], ['--place-nas', 'NAS · blauw'], ['--place-internet', 'Internet · rood'], ['--place-internet-line', 'Internet, vraagrand']]
const ELEV = ['--elev-window', '--elev-window-inactive', '--elev-orbit', '--elev-plate', '--elev-launcher', '--elev-inspector', '--elev-trusted', '--material-tile-rim', '--material-tile-sh']
const swatch = (cs, name, label) => `<figure><span class="gal-chip" style="background: var(${name})"></span><figcaption>${label || name}<span>${label ? name + ' · ' : ''}${cs.getPropertyValue(name).trim()}</span></figcaption></figure>`
for (const sec of $$('#panels > .gal-panel')) {
  const cs = getComputedStyle(sec)
  $('[data-swatches="color"]', sec).innerHTML = Object.entries(SW).map(([h, list]) => `<p class="gal-sw-h">${h}</p>` + list.map(n => swatch(cs, n)).join('')).join('')
  $('[data-swatches="place"]', sec).innerHTML = PLACE.map(([n, l]) => swatch(cs, n, l)).join('')
  $('[data-swatches="elev"]', sec).innerHTML = ELEV.map(n => `<figure style="box-shadow: var(${n})">${n}</figure>`).join('')
}

// icon set and type scale (reference section)
const keys = Object.keys(ICONS)
$('#icon-count').textContent = `${keys.length} sleutels`
$('#icon-grid').append(...keys.map((n, i) => {
  const alias = keys.slice(0, i).find(o => ICONS[o] === ICONS[n])
  const f = document.createElement('figure')
  const c = document.createElement('figcaption')
  c.textContent = alias ? `${n} = ${alias}` : n
  f.append(icon(n, { size: 24 }), c)
  return f
}))
const TYPE = [['display', '09:41'], ['title', 'Tuinplan'], ['reading', 'Moestuin achterin: tomaten langs het hek, bonen tegen de schutting.'], ['body-strong', 'Webvenster vraagt internet'], ['body', 'Je werk staat op dit apparaat en op Thuis-NAS.'], ['body-dense', 'Huurcontract Spoorstraat 12 · 1,2 MB'], ['caption', 'Gewijzigd vandaag 09:12']]
const rcs = getComputedStyle(root)
$('#type-scale').innerHTML = TYPE.map(([r, s]) => {
  const v = (x) => rcs.getPropertyValue(`--type-${r}-${x}`).trim()
  const txt = /^\d\d:\d\d$/.test(s) ? `<span class="ds-num">${s}</span>` : s
  return `<div><span class="lab">--type-${r}-* · ${v('size')} / ${v('line')} · ${v('weight')}</span><span class="ds-t-${r}">${txt}</span></div>`
}).join('')

// ---------- behaviour ----------
const motion = $('#motion-toggle')
motion.setAttribute('aria-checked', root.dataset.motion === 'reduce')

const menuOf = (btn) => document.getElementById(btn.getAttribute('aria-controls'))
function setMenu(btn, open, focus) {
  const menu = menuOf(btn)
  menu.hidden = !open
  btn.setAttribute('aria-expanded', open)
  const items = $$('[role="menuitem"]', menu)
  if (open && focus) (focus === 'last' ? items.at(-1) : items[0]).focus()
}
function selectOption(opt) {
  const list = opt.closest('[role="listbox"]')
  for (const o of $$('[role="option"]', list)) { o.setAttribute('aria-selected', o === opt); o.tabIndex = o === opt ? 0 : -1 }
  opt.focus()
}
function activateTab(tab) {
  for (const t of $$('[role="tab"]', tab.closest('[role="tablist"]'))) {
    const on = t === tab
    t.setAttribute('aria-selected', on); t.tabIndex = on ? 0 : -1
    document.getElementById(t.getAttribute('aria-controls')).hidden = !on
  }
  tab.focus()
}
const step = (list, cur, key) => {
  const i = list.indexOf(cur), n = list.length
  return { ArrowDown: list[(i + 1) % n], ArrowRight: list[(i + 1) % n], ArrowUp: list[(i - 1 + n) % n], ArrowLeft: list[(i - 1 + n) % n], Home: list[0], End: list[n - 1] }[key]
}

// permission result: the panel's toast reports what happened (strings.js trusted.result.*)
function report(panel, text) {
  const toast = $('.ds-toast', panel)
  if (!toast) return
  $('.ds-toast-title', toast).textContent = 'Systeem'
  $('.ds-toast-body', toast).textContent = text
  toast.hidden = false
  const show = $('[data-toast-show]', panel); if (show) show.hidden = true
}
// a covered request (same app, resource, right and scope) never prompts; answer = a click on an inline specimen
async function request(kind, panel, given) {
  if (grants.covering(REQ[kind])) return report(panel, T.trusted.result.covered(APP.browser, WHAT[kind]))
  askPanel = panel; specimenAnswer = given ?? null
  const choice = await ask(REQ[kind])
  report(panel, resultText(kind, choice))
}

let opener = null
const askDlg = $('#dlg-ask'), delDlg = $('#dlg-delete')
function openDialog(dlg, from, theme, focus) {
  opener = from
  dlg.dataset.theme = theme
  dlg.returnValue = ''
  focus.autofocus = true // showModal's own focusing step lands here, not on the prompt's scrollable body (C1 repair-2)
  dlg.showModal()
  focus.focus()
}
function openAskDialog(kind) {
  const p = askPanel.dataset.theme, id = `m-${p}-${AB[kind]}`
  askDlg.innerHTML = fill(`tpl-ask-${kind}`, `m-${p}`, KERN[p]); hydrate(askDlg); syncAsk(askDlg)
  // one alertdialog: the <dialog> carries role, name and description; the template's inner copy becomes presentational
  const inner = $('.ds-ask', askDlg)
  inner.setAttribute('role', 'none'); inner.removeAttribute('aria-labelledby'); inner.removeAttribute('aria-describedby')
  askDlg.setAttribute('aria-labelledby', `${id}-t`); askDlg.setAttribute('aria-describedby', `${id}-w`)
  // initial focus: the prompt title (tabindex=-1), never a choice (DESIGN-SYSTEM.md §11)
  openDialog(askDlg, $(`[data-open-ask="${kind}"]`, askPanel), p, $('[data-initial-focus]', askDlg))
}
for (const dlg of [askDlg, delDlg]) {
  dlg.addEventListener('cancel', () => { dlg.returnValue = dlg === askDlg ? 'deny' : 'cancel' })
  dlg.addEventListener('close', () => {
    if (dlg.open) return // a late event of an earlier open: this dialog shows a newer prompt
    if (dlg === askDlg) answer('deny') // no-op when a choice already answered this open
    if (opener) opener.focus()
    opener = null
  })
}

document.addEventListener('click', (e) => {
  const t = e.target
  const sw = t.closest('[role="switch"]')
  if (sw && !sw.disabled) {
    const on = sw.getAttribute('aria-checked') !== 'true'
    sw.setAttribute('aria-checked', on)
    if (sw === motion) { if (on) root.dataset.motion = 'reduce'; else delete root.dataset.motion }
    return
  }
  const opt = t.closest('[role="option"]'); if (opt) return selectOption(opt)
  const tab = t.closest('[role="tab"]'); if (tab) return activateTab(tab)
  const trig = t.closest('[aria-haspopup="menu"]:not([role="menuitem"])')
  if (trig) return setMenu(trig, trig.getAttribute('aria-expanded') !== 'true', e.detail === 0 ? 'first' : null)
  const item = t.closest('[role="menuitem"]')
  if (item) {
    if (item.getAttribute('aria-disabled') === 'true' || item.hasAttribute('aria-haspopup')) return
    const btn = document.querySelector(`[aria-controls="${item.closest('[role="menu"]').id}"]`)
    setMenu(btn, false); return btn.focus()
  }
  const choice = t.closest('[data-choice]')
  if (choice) {
    const dlg = choice.closest('dialog')
    if (dlg) { answer(choice.dataset.choice); return dlg.close(choice.dataset.choice) }
    return request(choice.closest('.ds-ask').dataset.kind, choice.closest('.gal-panel'), choice.dataset.choice)
  }
  const revoke = t.closest('[data-revoke]')
  if (revoke) {
    const host = revoke.closest('[data-grants]'), panel = revoke.closest('.gal-panel'), i = +revoke.dataset.revoke, g = grants.all()[i]
    const G = T.trusted.grant, { key } = labels(g)
    grants.revoke(g) // emits 'grants', which re-renders every list
    const live = $('[data-grants-live]', panel) // looked up before the re-render detached the button
    live.textContent = ''; setTimeout(() => { live.textContent = G.revoked(...key) }, 30)
    // focus stays in the list: the next row's Intrekken, else the previous one, else the empty message
    const btns = $$('[data-revoke]', host)
    return (btns[Math.min(i, btns.length - 1)] || $('.ds-grant-empty', host)).focus()
  }
  if (t.closest('[data-close-app]')) { grants.appClosed('browser'); return report(t.closest('.gal-panel'), T.trusted.result.appClosed(APP.browser)) }
  if (t.closest('dialog button[value]')) return t.closest('dialog').close(t.closest('button').value)
  const openAsk = t.closest('[data-open-ask]')
  if (openAsk) return request(openAsk.dataset.openAsk, openAsk.closest('.gal-panel'))
  const openDel = t.closest('[data-open-delete]')
  if (openDel) return openDialog(delDlg, openDel, openDel.closest('.gal-panel').dataset.theme, $('[data-safe]', delDlg))
  const close = t.closest('[data-toast-close]')
  if (close) {
    const card = close.closest('.gal-card'); close.closest('.ds-toast').hidden = true
    const show = $('[data-toast-show]', card); show.hidden = false; return show.focus()
  }
  const show = t.closest('[data-toast-show]')
  if (show) { const card = show.closest('.gal-card'); $('.ds-toast', card).hidden = false; show.hidden = true; $('[data-toast-close]', card).focus() }
})

document.addEventListener('keydown', (e) => {
  const t = e.target
  const k = e.key
  const menu = t.closest('[role="menu"]')
  if (menu) {
    const btn = document.querySelector(`[aria-controls="${menu.id}"]`)
    if (k === 'Escape') { e.preventDefault(); setMenu(btn, false); return btn.focus() }
    if (k === 'Tab') return setMenu(btn, false)
    const next = step($$('[role="menuitem"]', menu), t.closest('[role="menuitem"]'), k)
    if (next) { e.preventDefault(); next.focus() }
    return
  }
  if (t.matches('[aria-haspopup="menu"]:not([role="menuitem"])')) {
    if (k === 'ArrowDown' || k === 'ArrowUp') { e.preventDefault(); return setMenu(t, true, k === 'ArrowUp' ? 'last' : 'first') }
    if (k === 'Escape' && t.getAttribute('aria-expanded') === 'true') { e.preventDefault(); return setMenu(t, false) }
  }
  if (t.matches('[role="tab"]')) {
    const next = step($$('[role="tab"]', t.closest('[role="tablist"]')), t, k)
    if (next) { e.preventDefault(); activateTab(next) }
    return
  }
  if (t.matches('[role="option"]')) {
    const next = (k === 'ArrowLeft' || k === 'ArrowRight') ? null : step($$('[role="option"]', t.closest('[role="listbox"]')), t, k)
    if (next) { e.preventDefault(); selectOption(next) }
    if (k === ' ') { e.preventDefault(); selectOption(t) }
    return
  }
  if (k === 'Escape') {
    const a = t.closest('.ds-tip-anchor'); if (a) a.dataset.dismissed = ''
  }
})
const undismiss = (e) => { const a = e.target.closest?.('.ds-tip-anchor'); if (a) delete a.dataset.dismissed }
document.addEventListener('focusout', undismiss)
document.addEventListener('mouseout', undismiss)

// ---------- query: ?only and ?measure ----------
const ONLY = { dark: '#panel-dark', light: '#panel-light', contrast: '#panel-contrast', a11y: '#panel-a11y', combos: '#panel-combos', scale: '#panel-scale', ref: '#panel-ref' }
if (ONLY[q.get('only')]) { root.dataset.only = q.get('only'); $(ONLY[q.get('only')]).dataset.show = '' }

const rgb = (s) => {
  s = s.trim()
  if (s.startsWith('#')) { const h = s.length === 4 ? [...s.slice(1)].map(c => c + c).join('') : s.slice(1, 7); return [0, 2, 4].map(i => parseInt(h.slice(i, i + 2), 16)) }
  const m = s.match(/[\d.]+/g); return m ? m.slice(0, 3).map(Number) : null
}
const lum = (c) => c.map(v => { v /= 255; return v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4 }).reduce((a, v, i) => a + v * [0.2126, 0.7152, 0.0722][i], 0)
const ratio = (a, b) => { const [x, y] = [lum(a), lum(b)].sort((m, n) => n - m); return +((x + 0.05) / (y + 0.05)).toFixed(2) }
function baseOf(el) {
  for (let n = el.parentElement; n; n = n.parentElement) {
    if (n.classList.contains('ds-ask')) return rgb(getComputedStyle(n).getPropertyValue('--material-kern-bg'))
    const c = getComputedStyle(n).backgroundColor
    if (c && !/rgba\(.*,\s*0\)$/.test(c) && c !== 'transparent') return rgb(c)
  }
  return rgb(getComputedStyle(document.body).backgroundColor)
}

// the element's own fill, composited over the base below it when translucent (an inset focus ring sits on it)
function ownBase(el) {
  const m = (getComputedStyle(el).backgroundColor.match(/[\d.]+/g) || []).map(Number), a = m.length > 3 ? m[3] : 1, b = baseOf(el)
  return !m.length || a === 0 ? b : m.slice(0, 3).map((v, i) => v * a + b[i] * (1 - a))
}

async function measure() {
  await document.fonts.ready
  await new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r)))
  const out = { errors: window.__errs }
  out.fonts = [...document.fonts].map(f => `${f.family} ${f.weight} ${f.status}`)
  out.interLoaded = [...document.fonts].some(f => f.family.replace(/"/g, '') === 'Inter' && f.status === 'loaded')
  const used = $$('svg.icon').map(s => [...s.classList].find(c => c.startsWith('icon-')).slice(5))
  out.icons = { keys: keys.length, rendered: used.length, distinctUsed: new Set(used).size, unresolved: [...new Set(used.filter(n => !(n in ICONS)))], placeholdersLeft: $$('i[data-icon]').length }
  out.unfilledPlaceholders = ['#panels', '#scale-host'].some(s => /\{[pk]\}/.test($(s).innerHTML))
  out.scrollWidth = root.scrollWidth
  const small = []
  let count = 0
  for (const el of $$('button, input, select, textarea, a[href], [role="option"], [role="tab"], [role="menuitem"]')) {
    if (el.closest('[inert], template') || el.disabled || !el.getClientRects().length) continue
    const box = (el.matches('input[type="checkbox"], input[type="radio"]') && el.closest('label')) || el
    const r = box.getBoundingClientRect(); count++
    if (r.width < 23.99 || r.height < 23.99) small.push(`${el.tagName}.${el.className} ${r.width.toFixed(1)}x${r.height.toFixed(1)}`)
  }
  out.targets = { checked: count, under24: small }
  // computed type sweep: every visible text element (zoomed strip excluded) must be Inter in a type-1.0 role (size/line/weight)
  const TY = { 13: ['18/400'], 15: ['22/400', '20/400', '22/600'], 16: ['26/400'], 22: ['28/600'], 48: ['56/400'] }
  const off = new Set(); let tn = 0
  for (const el of $$('body *')) {
    if (el.closest('template, [data-scale], #out, svg, dialog:not([open]), #a11y-preview, #combo-host, .gal-sample') || !el.getClientRects().length) continue // axis specimens and tile samples vary type on purpose (section 21)
    if (!el.matches('input:not([type="checkbox"], [type="radio"])') && ![...el.childNodes].some(c => c.nodeType === 3 && c.textContent.trim())) continue
    const c = getComputedStyle(el), sz = Math.round(parseFloat(c.fontSize)), lh = Math.round(parseFloat(c.lineHeight)), w = +c.fontWeight; tn++
    if (!/^"?Inter/.test(c.fontFamily) || !(TY[sz] || []).includes(`${lh}/${w}`)) off.add(`${el.tagName.toLowerCase()}.${el.className} ${c.fontFamily.split(',')[0]} ${sz}/${c.lineHeight}/${w}`)
  }
  out.type = { textElements: tn, offRole: [...off].slice(0, 20) }
  // focus ring: every interactive kind, in every theme; ring colour against the base it sits on
  const KINDS = ['.ds-btn[data-variant="primary"]', '.ds-btn[data-variant="secondary"]', '.ds-btn[data-variant="danger"]', '.ds-input', '.ds-switch', '.ds-check > input', '.ds-radio:not(.ds-seg > *) > input', '.ds-seg > .ds-radio > input', '.ds-row', '.ds-tab', '.ds-tabpanel', '.ds-window-ctl', '.ds-taskitem', '.ds-ask .ds-btn[data-choice="deny"]', '[aria-haspopup="menu"]']
  out.focus = {}
  for (const [p] of THEMES) for (const [h, host] of [['', ''], [' compact', ' [data-density="compact"]'], [' neutral', ' [data-surface="neutral"]']]) {
    out.focus[p + h] = [...KINDS, '.gal-canvas'].map(sel => {
      const el = $$(`#panel-${p}${host} ${sel}`).find(x => !x.closest('[inert]') && !x.disabled && x.getClientRects().length)
      if (!el) return `${sel}: none`
      el.focus({ focusVisible: true, preventScroll: true })
      const box = el.closest('.ds-seg > .ds-radio') || el // a segmented tile draws the ring for its transparent radio (repair-2)
      // a headless page without system focus never matches :focus-visible, so no ring was drawn and outline-color was the text
      // colour (run-2 repair 1); the focus rules also accept [data-demo~="focus"], which draws the same ring
      const fv = el.matches(':focus-visible'), demo = box.dataset.demo
      if (!fv) box.dataset.demo = ((demo || '') + ' focus').trim()
      const c = getComputedStyle(box), st = c.outlineStyle, w = c.outlineWidth, off = c.outlineOffset
      const res = st === 'none' ? 'no ring rendered' : `ring:base ${ratio(rgb(c.outlineColor), parseFloat(off) < 0 ? ownBase(box) : baseOf(box))}` // inset ring: on the element's own fill
      if (!fv) { if (demo === undefined) delete box.dataset.demo; else box.dataset.demo = demo }
      return `${sel}: fv=${fv}${fv ? '' : ' via data-demo'} ${st} ${w} off ${off} ${res}`
    })
  }
  document.activeElement.blur()
  // S0 D2: rendered sizes in the compact strips; S0 D3: inside a neutral host every colour is achromatic except place colours
  out.compact = Object.fromEntries(THEMES.map(([p]) => [p, ['.ds-btn', '.ds-input', '.ds-switch', '.ds-check', '.ds-radio', '.ds-tab', '.ds-row'].map(sel => {
    const r = $(`#panel-${p} [data-density="compact"] ${sel}`).getBoundingClientRect(); return `${sel} ${r.width.toFixed(0)}x${r.height.toFixed(1)}`
  })]))
  const tinted = new Set(); let nn = 0
  for (const el of $$('[data-surface="neutral"], [data-surface="neutral"] *')) {
    if (el.closest('.ds-place, .ds-node') || !el.getClientRects().length) continue
    const c = getComputedStyle(el); nn++
    for (const prop of ['color', 'backgroundColor', 'borderTopColor', 'outlineColor', 'fill', 'stroke']) {
      const v = rgb(c[prop]); if (v && c[prop] !== 'none' && Math.max(...v) - Math.min(...v) > 2) tinted.add(`${el.tagName.toLowerCase()}.${el.className.baseVal ?? el.className} ${prop} ${c[prop]}`)
    }
  }
  out.neutral = { elements: nn, tinted: [...tinted].slice(0, 20) }
  // keyboard handlers (synthetic keydown reaches these handlers only; native Tab/Space/radio arrows are exercised by kbd.mjs)
  const key = (el, k) => el.dispatchEvent(new KeyboardEvent('keydown', { key: k, bubbles: true, cancelable: true }))
  const tab1 = $('#panel-dark [role="tab"]'); tab1.focus(); key(tab1, 'ArrowRight')
  out.tabsArrow = document.activeElement.id + ' selected=' + document.activeElement.getAttribute('aria-selected') + ' panelShown=' + !$('#dark-tp-2').hidden
  key(document.activeElement, 'Home')
  const row = $('#panel-dark [role="option"][aria-selected="true"]'); row.focus(); key(row, 'ArrowDown')
  out.listArrow = document.activeElement.textContent.slice(0, 30) + ' selected=' + document.activeElement.getAttribute('aria-selected')
  key(document.activeElement, 'Home')
  const mbtn = $('#dark-menu-btn'); mbtn.focus(); key(mbtn, 'ArrowDown')
  const first = document.activeElement.textContent.trim().slice(0, 12); key(document.activeElement, 'ArrowDown')
  const second = document.activeElement.textContent.trim().slice(0, 12); key(document.activeElement, 'Escape')
  out.menu = `ArrowDown->${first} ArrowDown->${second} Escape->hidden=${$('#dark-menu').hidden} focus=${document.activeElement.id}`
  mbtn.click()
  $('#panel-dark [data-open-ask]').click()
  await new Promise(r => setTimeout(r, 50))
  out.askOpen = `open=${askDlg.open} activeIsTitle=${document.activeElement.matches('[data-initial-focus]')} kind=${$('.ds-ask', askDlg).dataset.kind} theme=${askDlg.dataset.theme} kern=${$('.ds-ask', askDlg).dataset.theme} labelledby=${!!document.getElementById(askDlg.getAttribute('aria-labelledby'))}`
  askDlg.close('deny'); await new Promise(r => setTimeout(r, 50))
  out.askClosed = `open=${askDlg.open} focusBack=${document.activeElement.matches('[data-open-ask]')} toast="${$('#panel-dark .ds-toast-body').textContent}"`
  $('#panel-dark [data-open-delete]').click()
  out.deleteOpen = `open=${delDlg.open} active=${document.activeElement.textContent.trim()}`
  delDlg.close('cancel'); await new Promise(r => setTimeout(r, 50)); document.activeElement.blur()
  // reduced motion
  const bar = $('#panel-dark .ds-progress[data-state="indeterminate"] .ds-progress-bar'), halo = $('#panel-dark .ds-node[data-state="ask"]')
  const anim = () => `bar=${getComputedStyle(bar).animationName}/${getComputedStyle(bar).width} halo=${getComputedStyle(halo, '::before').animationName} dur=${rcs.getPropertyValue('--motion-duration-base').trim()}`
  const was = root.dataset.motion
  delete root.dataset.motion; const a0 = anim()
  root.dataset.motion = 'reduce'; const a1 = anim()
  if (was) root.dataset.motion = was; else delete root.dataset.motion
  out.motion = { normal: a0, reduce: a1, media: matchMedia('(prefers-reduced-motion: reduce)').matches }
  // 200 % specimens (repair-1, repair-2): no element of .ds-empty, a progress field, the privacy strip or a settings fieldset is wider
  // than its box, and every label's text (Range rect) stays inside its box on both axes: scrollWidth cannot see vertical spill
  const SC = '#scale-host :is(.ds-empty, .ds-progress-field, .ds-fieldset, .ds-privacy)'
  const LB = `${SC} :is(.ds-btn, .ds-place, .ds-radio, .ds-empty-title, .ds-empty-place, .ds-progress-label > *, .ds-privacy-lbl, .ds-privacy-text > *)`
  out.scaleClip = $$(`${SC} *`).filter(e => e.getClientRects().length && e.scrollWidth > e.clientWidth + 1 && getComputedStyle(e).overflowX !== 'visible').map(e => `${e.className || e.tagName} ${e.scrollWidth}>${e.clientWidth}`).slice(0, 20)
  const lb = $$(LB).filter(b => b.getClientRects().length)
  out.scaleTextFit = { checked: lb.length, outside: lb.map(b => {
    const r = b.getBoundingClientRect(), g = document.createRange(); g.selectNodeContents(b); const t = g.getBoundingClientRect()
    const dy = Math.max(0, r.top - t.top, t.bottom - r.bottom), dx = Math.max(0, r.left - t.left, t.right - r.right)
    return dy > 1 || dx > 1 ? `${b.className} "${b.textContent.trim().slice(0, 24)}" x${Math.round(dx)} y${Math.round(dy)}` : ''
  }).filter(Boolean) }
  out.scale = (() => { const r = $('#scale-host .ds-btn').getBoundingClientRect(); return `zoom=${getComputedStyle($('#scale-host')).zoom} btnHeight=${r.height}` })()
  // ds-1.1 (matrix rows 24, 31): every animation in the gallery is finite, ends within 5 s and repeats at most every 1 s
  const loops = []; let animated = 0
  for (const el of $$('body *')) for (const pe of [null, '::before', '::after']) {
    const c = getComputedStyle(el, pe); if (c.animationName === 'none') continue
    const durs = c.animationDuration.split(','), its = c.animationIterationCount.split(',')
    c.animationName.split(',').forEach((n, i) => {
      animated++
      const ds = durs[i % durs.length].trim(), d = parseFloat(ds) * (ds.endsWith('ms') ? 1 : 1000), it = its[i % its.length].trim()
      const end = it === 'infinite' ? Infinity : d * +it
      if (end > 5000 || (+it > 1 && d < 1000 && d > 0)) loops.push(`${el.className?.baseVal ?? el.className}${pe || ''} ${n} ${d}ms x ${it}`)
    })
  }
  out.loops = { animated, over5sOrFast: loops.slice(0, 20) }
  out.combos = { rows: PW.rows.length, specimens: $$('#combo-host .gal-combo').length, pairs: PW.total, uncovered: PW.uncovered }
  const kern = $('#combo-host [data-theme="light"][data-contrast="increased"] .gal-kern')
  out.axisCascade = kern ? `Kern in light+increased: data-theme=${kern.dataset.theme} --color-text=${getComputedStyle(kern).getPropertyValue('--color-text').trim()} --color-text-muted=${getComputedStyle(kern).getPropertyValue('--color-text-muted').trim()}` : 'none'
  out.errorsAfter = window.__errs.length
  $('#out').textContent = JSON.stringify(out, null, 1)
  document.title += ' [measured]'
}
if (q.get('measure')) measure()
