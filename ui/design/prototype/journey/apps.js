// Slice A app bodies: Notities (Tuinplan), Rekenmachine, Bestanden (read-only list),
// Instellingen (Weergave, Toegankelijkheid; other categories follow). Others show a "volgt" body.
import { T, DATA, el, icon, files, bus, grants } from '../contract.js'

let draft = null // unsaved Notities text; survives minimising and station changes
let noteArea = null
export const noteDirty = () => draft != null
export const discardNote = () => { draft = null }
export function saveNote() {
  if (draft == null) return
  files.save('tuinplan', draft)
  draft = null
}

// Full .ds-empty anatomy as the gallery renders it (orbit with a place satellite, title, hint, actions), so the title never
// lands in the 72 px orbit column. The satellite names the place the app runs: this device.
export const emptyState = ({ id, tone = null, role = null, ic, title, hint, feas = null, actions = [] }) => el('section', { class: 'ds-empty', 'data-tone': tone, role, 'aria-labelledby': id, 'data-sim-scope': feas ? '' : null },
  el('div', { class: 'ds-empty-orbit', 'aria-hidden': 'true' }, icon(ic, { size: 28 }),
    el('span', { class: 'ds-empty-sat', 'data-place': 'device' }, el('i', { class: 'ds-node', 'data-place': 'device', 'data-shape': 'device', 'data-state': tone === 'error' ? 'off' : null }))),
  el('h3', { class: 'ds-empty-title', id }, tone === 'error' ? el('span', { class: 'ds-status-marker', 'aria-hidden': 'true' }) : null, title),
  el('p', { class: 'ds-empty-hint', 'data-sim-fn': feas ? '' : null }, hint, feas ? ' ' : null, feas),
  actions.length ? el('div', { class: 'ds-empty-actions' }, actions) : null)

// Consent (OQ-5): Notities shows Tuinplan only while a grant covers it. The seeded temporary grant ends when the last Notities
// window closes; until slice B asks through the UX-C prompt, a reopened Notities fails closed and says why.
const tuinplanCovered = () => grants.temporary().some((g) => g.app === 'notities' && g.data === 'tuinplan')
  || grants.remembered().some((g) => g.app === 'notities' && g.data === 'tuinplan' && g.decision === 'allow')

export function renderApp(app, body, ctx) {
  const J = ctx.J
  const A = J.apps
  body.replaceChildren()
  if (app === 'notities' && !tuinplanCovered()) {
    body.append(emptyState({ id: `${body.parentElement.id || app}-nogrant`, ic: 'notities', title: J.more.noGrantTitle, hint: J.more.noGrantHint, feas: ctx.feas('simulated') }))
    return
  }
  if (app === 'notities') {
    const f = files.get('tuinplan')
    const status = el('p', { class: 'jr-app-status', 'aria-live': 'polite' })
    const showStatus = () => status.replaceChildren(draft != null ? el('span', { class: 'jr-chip' }, J.shell.draft) : el('span', {}, `${f.name} · ${f.changed}`))
    noteArea = el('textarea', { class: 'ds-input jr-note-text', id: 'jr-note', 'aria-label': J.shell.noteLabel, 'data-autofocus': '', spellcheck: 'false',
      on: { input: (e) => { draft = e.target.value; showStatus() },
        keydown: (e) => { if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 's' && !e.repeat) { e.preventDefault(); save() } } } })
    noteArea.value = draft ?? f.versions.at(-1).body
    const saved = el('p', { class: 'jr-pop-note', 'data-sim-fn': '', hidden: true }, J.shell.saved, ' ', ctx.feas('simulated'))
    const save = () => { const had = draft != null; saveNote(); showStatus(); if (had) { saved.hidden = false; ctx.live(J.shell.saved) } }
    body.append(el('div', { class: 'jr-app-bar', 'data-sim-scope': '' },
      el('button', { class: 'ds-btn', 'data-variant': 'secondary', type: 'button', on: { click: save } }, icon('check', { size: 16 }), J.session.save, el('kbd', { class: 'ds-kbd' }, 'Ctrl+S')), status, saved), noteArea)
    showStatus()
    return
  }
  if (app === 'rekenmachine') {
    let expr = ''
    const out = el('output', { class: 'jr-calc-out', 'aria-label': T.rekenmachine.display }, '0')
    const key = (label, aria, fn, variant = 'secondary') => el('button', { class: 'ds-btn', 'data-variant': variant, type: 'button', 'aria-label': aria, on: { click: fn } }, label)
    const put = (s) => () => { expr += s; out.textContent = expr }
    const calc = () => {
      // shortcut: digits and four operators only; no eval of user text
      const m = expr.match(/^(-?\d+(?:,\d+)?)([+\-x/])(\d+(?:,\d+)?)$/)
      if (!m) { out.textContent = expr || '0'; return }
      const [a, b] = [m[1], m[3]].map((v) => parseFloat(v.replace(',', '.')))
      const r = { '+': a + b, '-': a - b, x: a * b, '/': b ? a / b : NaN }[m[2]]
      expr = Number.isFinite(r) ? String(Math.round(r * 1e6) / 1e6).replace('.', ',') : ''
      out.textContent = expr || A.calcError
    }
    const grid = el('div', { class: 'jr-calc' },
      ['7', '8', '9'].map((d) => key(d, null, put(d))), key('÷', A.div, put('/')),
      ['4', '5', '6'].map((d) => key(d, null, put(d))), key('x', A.mul, put('x')),
      ['1', '2', '3'].map((d) => key(d, null, put(d))), key('–', A.minus, put('-')),
      key('0', null, put('0')), key(',', A.comma, put(',')), key('C', T.rekenmachine.clear, () => { expr = ''; out.textContent = '0' }), key('+', A.plus, put('+')),
      key('=', T.rekenmachine.equals, calc, 'primary'))
    body.append(out, grid)
    grid.querySelector('button').setAttribute('data-autofocus', '')
    return
  }
  if (app === 'bestanden') {
    const coll = Object.fromEntries(DATA.collections.map((c) => [c.id, c.name]))
    body.append(el('div', { class: 'ds-banner', 'data-tone': 'info', 'data-sim-scope': '', 'data-sim-fn': '' }, icon('dot', { size: 18 }), el('p', {}, J.shell.readOnly, ' ', ctx.feas('now'))),
      el('ul', { class: 'jr-plain jr-file-list', 'aria-label': T.apps.bestanden }, files.list().map((f) => el('li', { class: 'ds-row', 'data-sim-scope': '' },
        icon(f.kind === 'image' ? 'image' : f.kind === 'note' ? 'note' : 'doc', { size: 20 }),
        el('span', { class: 'ds-row-name' }, f.name),
        el('span', { class: 'ds-row-meta' }, coll[f.collection] ?? ''),
        el('span', { class: 'ds-place', 'data-place': f.storage === 'local' ? 'device' : 'nas', 'data-sim-fn': f.storage === 'local' ? null : '' },
          el('i', { class: 'ds-node', 'data-shape': f.storage === 'local' ? 'device' : 'nas', 'aria-hidden': 'true' }), icon(f.storage === 'local' ? 'device' : 'nas', { size: 16 }), f.storage === 'local' ? T.privacy.local : DATA.nas.name),
        f.storage === 'local' ? null : ctx.feas('simulated'),
        el('span', { class: 'ds-row-meta' }, f.changed)))))
    return
  }
  if (app === 'instellingen') return settings(body, ctx)
  body.append(emptyState({ id: `${body.parentElement.id || app}-later`, ic: app, title: A.later(T.apps[app] ?? app), hint: J.tour.laterWhy }))
}

let page = 'toegankelijkheid'
let unsub = null
function settings(body, ctx) {
  const J = ctx.J
  const cats = T.instellingen.categories
  const BUILT = new Set(['weergave', 'toegankelijkheid'])
  const nav = el('ul', { class: 'jr-plain jr-set-nav' })
  const pane = el('div', { class: 'jr-set-pane' })
  const q = el('input', { class: 'ds-input', type: 'search', id: 'jr-set-q', on: { input: () => drawNav() } })
  const seg = (axis, name, opts, cur, onchange) => el('fieldset', { class: 'ds-fieldset', 'data-axis': axis }, el('legend', {}, name),
    el('div', { class: 'ds-seg' }, opts.map(([v, l]) => el('label', { class: 'ds-radio' }, el('input', { type: 'radio', name: `jr-set-${axis}`, value: v, checked: cur === v, on: { change: () => onchange(v) } }), l))))
  function drawNav() {
    const term = q.value.trim().toLowerCase()
    const hit = Object.entries(cats).filter(([id, l]) => !term || l.toLowerCase().includes(term) || (id === 'toegankelijkheid' && Object.values(J.a11y.axes).some(([n]) => n.toLowerCase().includes(term))) || (id === 'weergave' && ['thema', 'schaal', 'contrast'].some((w) => w.includes(term))))
    nav.replaceChildren(...(hit.length ? hit.map(([id, l]) => el('li', {}, el('button', { class: 'ds-row', type: 'button', 'aria-current': id === page ? 'page' : null, on: { click: () => { page = id; drawNav(); drawPane(); pane.querySelector('h3').focus() } } },
      el('span', { class: 'ds-row-name' }, l), BUILT.has(id) ? null : el('span', { class: 'ds-row-meta' }, J.tour.later))))
      : [el('li', { class: 'jr-empty-line' }, J.shell.settingsNone(q.value.trim()))]))
  }
  function drawPane() {
    const h = el('h3', { tabindex: '-1', class: 'jr-set-h' }, cats[page], ' ', BUILT.has(page) ? ctx.feas('now') : null)
    if (page === 'weergave') {
      pane.replaceChildren(h, seg('theme', J.a11y.axes.theme[0], J.a11y.axes.theme[1], ctx.axes.theme ?? '', (v) => ctx.setAxis('theme', v)),
        seg('scale', J.a11y.scale, [['1', '100 %'], ['2', '200 %']], String(ctx.scale), (v) => ctx.setScale(+v)))
    } else if (page === 'toegankelijkheid') {
      const reset = el('button', { class: 'ds-btn', 'data-variant': 'secondary', type: 'button', on: { click: () => { const undo = ctx.resetAxes(); drawPane(); ctx.live(J.shell.resetDone); undoBtn.hidden = false; undoBtn.onclick = () => { undo(); undoBtn.hidden = true; drawPane() } } } }, J.shell.resetAll)
      const undoBtn = el('button', { class: 'ds-btn', 'data-variant': 'quiet', type: 'button', hidden: true }, J.shell.undo)
      pane.replaceChildren(h, el('p', { class: 'jr-lead' }, J.shell.a11yLead),
        ...Object.entries(J.a11y.axes).map(([axis, [name, opts]]) => seg(axis, name, opts, ctx.axes[axis] ?? '', (v) => ctx.setAxis(axis, v))),
        el('div', { class: 'jr-row' }, reset, undoBtn))
    } else {
      pane.replaceChildren(h, el('p', { class: 'jr-empty-line' }, J.tour.laterWhy))
    }
  }
  unsub?.()
  unsub = bus.on('jr:settings', (p) => { if (cats[p]) { page = p; drawNav(); drawPane(); pane.querySelector('h3').focus() } })
  if (ctx.args?.page && cats[ctx.args.page]) page = ctx.args.page
  body.append(el('div', { class: 'jr-settings' },
    el('div', { class: 'jr-set-side' }, el('div', { class: 'ds-field', 'data-variant': 'search' }, el('label', { class: 'ds-field-label', for: 'jr-set-q' }, J.shell.settingsSearch), el('span', { class: 'ds-input-wrap' }, icon('zoeken', { size: 18 }), q)),
      el('nav', { 'aria-label': T.apps.instellingen }, nav)), pane))
  drawNav(); drawPane()
  pane.querySelector('h3').setAttribute('data-autofocus', '')
}
