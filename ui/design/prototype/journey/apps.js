// Slice A app bodies: Notities (Tuinplan), Rekenmachine, Bestanden (read-only list),
// Instellingen (Weergave, Toegankelijkheid; other categories follow). Others show a "volgt" body.
import { T, el, icon, files, bus } from '../contract.js'
import { appAccess, appName } from './access.js'
import { renderFiles, renderBrowser, renderFotos, settingsPage } from './stations.js'

// Notities holds one document at a time (single window). E11: the app side of the broker can read what the user opened and
// write only after a trusted edit or save action in the shell-drawn window bar; it can never grant anything itself.
const notes = appAccess('notities')
let doc = null // the document the user opened in Notities (shell sets it)
let draft = null // unsaved Notities text; survives minimising and station changes
let noteArea = null
export const noteDoc = () => doc
let insNote = null // the quiet in-app note after the Boodschappen request (station 7)
export const setNoteDoc = (id) => { doc = id; draft = null; insNote = null }
export const noteDirty = () => draft != null
export const discardNote = () => { draft = null }
// -> true when saved or nothing to save; false when the write failed closed (no readWrite grant): the draft is kept
export function saveNote() {
  if (draft == null || !doc) return true
  const text = draft
  const r = notes.write(doc, () => files.save(doc, text))
  if (r.ok) draft = null
  return r.ok
}

// Full .ds-empty anatomy as the gallery renders it (orbit with a place satellite, title, hint, actions), so the title never
// lands in the 72 px orbit column. The satellite names the place the app runs: this device.
export const emptyState = ({ id, tone = null, role = null, ic, title, hint, feas = null, actions = [] }) => el('section', { class: 'ds-empty', 'data-tone': tone, role, 'aria-labelledby': id, 'data-sim-scope': feas ? '' : null },
  el('div', { class: 'ds-empty-orbit', 'aria-hidden': 'true' }, icon(ic, { size: 28 }),
    el('span', { class: 'ds-empty-sat', 'data-place': 'device' }, el('i', { class: 'ds-node', 'data-place': 'device', 'data-shape': 'device', 'data-state': tone === 'error' ? 'off' : null }))),
  el('h3', { class: 'ds-empty-title', id }, tone === 'error' ? el('span', { class: 'ds-status-marker', 'aria-hidden': 'true' }) : null, title),
  el('p', { class: 'ds-empty-hint', 'data-sim-fn': feas ? '' : null }, hint, feas ? ' ' : null, feas),
  actions.length ? el('div', { class: 'ds-empty-actions' }, actions) : null)

const openable = (f) => !!f && typeof f.versions?.at(-1)?.body === 'string' // documents Notities can show
export const canOpenInNotes = (id) => openable(files.get(id))

export function renderApp(app, body, ctx) {
  const J = ctx.J
  const A = J.apps
  body.replaceChildren()
  const X = J.access
  const wid = body.parentElement.id || app
  const O = J.b.open, P = J.b.perm
  // the system file chooser (a shell-drawn dialog outside the app content): the user's pick in it is the open gesture
  const openBtn = () => el('button', { class: 'ds-btn', 'data-variant': 'primary', type: 'button', on: { click: (e) => ctx.chooser(e) } }, icon('bestanden', { size: 16 }), O.dots)
  if (app === 'notities' && !doc) {
    body.append(emptyState({ id: `${wid}-nodoc`, ic: 'notities', title: X.noDocTitle, hint: O.noDocHint, feas: ctx.feas('now'), actions: [openBtn()] }))
    return
  }
  if (app === 'notities' && !notes.canRead(doc)) { // fail closed: the grant ended (closed, revoked, signed out)
    body.append(emptyState({ id: `${wid}-nogrant`, ic: 'notities', title: X.revokedTitle(files.get(doc)?.name ?? doc), hint: X.revokedHint, feas: ctx.feas('now'), actions: [openBtn()] }))
    return
  }
  if (app === 'notities') {
    const f = files.get(doc)
    const rw = notes.canWrite(doc)
    const status = el('p', { class: 'jr-app-status', 'aria-live': 'polite' })
    const showStatus = () => status.replaceChildren(el('span', { class: 'jr-chip' }, rw ? X.modeWrite : X.modeRead), ' ', draft != null ? el('span', { class: 'jr-chip' }, J.shell.draft) : el('span', {}, `${f.name} · ${f.changed}`))
    noteArea = el('textarea', { class: 'ds-input jr-note-text', id: 'jr-note', 'aria-label': `${T.apps.notities}: ${f.name}`, 'aria-describedby': `${wid}-mode`, 'data-autofocus': '', spellcheck: 'false', readonly: !rw,
      on: { input: (e) => { if (!notes.canWrite(doc)) return; draft = e.target.value; showStatus() } } })
    noteArea.value = draft ?? f.versions.at(-1).body
    // station 7: Notities asks for a document the user did not hand it; the answer only ever adds to the draft after Bewerken
    const insert = el('button', { class: 'ds-btn', 'data-variant': 'quiet', type: 'button', on: { click: async () => {
      const res = await ctx.ask.request({ kind: 'ongoing', app: 'notities', right: 'read', data: 'boodschappen', destination: null, scope: 'document' }, { gesture: true })
      if (res.answer !== 'allow') insNote = `${P.notInserted} ${res.text}`
      else if (!notes.canWrite(doc)) insNote = P.needEdit
      else { draft = `${(draft ?? f.versions.at(-1).body).trimEnd()}\n\n${files.get('boodschappen').versions.at(-1).body}`; insNote = P.inserted }
      renderApp(app, body, ctx); ctx.live(insNote)
      body.querySelector('.jr-note-tools button')?.focus()
    } } }, icon('list', { size: 16 }), insNote && !insNote.startsWith(P.inserted) ? P.again : P.insertList)
    // compact labels (repair 1): one visible chip next to the mode; the model sentence and the native label in a disclosure
    body.append(el('div', { class: 'jr-app-bar', 'data-sim-scope': '' }, status, el('span', { 'data-sim-fn': '' }, ctx.feas('now')),
      el('details', { class: 'jr-info' }, el('summary', {}, O.info),
        el('p', { class: 'jr-pop-note', id: `${wid}-mode`, 'data-sim-fn': '' }, rw ? X.edited(f.name) : X.readHint(f.name), ' ', X.modelNow, ' ', ctx.feas('now'), ' ', X.modelNative, ' ', ctx.feas('feasible')))),
    el('div', { class: 'jr-row jr-note-tools' }, insert, insNote ? el('p', { class: 'jr-pop-note', role: 'status' }, insNote) : null), noteArea)
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
  if (app === 'bestanden') return renderFiles(body, ctx)
  if (app === 'browser') return renderBrowser(body, ctx)
  if (app === 'fotos') return renderFotos(body, ctx)
  if (app === 'instellingen') return settings(body, ctx)
  body.append(emptyState({ id: `${body.parentElement.id || app}-later`, ic: app, title: A.later(appName(app)), hint: J.tour.laterWhy }))
}

let page = 'toegankelijkheid'
let unsub = null
function settings(body, ctx) {
  const J = ctx.J
  const cats = T.instellingen.categories
  const BUILT = new Set(['weergave', 'toegankelijkheid', 'privacy', 'opslag', 'updates'])
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
    } else if (settingsPage(page, pane, h, ctx, () => { drawPane() })) {
      // slice B pages: Privacy en beveiliging (Appmachtigingen and session file grants), Opslag en NAS, Updates
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
