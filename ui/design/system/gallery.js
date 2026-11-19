// BaryonOS design system ds-1.1: component gallery behaviour. DESIGN ARTEFACT, 14 November 2026 (ds-1.1 15 November 2026); not a working OS.
// Builds one specimen set per theme from <template id="tpl-panel"> ({p} = id prefix, {k} = Kern theme, the opposite
// theme per MF1), renders icons with contract.js icon(), lists token swatches, and wires keyboard behaviour:
// Tab/Shift+Tab (native), Space/Enter (native buttons), arrows for tabs, listbox rows and menu (radios are native),
// Escape closes the menu, dialogs and tooltips. Query: ?only=dark|light|contrast|a11y|combos|scale|ref ?motion=reduce
// ?theme=dark|light|contrast ?measure=1. ds-1.1: the Toegankelijkheid settings group drives #a11y-preview; #combo-host renders a pairwise set.
import { icon, T, ask, setPrompt, grants, files, bus, createPermissions, appReason, rememberAllowOffered, needsConfirm } from '../prototype/contract.js'
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

// ---------- permission prompt (UX-C run 2; contract.js ask/grants; DESIGN-SYSTEM.md §11.1) ----------
// PROTOTYPE: remembered and temporary decisions live in this page only; BaryonOS does not enforce them.
const TR = T.trusted
const APP = { notities: T.apps.notities, browser: T.apps.webvenster, fotos: T.apps.fotos }
const APP_ICON = { notities: 'notities', browser: 'browser', fotos: 'fotos' }
const INSTALLED = { notities: '3 nov.', browser: '12 nov.', fotos: '3 nov.' } // gallery sample data (not strings.js copy; O-D1)
const esc = (s) => String(s).replace(/[&<>"']/g, (ch) => `&#${ch.charCodeAt(0)};`)
// gallery sample requests: one per right; `name` is the exact item's display name
const CASE = {
  read: { kind: 'ongoing', app: 'notities', right: 'read', data: 'tuinplan', destination: null, scope: 'document' },
  save: { kind: 'action', app: 'notities', right: 'save-copy', data: 'tuinplan', destination: null, scope: 'document' },
  send: { kind: 'action', app: 'browser', right: 'send-copy', data: 'tuinplan', destination: 'tuinplanner.example', scope: 'document', irreversible: true },
  combo: { kind: 'ongoing', app: 'browser', right: 'read-connect', data: 'tuinplan', destination: 'tuinplanner.example', scope: 'document' },
  camera: { kind: 'ongoing', app: 'fotos', right: 'camera', data: 'camera', destination: null, scope: 'while-open', reason: 'Om een foto van je tuin bij je plan te zetten.' },
}
const caseName = (r) => Object.keys(CASE).find((n) => CASE[n].app === r.app && CASE[n].right === r.right) ?? 'read'
const KEPT = { local: ['device'], nas: ['nas'], both: ['device', 'nas'] }
const badge = (place, label) => `<span class="ds-place" data-place="${place}"><i class="ds-node" data-shape="${place}" aria-hidden="true"></i><i data-icon="${place}"></i>${esc(label)}</span>`
const SENSOR = { camera: 'Camera', microphone: 'Microfoon', location: 'Locatie' } // device sensors: the item is the sensor itself
const itemName = (r) => SENSOR[r.right] ?? files.get(r.data)?.name ?? r.data
const places = (r) => (SENSOR[r.right] ? ['device'] : KEPT[files.get(r.data)?.storage] ?? ['device'])
const what = (r) => TR.what[r.right](itemName(r), r.destination)
const words = (r) => { const G = TR.grant; return [APP[r.app] ?? r.app, G.rights[r.right] ?? r.right, itemName(r), r.destination ?? TR.key.none, G.scopes[r.scope] ?? r.scope] }
// the five-field key as chips: the same markup in the prompt's disclosure and in every Appmachtigingen row
const keyChips = (r) => `<dl class="ds-key">${['app', 'right', 'data', 'destination', 'scope'].map((f, i) => `<div><dt>${TR.key.labels[f]}</dt><dd>${esc(words(r)[i])}</dd></div>`).join('')}</dl>`
// state sentence and outcome sentences for the remember checkbox state (MF-1: each states its real lifetime)
function lines(r, view, on) {
  const a = APP[r.app], w = what(r), O = TR.outcome
  if (!on) return { state: r.kind === 'action' ? TR.state.offAction(a) : TR.state.off(a), allow: O.allow[r.kind](a, w), deny: O.deny[r.kind](a) }
  if (!view.rememberAllow) return { state: TR.state.onDenyOnly, allow: O.allow.denyOnly(a, w), deny: O.deny.remembered(a) }
  if (view.confirmAllow) return { state: TR.state.onConfirm, allow: O.allow.confirm(a, w), deny: O.deny.remembered(a) }
  return { state: TR.state.on, allow: O.allow.remembered(a, w), deny: O.deny.remembered(a) }
}
const viewOf = (r, compact = false) => ({ compact, rememberAllow: rememberAllowOffered(r), confirmAllow: needsConfirm(r), reason: appReason(r.reason) })

// a leading icon per shell fact names its consequence class (in the order of TR.facts); the text stays the carrier
const FACT_ICON = {
  read: ['internet', 'toegang', 'nas'], readWrite: ['draw', 'internet', 'toegang', 'nas'],
  connect: ['internet', 'web', 'toegang'], 'save-copy': ['device', 'internet'], 'send-copy': ['warn', 'web'],
  'read-connect': ['internet', 'warn', 'toegang'], camera: ['internet', 'toegang', 'microfoon'], microphone: ['internet', 'toegang', 'camera'],
}
// one renderer for the live prompt and every specimen (p = id prefix, k = Kern theme)
function renderAsk(r, { p, k, view = viewOf(r), on = false, spec = '', preset = false, confirm = false }) {
  const a = APP[r.app], L = lines(r, view, on), cmp = TR.compare[r.right], held = r.right === 'read-connect'
  const F = TR.facts[r.right]
  const FI = FACT_ICON[r.right] ?? []
  const facts = (['read', 'readWrite'].includes(r.right) ? F(a, T.instellingen.opslag.nas, places(r).includes('nas')) : F(a, ...(r.right === 'read-connect' ? [itemName(r), r.destination] : [r.destination])))
    .map((t, i) => [FI[i] ?? 'dot', t]).filter(([, t]) => t)
  const src = r.right === 'connect' ? `<li class="ds-ask-stop" data-held><span class="ds-ask-stop-name">${esc(TR.path.appData(a))}</span></li>` // no document: the route starts at what the app already holds
    : `<li class="ds-ask-stop"${held ? ' data-held' : ''}><span class="ds-ask-stop-name">${esc(itemName(r))}</span>${places(r).map((pl) => badge(pl, T.instellingen.opslag[pl])).join('')}${held ? `<span class="ds-ask-held">${esc(a)} ${TR.path.already}</span>` : ''}</li>`
  const app = `<li class="ds-ask-stop" data-stop="app"><span class="ds-ask-link" aria-hidden="true"><i data-icon="chev"></i></span><span class="ds-ask-app" aria-hidden="true"><i data-icon="${APP_ICON[r.app]}"></i></span><span class="ds-ask-stop-name">${esc(a)}</span></li>`
  const dest = r.destination ? `<li class="ds-ask-stop" data-place="internet"><span class="ds-ask-link" aria-hidden="true"><i data-icon="chev"></i></span><span class="ds-ask-stop-name">${esc(r.destination)}</span>${badge('internet', T.instellingen.privacy.internet)}</li>` : ''
  const details = `<ol class="ds-ask-path" aria-label="${TR.path.label}">${src}${app}${dest}</ol>`
    + `<p class="ds-ask-holds">${esc(TR.path.holds(r.destination ?? a))}</p>`
    + (cmp ? `<div class="ds-ask-explain"><p>${esc(cmp.line)}</p>${cmp.why ? `<details><summary>${esc(cmp.why)}</summary><p>${esc(cmp.whyText)}</p></details>` : ''}</div>` : '')
    + `<details class="ds-ask-keybox"><summary>${TR.key.disclosure}</summary>${keyChips(r)}</details>`
  const body = `<div class="ds-ask-main"><div class="ds-ask-body">`
    + (spec ? `<p class="gal-spec">${esc(spec)}</p>` : '')
    + `<p class="ds-ask-proto"><i data-icon="warn"></i><span>${TR.prototype}</span></p>`
    + (preset ? `<p class="ds-ask-proto" data-preset><i data-icon="warn"></i><span>Testvoorinstelling: deze vraag opende met Keuze onthouden aan. Normaal staat het bij elke vraag uit.</span></p>` : '')
    + `<h3 class="ds-ask-title" id="${p}-t" tabindex="-1" data-initial-focus>${esc(TR.question[r.right](a, itemName(r), r.destination))}</h3>`
    + `<p class="ds-ask-from"><span class="ds-ask-app" aria-hidden="true"><i data-icon="${APP_ICON[r.app]}"></i></span><span><strong>${esc(a)}</strong> · ${esc(TR.provenance(INSTALLED[r.app]))}</span></p>`
    + (view.compact ? `<p class="ds-ask-again">${esc(TR.compact.note(a))}</p>` : '')
    + `<ul class="ds-ask-facts" id="${p}-f">${facts.map(([ic, f]) => `<li><span class="ds-ask-fact-icon" aria-hidden="true"><i data-icon="${ic}"></i></span><span>${esc(f)}</span></li>`).join('')}</ul>`
    + (view.reason ? `<p class="ds-ask-reason"><span class="ds-ask-reason-who">${esc(TR.reason(a))}</span> <span data-reason></span><span class="ds-ask-reason-tag">${TR.reasonTag}</span></p>` : '')
    + (view.compact ? `<details class="ds-ask-more"><summary>${TR.compact.details}</summary>${details}</details>` : details)
    + `</div><div class="ds-ask-decide"><div class="ds-ask-lifetime">`
    + `<label class="ds-check ds-ask-remember"><input type="checkbox" data-remember${on ? ' checked' : ''} aria-describedby="${p}-s">${TR.remember}</label>`
    + `<p class="ds-ask-state" id="${p}-s" data-on="${on}"><i data-icon="${on ? 'altijd' : 'klok'}"></i><span>${esc(L.state)}</span></p></div>`
    + `<ul class="ds-ask-outcomes"><li id="${p}-oa">${esc(L.allow)}</li><li id="${p}-od">${esc(L.deny)}</li></ul>` // MF-1: next to the answers, never scrolled away
    + `<div class="ds-ask-choices" role="group" aria-label="${TR.choicesLabel}">`
    + `<button class="ds-btn" data-variant="secondary" data-size="large" data-choice="allow" aria-describedby="${p}-oa">${TR.choices.allow}</button>`
    + `<button class="ds-btn" data-variant="secondary" data-size="large" data-choice="deny" aria-describedby="${p}-od">${TR.choices.deny}</button></div></div></div>`
  const conf = needsConfirm(r) ? `<div class="ds-ask-confirm"${confirm ? '' : ' hidden'}><div class="ds-ask-body">`
    + (spec ? `<p class="gal-spec">${esc(spec)}</p>` : '')
    + `<p class="ds-ask-proto"><i data-icon="warn"></i><span>${TR.prototype}</span></p>`
    + `<h3 class="ds-ask-title" id="${p}-ct" tabindex="-1" data-confirm-focus>${TR.confirm.title}</h3><p class="ds-ask-confirm-text" id="${p}-cd">${esc(TR.confirm.text(a, itemName(r), r.destination))}</p>${keyChips(r)}</div>`
    + `<div class="ds-ask-decide"><div class="ds-ask-choices"><button class="ds-btn" data-variant="secondary" data-size="large" data-confirm="back">${TR.confirm.back}</button><button class="ds-btn" data-variant="secondary" data-size="large" data-confirm="ok">${TR.confirm.ok}</button></div></div></div>` : ''
  return `<div class="ds-ask" data-theme="${k}" data-kind="${r.kind}"${view.compact ? ' data-compact' : ''}${confirm ? ' data-step="confirm"' : ''} role="alertdialog" aria-labelledby="${p}-${confirm ? 'ct' : 't'}" aria-describedby="${p}-${confirm ? 'cd' : 'f'}">`
    + `<div class="ds-ask-band"><i data-icon="lock"></i><span>${TR.origin}</span></div>${confirm ? body.replace('class="ds-ask-main"', 'class="ds-ask-main" hidden') : body}${conf}</div>`
}
// OQ-6: the app's text goes in as text, never as markup
const fillReason = (scope, r) => { for (const n of $$('[data-reason]', scope)) n.textContent = appReason(r.reason) ?? '' }
function syncAsk(ask, r, view) { // the checkbox changed: state sentence, icon and both outcome sentences follow
  const on = $('[data-remember]', ask).checked, L = lines(r, view, on), st = $('.ds-ask-state', ask)
  st.dataset.on = on; $('span', st).textContent = L.state
  st.querySelector('svg').replaceWith(icon(on ? 'altijd' : 'klok'))
  $('.ds-ask-outcomes > :first-child', ask).textContent = L.allow
  $('.ds-ask-outcomes > :last-child', ask).textContent = L.deny
}

// specimens: four decision states (ongoing and single action x remember off and on; the eight outcomes are in the table),
// the two OQ-1 states, the camera with an app reason and the compact repeat form. Illustrative: inert, no Tab stops.
const SPECS = [
  ['read', { on: false }, 'Voorbeeld · doorlopend · onthouden uit'],
  ['read', { on: true }, 'Voorbeeld · doorlopend · onthouden aan'],
  ['save', { on: false }, 'Voorbeeld · één handeling · onthouden uit'],
  ['save', { on: true }, 'Voorbeeld · één handeling · onthouden aan'],
  ['send', { on: true }, 'Voorbeeld · verzenden, onomkeerbaar · onthouden aan'],
  ['combo', { on: true, confirm: true }, 'Voorbeeld · document en internet · extra bevestiging'],
  ['camera', { on: false }, 'Voorbeeld · camera, met reden van de app'],
  ['save', { on: false, compact: true }, 'Voorbeeld · korte herhaalvraag'],
]
function fillSpecs(host, p, k, list) {
  host.innerHTML = list.map(([n, o, spec], i) => renderAsk(CASE[n], { p: `${p}-s${i}`, k, view: viewOf(CASE[n], o.compact), on: o.on, confirm: o.confirm, spec })).join('')
  for (const a of host.children) a.inert = true
  hydrate(host); fillReason(host, CASE.camera)
}
for (const h of $$('[data-ask-specs]')) { const p = h.closest('.gal-panel').dataset.theme; fillSpecs(h, p, KERN[p], SPECS) }
for (const h of $$('[data-ask-spec]')) fillSpecs(h, 's', 'light', [SPECS[0]])

// the eight decision states, computed by running the model (not written by hand)
const NEXT = { covered: 'Geen vraag: toegestaan tot de app sluit', remembered: 'Geen vraag: onthouden', 'temporary-deny': 'Geen vraag: stil geweigerd tot de app sluit', prompt: 'Opnieuw gevraagd', cancel: 'Opnieuw gevraagd' }
const rows = []
for (const n of ['read', 'save']) for (const choice of ['allow', 'deny']) for (const remember of [false, true]) {
  const P = createPermissions(); let first = true
  P.setPrompt(() => { const res = first ? { choice, remember } : { choice: 'cancel' }; first = false; return Promise.resolve(res) })
  const r = CASE[n], res = await P.ask(r), again = await P.ask(r)
  const L = lines(r, viewOf(r), remember)
  rows.push(`<tr><th scope="row">${r.kind === 'ongoing' ? 'Doorlopend' : 'Eén handeling'} · ${TR.choices[choice]} · ${remember ? 'onthouden' : 'niet onthouden'}</th><td>${esc(choice === 'allow' ? L.allow : L.deny)}</td><td>${NEXT[again.via]}${res.remembered ? ' · in Appmachtigingen' : ''}</td></tr>`)
}
for (const t of $$('[data-ask-states]')) t.innerHTML = rows.join('')

// Appmachtigingen: remembered allows AND remembered denials, each with "Precies:" (the prompt's key chips), change and forget
grants.seedRemembered({ kind: 'ongoing', app: 'notities', right: 'read', data: 'plattegrond', destination: null, scope: 'document' }, 'allow')
grants.seedRemembered({ ...CASE.send, destination: 'zaadruil.example' }, 'deny')
// covered-request specimen: a request the seeded remembered grant already covers gets no prompt, only this quiet shell note
{ const g = grants.remembered().find((x) => x.data === 'plattegrond' && x.decision === 'allow')
  for (const n of $$('[data-ask-covered] > p')) n.innerHTML = `<strong>Voorbeeld · al onthouden, dus geen vraag:</strong> ${esc(TR.quiet.covered(APP[g.app], what(g)))} ${TR.prototype}` }
function renderGrants() {
  const G = TR.grant, list = grants.remembered()
  for (const host of $$('[data-grants]')) {
    host.innerHTML = list.length ? list.map((g, i) => {
      const key = words(g).join(', '), to = g.decision === 'allow' ? 'deny' : 'allow'
      const change = to === 'deny' || grants.canChangeToAllow(g) ? `<button class="ds-btn" data-variant="secondary" data-change="${i}" data-to="${to}" aria-label="${esc(G.changeName(G.change[to], key))}">${G.change[to]}</button>` : ''
      return `<li class="ds-grant" data-decision="${g.decision}"><span class="ds-grant-app" aria-hidden="true"><i data-icon="${APP_ICON[g.app] ?? 'dot'}"></i></span>`
        + `<div><p class="ds-grant-name">${esc(APP[g.app] ?? g.app)} · ${esc(what(g))}</p><div class="ds-grant-key"><span>${G.precise}</span>${keyChips(g)}</div>`
        + `<p class="ds-grant-reach">${g.destination ? badge('internet', `${T.instellingen.privacy.internet} · ${g.destination}`) : places(g).map((pl) => badge(pl, T.instellingen.opslag[pl])).join('')}<span class="ds-grant-until" data-decision="${g.decision}"><i data-icon="altijd"></i>${G.decision[g.decision]}</span></p></div>`
        + `<div class="ds-grant-actions">${change}<button class="ds-btn" data-variant="secondary" data-forget="${i}" aria-label="${esc(G.forgetName(key))}">${G.forget}</button></div></li>`
    }).join('') : `<li><p class="ds-grant-empty" tabindex="-1">${G.empty}</p></li>`
    hydrate(host)
  }
}
renderGrants()
bus.on('permission', (d) => { if (['remember', 'revoke', 'change'].includes(d.op)) renderGrants() })

// the live prompt: present(request, view) opens #dlg-ask; Escape and closing without a choice cancel (owner decision 1)
let askPanel = null, live = null, lastCase = 'read', retries = 0
bus.on('permission', (d) => { if (d.op === 'quiet-deny') retries = d.retries })
const presetOn = q.get('remember') === '1' // a URL preset for testing; the prompt says so visibly (R3 holds otherwise)
const settle = (res) => { const l = live; live = null; l?.resolve(res) }
// contract.js shows one prompt at a time (a distinct request waits its turn), so opening never cancels an open prompt (OQ-4)
setPrompt((req, view) => new Promise((resolve) => {
  const p = askPanel.dataset.theme, preset = presetOn && !view.compact
  askDlg.innerHTML = renderAsk(req, { p: `m-${p}`, k: KERN[p], view, on: preset, preset })
  hydrate(askDlg); fillReason(askDlg, req); placeOutcomes()
  const inner = $('.ds-ask', askDlg) // one alertdialog: the <dialog> carries role, name and description
  inner.setAttribute('role', 'none'); inner.removeAttribute('aria-labelledby'); inner.removeAttribute('aria-describedby')
  askDlg.setAttribute('aria-labelledby', `m-${p}-t`); askDlg.setAttribute('aria-describedby', `m-${p}-f`)
  live = { resolve, req, view }
  openDialog(askDlg, $(`[data-open-ask="${caseName(req)}"]`, askPanel) ?? $('[data-open-ask]', askPanel), p, $('[data-initial-focus]', askDlg)) // focus: the question, never a choice (MF-4)
}))
// MF-1: the outcome sentences sit in the fixed decide row next to the answers. In a short viewport (200 % zoom of a common
// window, 320 x 640) or with the page-zoom setting they move to the end of the scrolling body, so the question stays readable
// and the checkbox and both answers stay reachable (WCAG 1.4.10, 2.4.11); aria-describedby links them either way.
const shortView = matchMedia('(max-height: 40rem)')
function placeOutcomes() {
  const d = document.getElementById('dlg-ask'), ul = $('.ds-ask-main .ds-ask-outcomes', d); if (!ul) return
  if (shortView.matches || root.dataset.scale === '2') $('.ds-ask-main .ds-ask-body', d).append(ul)
  else $('.ds-ask-main .ds-ask-decide', d).insertBefore(ul, $('.ds-ask-main .ds-ask-choices', d))
}
shortView.addEventListener('change', placeOutcomes)
new MutationObserver(placeOutcomes).observe(root, { attributes: true, attributeFilter: ['data-scale'] })
function note(panel, text, choose) {
  const n = $('[data-ask-note]', panel); if (!n) return
  $('[data-ask-note-text]', n).textContent = text
  const b = $('[data-choose]', n); b.hidden = !choose; if (choose) b.textContent = TR.quiet.choose(APP[CASE[lastCase].app])
}
async function request(name, panel, gesture) {
  askPanel = panel; lastCase = name
  const r = CASE[name], a = APP[r.app], w = what(r), res = await ask(r, { gesture })
  const R = TR.result, Q = TR.quiet
  if (res.via === 'remembered') return note(panel, res.answer === 'allow' ? Q.covered(a, w) : Q.deniedRemembered(a, w))
  if (res.via === 'covered') return note(panel, R.allow['app-session'](a, w))
  if (res.via === 'temporary-deny') return note(panel, Q.tempDeny(a, w, retries), true)
  if (res.via === 'cancel') return note(panel, R.cancel(a), true)
  if (res.via === 'error') return note(panel, R.error(a), true)
  if (res.remembered) return note(panel, R.remembered(a, w, TR.grant.decision[res.answer]))
  note(panel, res.answer === 'allow' ? R.allow[res.until](a, w) : R.deny(a), res.answer === 'deny')
}

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
for (const dlg of [askDlg, delDlg]) {
  dlg.addEventListener('cancel', () => { dlg.returnValue = 'cancel' })
  dlg.addEventListener('close', () => {
    if (dlg.open) return // a late event of an earlier open: this dialog shows a newer prompt
    if (dlg === askDlg) settle({ choice: 'cancel' }) // Escape or closing without a choice: fail closed, nothing stored (no-op once answered)
    if (opener?.isConnected) opener.focus()
    opener = null
  })
}
askDlg.addEventListener('change', (e) => { if (e.target.matches('[data-remember]') && live) syncAsk($('.ds-ask', askDlg), live.req, live.view) })

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
  const choice = t.closest('#dlg-ask [data-choice]')
  if (choice && live) {
    const on = $('[data-remember]', askDlg).checked
    if (choice.dataset.choice === 'allow' && on && live.view.confirmAllow) { // OQ-1: the extra confirmation, in the same dialog
      $('.ds-ask-main', askDlg).hidden = true; $('.ds-ask-confirm', askDlg).hidden = false
      askDlg.setAttribute('aria-labelledby', $('[data-confirm-focus]', askDlg).id); askDlg.setAttribute('aria-describedby', $('.ds-ask-confirm-text', askDlg).id)
      return $('[data-confirm-focus]', askDlg).focus()
    }
    settle({ choice: choice.dataset.choice, remember: on }); return askDlg.close()
  }
  const conf = t.closest('#dlg-ask [data-confirm]')
  if (conf && live) {
    if (conf.dataset.confirm === 'ok') { settle({ choice: 'allow', remember: true, confirmed: true }); return askDlg.close() }
    $('.ds-ask-confirm', askDlg).hidden = true; $('.ds-ask-main', askDlg).hidden = false
    askDlg.setAttribute('aria-labelledby', $('[data-initial-focus]', askDlg).id); askDlg.setAttribute('aria-describedby', $('.ds-ask-facts', askDlg).id)
    return $('[data-initial-focus]', askDlg).focus()
  }
  const g = t.closest('[data-forget], [data-change]')
  if (g) {
    const host = g.closest('[data-grants]'), panel = g.closest('.gal-panel') ?? document, i = +(g.dataset.forget ?? g.dataset.change), gr = grants.remembered()[i], key = words(gr).join(', ')
    const msg = g.dataset.forget != null ? (grants.revoke(gr), TR.grant.forgotten(key)) : (grants.change(gr, g.dataset.to), TR.grant.changed(key, TR.grant.decision[g.dataset.to]))
    const live2 = $('[data-grants-live]', panel); if (live2) { live2.textContent = ''; setTimeout(() => { live2.textContent = msg }, 30) }
    const btns = $$('[data-forget]', host) // focus stays in the list: the same row's Vergeten, else the previous one, else the empty message
    return (btns[Math.min(i, btns.length - 1)] || $('.ds-grant-empty', host)).focus()
  }
  const end = t.closest('[data-ask-end]')
  if (end) {
    const panel = end.closest('.gal-panel'), op = end.dataset.askEnd, app = CASE[lastCase].app
    grants[op](app); if (op !== 'windowHidden') retries = 0
    return note(panel, op === 'windowHidden' ? TR.result.hidden(APP[app]) : op === 'appClosed' ? TR.result.appClosed(APP[app]) : TR.result[op])
  }
  if (t.closest('[data-auto-ask]')) return request(lastCase, t.closest('.gal-panel'), false) // an automatic re-request: no gesture
  if (t.closest('[data-choose]')) return request(lastCase, t.closest('.gal-panel'), true) // the user chooses again: a new gesture
  if (t.closest('dialog button[value]')) return t.closest('dialog').close(t.closest('button').value)
  const openAsk = t.closest('[data-open-ask]')
  if (openAsk) return request(openAsk.dataset.openAsk, openAsk.closest('.gal-panel'), true)
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
  // B8 (WCAG 2.3.1, ds-1.1 run 3): one activation per physical key press. An auto-repeated Enter or Space never activates a
  // control again, so no dialog, menu, toast or switch opener can flip open and closed at the key-repeat rate (about 30 Hz)
  // when focus lands on a control that closes it (the delete dialog's Annuleren). Arrow keys and text fields keep their repeat.
  if (e.repeat && (k === 'Enter' || k === ' ') && t.matches('button, [role="button"], [role="switch"], [role="menuitem"], [role="tab"], summary, input:is([type="button"], [type="submit"], [type="reset"])')) return e.preventDefault()
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
  const KINDS = ['.ds-btn[data-variant="primary"]', '.ds-btn[data-variant="secondary"]', '.ds-btn[data-variant="danger"]', '.ds-input', '.ds-switch', '.ds-check > input', '.ds-radio:not(.ds-seg > *) > input', '.ds-seg > .ds-radio > input', '.ds-row', '.ds-tab', '.ds-tabpanel', '.ds-window-ctl', '.ds-taskitem', '[aria-haspopup="menu"]']
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
  out.askOpen = `open=${askDlg.open} activeIsTitle=${document.activeElement.matches('[data-initial-focus]')} remember=${$('[data-remember]', askDlg).checked} kind=${$('.ds-ask', askDlg).dataset.kind} theme=${askDlg.dataset.theme} kern=${$('.ds-ask', askDlg).dataset.theme} labelledby=${!!document.getElementById(askDlg.getAttribute('aria-labelledby'))}`
  askDlg.close(); await new Promise(r => setTimeout(r, 50))
  out.askClosed = `open=${askDlg.open} focusBack=${document.activeElement.matches('[data-open-ask]')} note="${$('#panel-dark [data-ask-note-text]').textContent}" remembered=${grants.remembered().length}`
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
