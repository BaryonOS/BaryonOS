// UX-E slice B app bodies: station 5 Bestanden ordenen, station 6 Versies (Bestanden inspector), station 7 in-app requests
// (Webvenster connect, Foto's camera; the Notities document read lives in apps.js) and station 8 Instellingen pages
// (Appmachtigingen, Opslag en NAS, Updates). PROTOTYPE ONLY: places, sync, versions on the NAS, the camera and the site are
// simulated; nothing here is enforced, stored, synchronised or protected by BaryonOS.
// No file-access facet here: documents open only through ctx.openFile / ctx.chooser, which the shell draws and checks.
import { T, DATA, el, icon, files, store, bus, grants, motionMs } from '../contract.js'
import { TR, APP, esc, keyChips, grantRows, words } from '../ask-view.js'
import { appName } from './access.js'

const B = () => T.journey.b
const KIND_ICON = { note: 'note', list: 'list', drawing: 'drawing', pdf: 'pdf', sheet: 'sheet', image: 'image' }
const openable = (f) => typeof f?.versions?.at(-1)?.body === 'string'
// journey-side place map (contract files has no place setter): 'device' | 'nas' | 'both' | 'external', initialised from
// file.storage. 'external' = this device plus an encrypted copy at the external provider the user chose (simulated).
const placeOf = {}
// B repair 2 (E1 5): the internet place. An external provider exists only after the user chooses it in Instellingen > Opslag
// en NAS (LOCAL-PRIVACY-CONTRACT 2.4: explicitly selected, opaque ciphertext only, never a fallback). Simulated: nothing is
// encrypted or sent. The provider is fictitious, like the site of station 7.
const PROVIDER = 'opslag.example'
const ext = { linked: false }
// ?test=1 sweep hook only: the linked provider with files kept there, and back (reset)
export function externalForTest(on, ids = []) { ext.linked = on; for (const id of Object.keys(placeOf)) if (placeOf[id] === 'external') placeOf[id] = 'device'; if (on) for (const id of ids) placeOf[id] = 'external' }
const where = (f) => (placeOf[f.id] ??= f.storage === 'local' ? 'device' : f.storage === 'nas' ? 'nas' : 'both')
const nasDown = (ctx) => ctx.sim.files === 'error' || store.get('nasOnline') === false
const hydrate = (scope) => { for (const ph of scope.querySelectorAll('i[data-icon]')) { const svg = icon(ph.dataset.icon); if (ph.className) svg.classList.add(...ph.className.split(/\s+/).filter(Boolean)); ph.replaceWith(svg) } }
const html = (tag, attrs, markup) => { const n = el(tag, attrs); n.innerHTML = markup; hydrate(n); return n }
// focus survives a re-render: the focused control carries data-fk, the new tree gets the same key focused again
const keepFocus = (body) => { const a = document.activeElement; return body.contains(a) ? a.dataset?.fk ?? null : null }
const restore = (body, fk) => { if (fk) body.querySelector(`[data-fk="${CSS.escape(fk)}"]`)?.focus() }

// place badge: colour only for place, always with node shape, icon and text (ds-1.1 .ds-place)
function badge(p, label, off = false) {
  return el('span', { class: 'ds-place', 'data-place': p }, el('i', { class: 'ds-node', 'data-shape': p, 'data-state': off ? 'off' : null, 'aria-hidden': 'true' }), icon(p, { size: 16 }), label)
}
function placeBadges(f, ctx) {
  const w = where(f), X = B().files, down = nasDown(ctx)
  if (w === 'external') return [badge('device', X.keepDevice), badge('internet', PROVIDER)]
  return [w !== 'nas' ? badge('device', X.keepDevice) : null, w !== 'device' ? badge('nas', DATA.nas.name, down) : null]
}

// ---------- station 5: Bestanden ordenen ----------
const fv = { side: 'all', group: 'place', q: '', sel: null, tab: 'versies', status: null, undo: null, travel: null, cmp: null, lifted: null }
bus.on('jr:files', (a) => { if (a?.select) fv.sel = a.select; if (a?.tab) fv.tab = a.tab; if (a?.side) fv.side = a.side })
export function setFilesView(v) { Object.assign(fv, v) } // used by the ?test=1 sweep hook and the router
export function renderFiles(body, ctx) {
  const fk = keepFocus(body)
  const X = B().files, J = ctx.J, BT = T.bestanden
  const redraw = () => renderFiles(body, ctx)
  const say = (text, undo = null) => { fv.status = text; fv.undo = undo; ctx.live(text); redraw() }
  const mode = ctx.sim.files
  if (mode === 'empty') { fv.side = 'cursus'; fv.fromEmpty = true } else if (fv.fromEmpty) { fv.side = 'all'; fv.fromEmpty = false } // leaving Leeg shows every file again
  const side = [['all', X.all, 'documents'], ...DATA.collections.map((c) => [c.id, c.name, 'coll']), ['p:device', X.keepDevice, 'device'], ['p:nas', DATA.nas.name, 'nas']]
  const nav = el('nav', { class: 'jr-files-side', 'aria-label': `${T.apps.bestanden}: ${X.places}` }, el('ul', { class: 'jr-plain' }, side.map(([id, label, ic]) =>
    el('li', {}, el('button', { class: 'ds-row', type: 'button', 'data-fk': `side:${id}`, 'aria-current': fv.side === id ? 'page' : null, on: { click: () => { fv.side = id; fv.status = null; redraw() } } }, icon(ic, { size: 18 }), el('span', { class: 'ds-row-name' }, label))))),
  el('p', { class: 'jr-pop-note', 'data-sim-scope': '', 'data-sim-fn': '' }, ext.linked ? X.extFiles(PROVIDER) : X.extChoose, ' ', ctx.feas('simulated')))

  const q = el('input', { class: 'ds-input', type: 'search', id: 'jr-files-q', value: fv.q, 'data-fk': 'q', on: { input: (e) => { fv.q = e.target.value; drawList(); } } })
  const grp = el('fieldset', { class: 'ds-fieldset jr-files-group' }, el('legend', {}, X.group), el('div', { class: 'ds-seg' }, [['place', X.byPlace], ['coll', X.byColl], ['kind', X.byKind]].map(([v, l]) =>
    el('label', { class: 'ds-radio' }, el('input', { type: 'radio', name: 'jr-files-group', value: v, checked: fv.group === v, 'data-fk': `grp:${v}`, on: { change: () => { fv.group = v; drawList() } } }), l))))
  const tools = el('div', { class: 'jr-files-tools' }, el('div', { class: 'ds-field', 'data-variant': 'search' }, el('label', { class: 'ds-field-label', for: 'jr-files-q' }, X.find), el('span', { class: 'ds-input-wrap' }, icon('zoeken', { size: 18 }), q)), grp)
  const status = el('p', { class: 'jr-app-status', role: 'status' }, fv.status ?? '', fv.undo ? ' ' : null,
    fv.undo ? el('button', { class: 'ds-btn', 'data-variant': 'quiet', type: 'button', 'data-fk': 'undo', on: { click: () => { const u = fv.undo; fv.undo = null; u() } } }, X.undo) : null)
  const listHost = el('div', { class: 'jr-files-list' })
  const insp = el('section', { class: 'jr-files-insp', 'aria-labelledby': 'jr-insp-h' })
  const main = el('div', { class: 'jr-files-main' }, tools, status, listHost)
  body.replaceChildren(el('div', { class: 'jr-files' }, nav, main, insp))

  function row(f) {
    const down = nasDown(ctx), nasOnly = where(f) === 'nas' && down
    const sel = fv.sel === f.id
    const coll = DATA.collections.find((c) => c.id === f.collection)
    return el('li', { class: 'ds-row jr-file-row', 'data-sel': sel ? '' : null, 'data-sim-scope': '' },
      icon(KIND_ICON[f.kind] ?? 'doc', { size: 20 }),
      el('button', { class: 'jr-file-name', type: 'button', 'aria-pressed': String(sel), 'data-fk': `sel:${f.id}`, on: { click: () => { fv.sel = f.id; fv.cmp = null; redraw() } } }, f.name),
      el('span', { class: 'ds-row-meta' }, coll?.name ?? X.noColl, ' · ', f.changed),
      el('span', { class: 'jr-file-places', 'data-sim-fn': where(f) !== 'device' ? '' : null }, placeBadges(f, ctx), where(f) !== 'device' ? ctx.feas('simulated') : null),
      nasOnly ? el('span', { class: 'ds-row-meta' }, X.nasOnly)
        : el('span', { class: 'jr-row' },
          openable(f) ? el('button', { class: 'ds-btn', 'data-variant': 'secondary', type: 'button', 'data-fk': `open:${f.id}`, 'aria-label': J.access.openIn(f.name, T.apps.notities), on: { click: (e) => ctx.openFile(f.id, e) } }, J.access.open) : null,
          el('button', { class: 'ds-btn', 'data-variant': 'quiet', type: 'button', 'data-fk': `with:${f.id}`, 'aria-label': X.openWithTitle(f.name), on: { click: (e) => ctx.chooser(e, { file: f.id }) } }, X.openWith)))
  }
  function drawList() {
    if (mode === 'loading') {
      listHost.replaceChildren(el('section', { class: 'ds-empty', 'data-state': 'busy', 'aria-busy': 'true', 'aria-labelledby': 'jr-files-load', 'data-sim-scope': '' },
        el('div', { class: 'ds-empty-orbit', 'aria-hidden': 'true' }, icon('bestanden', { size: 28 }), el('span', { class: 'ds-empty-sat', 'data-place': 'nas' }, el('i', { class: 'ds-node', 'data-place': 'nas', 'data-shape': 'nas' }))),
        el('h3', { class: 'ds-empty-title', id: 'jr-files-load' }, X.loadingFrom), el('p', { class: 'ds-empty-hint', 'data-sim-fn': '' }, BT.loading, ' ', ctx.feas('simulated'))))
      return
    }
    let list = files.list(fv.side.startsWith('p:') ? {} : fv.side === 'all' ? {} : { collection: fv.side })
    if (fv.side === 'p:device') list = list.filter((f) => where(f) !== 'nas')
    if (fv.side === 'p:nas') list = list.filter((f) => where(f) === 'nas' || where(f) === 'both')
    if (fv.q.trim()) list = list.filter((f) => f.name.toLowerCase().includes(fv.q.trim().toLowerCase()))
    const err = nasDown(ctx) ? el('div', { class: 'ds-banner', 'data-tone': 'error', role: 'status', 'data-sim-scope': '', 'data-sim-fn': '' }, icon('warn', { size: 18 }),
      el('div', {}, el('p', {}, el('strong', {}, X.errTitle), ' ', X.errHint, ' ', ctx.feas('simulated')),
        el('div', { class: 'jr-row' }, el('button', { class: 'ds-btn', 'data-variant': 'secondary', type: 'button', 'data-fk': 'retry', on: { click: () => { ctx.setSim('files', 'filled'); if (store.get('nasOnline') === false) ctx.setSim('nas', 'online') } } }, X.retry),
          el('button', { class: 'ds-btn', 'data-variant': 'quiet', type: 'button', on: { click: () => ctx.shell.open('instellingen', { page: 'opslag' }) } }, X.nasSettings)))) : null
    if (!list.length) {
      const c = DATA.collections.find((x) => x.id === fv.side)
      listHost.replaceChildren(err ?? '', el('section', { class: 'ds-empty', 'aria-labelledby': 'jr-files-empty' },
        el('div', { class: 'ds-empty-orbit', 'aria-hidden': 'true' }, icon('coll', { size: 28 }), el('span', { class: 'ds-empty-sat', 'data-place': 'device' }, el('i', { class: 'ds-node', 'data-place': 'device', 'data-shape': 'device' }))),
        el('h3', { class: 'ds-empty-title', id: 'jr-files-empty' }, c ? BT.emptyCollection : X.count(0)), el('p', { class: 'ds-empty-hint' }, c ? `${c.name}. ${X.emptyHint}` : X.emptyHint)))
      return
    }
    const key = (f) => (fv.group === 'coll' ? DATA.collections.find((c) => c.id === f.collection)?.name ?? X.noColl
      : fv.group === 'kind' ? BT.kinds?.[f.kind] ?? J.more.kinds?.[f.kind] ?? f.kind
      : { device: X.keepDevice, nas: X.keepNas, both: X.keepBoth, external: X.keepExt(PROVIDER) }[where(f)])
    const groups = new Map()
    for (const f of list) { const k = key(f); if (!groups.has(k)) groups.set(k, []); groups.get(k).push(f) }
    listHost.replaceChildren(err ?? '', el('p', { class: 'jr-pop-note' }, X.count(list.length)), ...[...groups].map(([k, fs], i) => el('section', { class: 'jr-files-grp', 'aria-labelledby': `jr-fg-${i}` },
      el('h3', { class: 'jr-pop-h', id: `jr-fg-${i}` }, k), el('ul', { class: 'jr-plain', 'aria-labelledby': `jr-fg-${i}` }, fs.map(row)))))
  }
  drawList()
  drawInspector(insp, ctx, redraw, say)
  restore(body, fk)
}

function drawInspector(insp, ctx, redraw, say) {
  const X = B().files
  const f = fv.sel ? files.get(fv.sel) : null
  if (!f) { insp.replaceChildren(el('h3', { class: 'jr-set-h', id: 'jr-insp-h' }, X.inspector), el('p', { class: 'jr-empty-line' }, X.selectHint)); return }
  const tabs = [['versies', T.bestanden.inspector.versions], ['opslag', T.bestanden.inspector.storage], ['ordenen', X.moveTo]]
  const tabBtn = ([id, label]) => el('button', { class: 'ds-tab', role: 'tab', type: 'button', id: `jr-tab-${id}`, 'aria-selected': String(fv.tab === id), 'aria-controls': 'jr-tabpanel', tabindex: fv.tab === id ? '0' : '-1', 'data-fk': `tab:${id}`,
    on: { click: () => { fv.tab = id; redraw() }, keydown: (e) => {
      const i = tabs.findIndex((t) => t[0] === fv.tab)
      const n = e.key === 'ArrowRight' ? i + 1 : e.key === 'ArrowLeft' ? i - 1 : e.key === 'Home' ? 0 : e.key === 'End' ? tabs.length - 1 : null
      if (n === null) return
      e.preventDefault(); fv.tab = tabs[(n + tabs.length) % tabs.length][0]; redraw()
      document.getElementById(`jr-tab-${fv.tab}`)?.focus() // the live tree: redraw() replaced `insp` and restored the old tab's focus key
    } } }, label)
  const panel = el('div', { class: 'ds-tabpanel', role: 'tabpanel', id: 'jr-tabpanel', 'aria-labelledby': `jr-tab-${fv.tab}` })
  insp.replaceChildren(el('h3', { class: 'jr-set-h', id: 'jr-insp-h' }, f.name), el('div', { class: 'ds-tabs' }, el('div', { class: 'ds-tablist', role: 'tablist', 'aria-label': X.inspector }, tabs.map(tabBtn)), panel))
  if (fv.tab === 'versies') return drawVersions(f, panel, ctx, redraw, say)
  if (fv.tab === 'ordenen') {
    const sel = el('select', { class: 'ds-input', id: 'jr-move-coll', 'data-fk': 'coll', on: { change: (e) => { files.move(f.id, e.target.value || null); say(T.bestanden.moved(f.name, DATA.collections.find((c) => c.id === e.target.value)?.name ?? X.noColl)) } } },
      el('option', { value: '' }, X.noColl), DATA.collections.map((c) => el('option', { value: c.id, selected: f.collection === c.id }, c.name)))
    panel.append(el('div', { class: 'ds-field' }, el('label', { class: 'ds-field-label', for: 'jr-move-coll' }, X.moveTo), sel),
      el('button', { class: 'ds-btn', 'data-variant': 'secondary', type: 'button', 'data-fk': 'remove', on: { click: () => {
        const id = f.id, name = f.name; files.remove(id); fv.sel = null
        say(T.bestanden.deleted(name), () => { files.unremove(id); fv.sel = id; say(X.back(name)) })
      } } }, icon('trash', { size: 16 }), X.remove))
    return
  }
  // Opslag: "Bewaren op" with the route strip. Metaphor (where data travels): a node moves from this device to Thuis-NAS once;
  // reduced motion shows the end state at once (duration tokens are 0 ms) and the text change carries the meaning.
  const w = where(f), down = nasDown(ctx)
  const phase = fv.travel?.id === f.id ? fv.travel.phase : w === 'device' ? 'local' : down && w !== 'external' ? 'wait' : 'done'
  const text = phase === 'travel' ? X.travelling : phase === 'wait' ? X.waitNas : phase === 'local' ? X.arrivedDevice : w === 'external' ? X.arrivedExt(PROVIDER) : w === 'nas' ? X.arrivedNas : X.arrivedBoth
  const to = w === 'external' ? 'internet' : 'nas' // the far end of the route: Thuis-NAS (blue) or the chosen provider (red)
  const set = (v) => {
    placeOf[f.id] = v
    if (v === 'device') { fv.travel = null; return say(X.keptAt(f.name, X.arrivedDevice)) }
    if (v === 'external') { // the provider is reached over the internet, not through Thuis-NAS: a NAS outage does not hold it
      fv.travel = { id: f.id, phase: 'travel' }; redraw(); ctx.live(X.keptAt(f.name, X.travelling))
      return setTimeout(() => { if (fv.travel?.id === f.id) { fv.travel = null; say(X.keptAt(f.name, X.arrivedExt(PROVIDER))) } }, motionMs(900) || 1)
    }
    if (nasDown(ctx)) { fv.travel = { id: f.id, phase: 'wait' }; return say(X.keptAt(f.name, X.waitNas)) }
    fv.travel = { id: f.id, phase: 'travel' }; redraw(); ctx.live(X.keptAt(f.name, X.travelling))
    setTimeout(() => { if (fv.travel?.id === f.id) { fv.travel = null; say(X.keptAt(f.name, v === 'nas' ? X.arrivedNas : X.arrivedBoth)) } }, motionMs(900) || 1)
  }
  panel.append(el('fieldset', { class: 'ds-fieldset' }, el('legend', {}, X.keepOn), el('div', { class: 'jr-files-keep' },
    [['device', X.keepDevice], ['nas', X.keepNas], ['both', X.keepBoth]].map(([v, l]) => el('label', { class: 'ds-radio' }, el('input', { type: 'radio', name: 'jr-keep', value: v, checked: w === v, 'data-fk': `keep:${v}`, on: { change: () => set(v) } }), l)),
    ext.linked ? el('label', { class: 'ds-radio' }, el('input', { type: 'radio', name: 'jr-keep', value: 'external', checked: w === 'external', 'data-fk': 'keep:external', on: { change: () => set('external') } }), X.keepExt(PROVIDER))
      : el('label', { class: 'ds-radio' }, el('input', { type: 'radio', name: 'jr-keep', value: 'external', disabled: true, 'aria-describedby': 'jr-keep-ext' }), X.external),
    ext.linked ? null : el('p', { class: 'jr-pop-note', id: 'jr-keep-ext' }, X.extChoose, ' ',
      el('button', { class: 'ds-btn', 'data-variant': 'quiet', type: 'button', 'data-fk': 'ext-settings', on: { click: () => ctx.shell.open('instellingen', { page: 'opslag' }) } }, X.nasSettings)))),
  el('div', { class: 'jr-route', 'data-phase': phase, 'data-sim-scope': '', role: 'group', 'aria-label': X.routeLabel },
    el('div', { class: 'jr-route-track', 'aria-hidden': 'true' },
      el('span', { class: 'jr-route-stop' }, el('i', { class: 'ds-node', 'data-place': 'device', 'data-shape': 'device', 'data-state': w === 'nas' && phase === 'done' ? 'off' : null })),
      el('span', { class: 'jr-route-line' }, el('span', { class: 'jr-route-dot' }), icon('chev', { size: 16 })),
      el('span', { class: 'jr-route-stop' }, el('i', { class: 'ds-node', 'data-place': to, 'data-shape': to, 'data-state': phase === 'done' ? null : 'off' }))),
    el('div', { class: 'jr-route-labels' }, badge('device', X.keepDevice), to === 'internet' ? badge('internet', PROVIDER) : badge('nas', DATA.nas.name, down)),
    el('p', { class: 'jr-route-text', 'data-sim-fn': '' }, text, ' ', ctx.feas('simulated'))))
}

// ---------- station 6: Versies en terugzetten ----------
function drawVersions(f, panel, ctx, redraw, say) {
  const V = B().versions, mode = ctx.sim.versions
  const vs = [...f.versions].reverse() // newest first
  panel.dataset.simScope = ''
  if (mode === 'loading') {
    panel.append(el('p', { class: 'ds-vh' }, V.loading), el('p', { class: 'jr-pop-note', 'data-sim-fn': '' }, V.loading, ' ', ctx.feas('simulated')),
      el('div', { class: 'ds-skeleton', 'aria-hidden': 'true' }, [64, 48, 56].map((wd) => el('div', { class: 'ds-skeleton-row' }, el('span', { class: 'ds-skeleton-line', style: { width: `${wd}%` } })))))
    panel.setAttribute('aria-busy', 'true')
    return
  }
  const shown = mode === 'error' || mode === 'empty' ? vs.slice(0, 1) : vs
  if (mode === 'error') panel.append(el('div', { class: 'ds-banner', 'data-tone': 'error', role: 'status', 'data-sim-fn': '' }, icon('warn', { size: 18 }),
    el('div', {}, el('p', {}, el('strong', {}, V.errTitle), ' ', V.errHint, ' ', ctx.feas('simulated')),
      el('button', { class: 'ds-btn', 'data-variant': 'secondary', type: 'button', 'data-fk': 'vretry', on: { click: () => ctx.setSim('versions', 'filled') } }, V.retry))))
  if (shown.length === 1 && mode !== 'error') panel.append(el('p', { class: 'jr-empty-line' }, V.one))
  panel.append(el('p', { class: 'jr-pop-note', 'data-sim-fn': '' }, V.of(f.name), ' ', ctx.feas('now')))
  const cur = vs[0]
  const diff = (old) => {
    if (typeof old.body !== 'string' || typeof cur.body !== 'string') return null
    const a = old.body.split('\n').filter(Boolean), b = cur.body.split('\n').filter(Boolean)
    const add = b.filter((l) => !a.includes(l)), rem = a.filter((l) => !b.includes(l))
    return el('div', { class: 'jr-diff', role: 'region', 'aria-label': V.compareTitle(old.n) },
      add.length || rem.length ? el('ul', { class: 'jr-plain' }, [...rem.map((l) => el('li', { 'data-diff': 'removed' }, el('strong', {}, `${V.removed}: `), l)), ...add.map((l) => el('li', { 'data-diff': 'added' }, el('strong', {}, `${V.added}: `), l))]) : el('p', {}, V.same))
  }
  // Metaphor: a stack of drafts on a desk. Restoring lifts the chosen sheet to the top (a new current version) and the former
  // current sheet stays right beneath it with "Bewaard" (contract files.restore never deletes). Reduced motion: no lift.
  panel.append(el('ol', { class: 'jr-plain jr-versions', 'aria-label': V.of(f.name) }, shown.map((v, i) => el('li', { class: 'jr-sheet', 'data-lift': fv.lifted === `${f.id}:${v.n}` ? '' : null },
    el('p', { class: 'jr-sheet-h' }, el('strong', {}, V.version(v.n, v.when)), ' ',
      i === 0 ? el('span', { class: 'jr-chip', 'data-on': '' }, V.current) : null, ' ',
      v.restoredFrom != null ? el('span', { class: 'jr-chip' }, V.restoredChip) : null,
      i === 1 && vs[0].restoredFrom != null ? el('span', { class: 'jr-chip' }, V.keptChip) : null),
    v.restoredFrom != null ? el('p', { class: 'jr-pop-note' }, V.restoredFrom(v.restoredFrom)) : null,
    i === 0 ? null : el('div', { class: 'jr-row' },
      typeof v.body === 'string' ? el('button', { class: 'ds-btn', 'data-variant': 'quiet', type: 'button', 'aria-expanded': String(fv.cmp === v.n), 'data-fk': `cmp:${v.n}`, on: { click: () => { fv.cmp = fv.cmp === v.n ? null : v.n; redraw() } } }, V.compare) : null,
      el('button', { class: 'ds-btn', 'data-variant': 'secondary', type: 'button', 'data-fk': `restore:${v.n}`, on: { click: () => {
        const n = files.restore(f.id, v.n); fv.cmp = null; fv.lifted = `${f.id}:${n}`
        setTimeout(() => { fv.lifted = null }, motionMs(400) || 1)
        say(T.bestanden.versionRestored(v.when), () => { files.undoRestore(f.id); say(V.undone) })
      } } }, V.restore)),
    fv.cmp === v.n ? diff(v) : null))))
}

// ---------- station 7: Webvenster (connect) and Foto's (camera) ----------
const SITE = 'tuinplanner.example'
const hasTemp = (app, right) => grants.temporary().some((g) => g.app === app && g.right === right) || grants.remembered().some((g) => g.app === app && g.right === right && g.decision === 'allow')
// B repair 3 (lead decision 5): which running apps use internet, from the same grants the Webvenster window reads (hasTemp),
// so the Orbit status, the desktop privacy instrument and the quick settings never contradict the app. [{app, dest}]
export function netInUse(running) {
  const out = []
  for (const g of [...grants.temporary(), ...grants.remembered().filter((x) => x.decision === 'allow')]) {
    if (g.right === 'connect' && running.has(g.app) && !out.some((o) => o.app === g.app && o.dest === g.destination)) out.push({ app: g.app, dest: g.destination })
  }
  return out
}
const appNote = {} // app -> {text, retry}: the quiet in-app note after an answer (never a focus-stealing dialog)
bus.on('permission', (d) => { if (d.op === 'logout') for (const k of Object.keys(appNote)) delete appNote[k] })
export const connectRequest = () => ({ kind: 'ongoing', app: 'browser', right: 'connect', data: TR.path.appData(APP.browser), destination: SITE, scope: 'domain' })
export const cameraRequest = () => ({ kind: 'ongoing', app: 'fotos', right: 'camera', data: 'camera', destination: null, scope: 'while-open', reason: B().perm.cameraReason })
const web = { phase: null } // null until connected in this app session; then 'loading' | 'done'
export function renderBrowser(body, ctx) {
  const P = B().perm, redraw = () => renderBrowser(body, ctx)
  const fk = keepFocus(body)
  const ok = hasTemp('browser', 'connect')
  if (!ok) web.phase = null
  const ask = async (e) => {
    const res = await ctx.ask.request(connectRequest(), { gesture: true })
    appNote.browser = res.answer === 'allow' ? null : { text: res.text, denied: true }
    if (res.answer === 'allow') { web.phase = 'loading'; setTimeout(() => { if (ctx.sim.web !== 'slow') { web.phase = 'done'; redraw() } }, 600) }
    redraw(); ctx.live(res.text)
    body.querySelector('[data-fk]')?.focus()
  }
  const bar = el('div', { class: 'jr-web-bar' }, icon('web', { size: 16 }), el('span', { class: 'jr-web-addr' }, ok ? SITE : ''), el('span', { class: 'jr-chip' }, ok ? P.usesInternet(APP.browser, SITE) : T.browser.siteLocal))
  const btn = (label) => el('button', { class: 'ds-btn', 'data-variant': 'primary', type: 'button', 'data-fk': 'connect', on: { click: ask } }, icon('web', { size: 16 }), label)
  let content
  if (!ok) {
    const n = appNote.browser
    content = n ? el('section', { class: 'ds-empty', 'data-tone': 'error', role: 'status', 'aria-labelledby': 'jr-web-h', 'data-sim-scope': '' }, el('div', { class: 'ds-empty-orbit', 'aria-hidden': 'true' }, icon('browser', { size: 28 })),
      el('h3', { class: 'ds-empty-title', id: 'jr-web-h' }, el('span', { class: 'ds-status-marker', 'aria-hidden': 'true' }), P.webError(SITE)),
      el('p', { class: 'ds-empty-hint', 'data-sim-fn': '' }, P.webDenied, ' ', ctx.feas('now')), el('p', { class: 'jr-pop-note' }, n.text), el('div', { class: 'ds-empty-actions' }, btn(P.again)))
      : el('section', { class: 'ds-empty', 'aria-labelledby': 'jr-web-h', 'data-sim-scope': '' }, el('div', { class: 'ds-empty-orbit', 'aria-hidden': 'true' }, icon('browser', { size: 28 }),
        el('span', { class: 'ds-empty-sat', 'data-place': 'internet' }, el('i', { class: 'ds-node', 'data-place': 'internet', 'data-shape': 'internet', 'data-state': 'off' }))),
      el('h3', { class: 'ds-empty-title', id: 'jr-web-h' }, P.webEmpty(SITE)), el('p', { class: 'ds-empty-hint', 'data-sim-fn': '' }, P.webEmptyHint, ' ', ctx.feas('now')), el('div', { class: 'ds-empty-actions' }, btn(P.connect(SITE))))
  } else if (ctx.sim.web === 'fail') {
    content = el('section', { class: 'ds-empty', 'data-tone': 'error', role: 'status', 'aria-labelledby': 'jr-web-h', 'data-sim-scope': '' }, el('div', { class: 'ds-empty-orbit', 'aria-hidden': 'true' }, icon('browser', { size: 28 })),
      el('h3', { class: 'ds-empty-title', id: 'jr-web-h' }, el('span', { class: 'ds-status-marker', 'aria-hidden': 'true' }), P.webError(SITE)),
      el('p', { class: 'ds-empty-hint', 'data-sim-fn': '' }, P.webFail(SITE), ' ', ctx.feas('simulated')),
      el('div', { class: 'ds-empty-actions' }, el('button', { class: 'ds-btn', 'data-variant': 'primary', type: 'button', 'data-fk': 'retry', on: { click: () => ctx.setSim('web', 'normal') } }, P.retry)))
  } else if (web.phase === 'loading' || ctx.sim.web === 'slow') {
    content = el('div', { class: 'jr-web-load', 'aria-busy': 'true', 'data-sim-scope': '' }, el('p', { 'data-sim-fn': '' }, P.webLoading(SITE), ' ', ctx.feas('simulated')),
      el('div', { class: 'ds-progress', role: 'progressbar', 'data-state': 'indeterminate', 'aria-label': P.webLoading(SITE) }, el('span', { class: 'ds-progress-bar' })))
  } else {
    content = el('article', { class: 'jr-web-site', 'data-sim-scope': '' }, el('div', { class: 'ds-banner', 'data-tone': 'info', 'data-sim-fn': '' }, icon('warn', { size: 18 }), el('p', {}, P.siteBand, ' ', ctx.feas('simulated'))),
      el('h3', {}, P.siteTitle), el('p', {}, P.siteText))
  }
  body.replaceChildren(el('div', { class: 'jr-web' }, bar, content))
  restore(body, fk)
}
bus.on('jr:sim', (g) => { if (g === 'web' && web.phase === 'loading') web.phase = 'done' }) // a held load finishes when Traag ends

export function renderFotos(body, ctx) {
  const P = B().perm, redraw = () => renderFotos(body, ctx)
  const fk = keepFocus(body)
  const ok = hasTemp('fotos', 'camera')
  const ask = async () => {
    const res = await ctx.ask.request(cameraRequest(), { gesture: true })
    appNote.fotos = { text: res.text, denied: res.answer !== 'allow' }
    redraw(); ctx.live(res.text)
  }
  const n = appNote.fotos
  body.replaceChildren(el('div', { class: 'jr-fotos', 'data-sim-scope': '' },
    el('div', { class: 'jr-row' }, el('h3', { class: 'jr-set-h' }, T.fotos.all), ok ? el('span', { class: 'jr-chip', 'data-on': '' }, icon('camera', { size: 14 }), P.usesCamera(APP.fotos)) : null),
    ok ? el('figure', { class: 'jr-viewfinder', 'data-sim-fn': '' }, el('span', { class: 'jr-viewfinder-frame', 'aria-hidden': 'true' }), el('figcaption', {}, P.viewfinder, ' ', ctx.feas('simulated')),
      el('button', { class: 'ds-btn', 'data-variant': 'primary', type: 'button', 'data-fk': 'shutter', on: { click: () => { appNote.fotos = { text: P.shot }; redraw(); ctx.live(P.shot) } } }, icon('camera', { size: 16 }), P.shutter))
      : el('div', { class: 'jr-row' }, el('button', { class: 'ds-btn', 'data-variant': 'primary', type: 'button', 'data-fk': 'photo', on: { click: ask } }, icon('camera', { size: 16 }), n?.denied ? B().perm.again : P.photo)),
    n ? el('p', { class: 'jr-pop-note', role: 'status' }, n.text, n.denied ? ` ${P.noCamera}` : '') : el('p', { class: 'jr-empty-line', 'data-sim-fn': '' }, T.fotos.empty, ' ', ctx.feas('now'))))
  restore(body, fk)
}
// the app session ends with the app (OQ-5): its in-app note and connection state go with it
export function appEnded(app) { delete appNote[app]; if (app === 'browser') web.phase = null }

// ---------- station 8: Instellingen pages ----------
// Externe opslag (E1 5, B repair 2): the user chooses the provider here, after reading what it gets and what it never gets;
// Bestanden then offers it per file. Choosing it is the explicit selection; loosening it returns every copy to this device.
function externalStorage(S, ctx, redraw, pane) {
  const link = () => { ext.linked = true; redraw(); ctx.live(S.extLinkedLive(PROVIDER)); pane.querySelector('[data-fk="ext-unlink"]')?.focus() }
  const unlink = () => {
    const moved = Object.keys(placeOf).filter((id) => placeOf[id] === 'external')
    for (const id of moved) placeOf[id] = 'device'
    ext.linked = false; redraw(); ctx.live(S.extUnlinkedLive(PROVIDER, moved.length)); pane.querySelector('[data-fk="ext-link"]')?.focus()
  }
  return el('section', { class: 'jr-ext', 'aria-labelledby': 'jr-ext-h', 'data-sim-scope': '' },
    el('h4', { class: 'jr-set-h', id: 'jr-ext-h' }, S.extTitle), el('p', { class: 'jr-lead' }, S.extLead),
    el('ul', { class: 'jr-ext-facts' }, S.extFacts.map((t) => el('li', {}, t))),
    ext.linked
      ? el('div', { class: 'jr-row' }, badge('internet', PROVIDER), el('button', { class: 'ds-btn', 'data-variant': 'secondary', type: 'button', 'data-fk': 'ext-unlink', on: { click: unlink } }, S.extUnlink))
      : el('div', { class: 'jr-row' }, el('button', { class: 'ds-btn', 'data-variant': 'secondary', type: 'button', 'data-fk': 'ext-link', on: { click: link } }, icon('internet', { size: 16 }), S.extLink(PROVIDER))),
    el('p', { class: 'jr-pop-note', 'data-sim-fn': '' }, ext.linked ? `${S.extLinked(PROVIDER)} ` : '', S.extSim, ' ', ctx.feas('simulated')))
}
const upd = { phase: null }
export function settingsPage(page, pane, h, ctx, redraw) {
  const S = B().settings
  if (page === 'privacy') {
    const X = ctx.J.access
    const remembered = grants.remembered()
    const perms = html('ul', { class: 'ds-grants', 'aria-labelledby': 'jr-perm-h' }, grantRows(remembered))
    const liveG = el('p', { class: 'ds-live', role: 'status' })
    perms.addEventListener('click', (e) => {
      const g = e.target.closest('[data-forget], [data-change]'); if (!g) return
      const i = +(g.dataset.forget ?? g.dataset.change), gr = grants.remembered()[i], key = words(gr).join(', ')
      const msg = g.dataset.forget != null ? (grants.revoke(gr), TR.grant.forgotten(key)) : (grants.change(gr, g.dataset.to), TR.grant.changed(key, TR.grant.decision[g.dataset.to]))
      redraw(); ctx.live(msg)
      const btns = pane.querySelectorAll('.ds-grants [data-forget]'); (btns[Math.min(i, btns.length - 1)] ?? pane.querySelector('.ds-grant-empty'))?.focus()
    })
    // session file grants (E11 c): the same .ds-grant anatomy, with the five-field key and "Tot sluiten", revocable here
    const open = ctx.openFiles()
    const rows = open.map((g, i) => {
      const r = { app: g.app, right: g.level, data: g.doc, destination: null, scope: 'document' }
      const name = files.get(g.doc)?.name ?? g.doc, a = appName(g.app)
      return `<li class="ds-grant" data-decision="allow"><span class="ds-grant-app" aria-hidden="true"><i data-icon="${g.app === 'notities' ? 'notities' : 'apps'}"></i></span>`
        + `<div><p class="ds-grant-name">${esc(a)} · ${esc(name)} · ${esc(S.fileRight[g.level] ?? g.level)}</p><div class="ds-grant-key"><span>${TR.grant.precise}</span>${keyChips(r)}</div>`
        + `<p class="ds-grant-reach"><span class="ds-grant-until" data-decision="allow"><i data-icon="klok"></i>${esc(S.untilClose)}</span></p></div>`
        + `<div class="ds-grant-actions"><button class="ds-btn" data-variant="secondary" type="button" data-revoke-file="${i}" aria-label="${esc(`${T.instellingen.privacy.revoke}: ${a}, ${name}`)}">${esc(T.instellingen.privacy.revoke)}</button></div></li>`
    }).join('')
    const sess = html('ul', { class: 'ds-grants', 'aria-labelledby': 'jr-acc-h' }, rows || `<li><p class="ds-grant-empty" tabindex="-1">${esc(X.grantsEmpty)}</p></li>`)
    sess.addEventListener('click', async (e) => {
      const b = e.target.closest('[data-revoke-file]'); if (!b) return
      const g = open[+b.dataset.revokeFile], name = files.get(g.doc)?.name ?? g.doc
      if (await ctx.revokeFile(g.app, g.doc)) { redraw(); ctx.live(X.revoked(appName(g.app), name)); (pane.querySelector('[data-revoke-file]') ?? pane.querySelector('#jr-acc-h'))?.focus() }
    })
    // a document read the user allowed in the prompt (station 7) is a contract temporary grant; it ends with the app (OQ-5) or
    // earlier with Intrekken here, which ends exactly that five-field key (grants.endTemporary via the shell; B repair 3)
    const viaPrompt = grants.temporary().filter((g) => g.scope === 'document' && !open.some((o) => o.app === g.app && o.doc === g.data))
    const pdocList = html('ul', { class: 'ds-grants', 'aria-labelledby': 'jr-pdoc-h' }, viaPrompt.map((g, i) => {
        const name = files.get(g.data)?.name ?? g.data, a = appName(g.app)
        return `<li class="ds-grant" data-decision="allow"><span class="ds-grant-app" aria-hidden="true"><i data-icon="${g.app === 'notities' ? 'notities' : 'apps'}"></i></span>`
          + `<div><p class="ds-grant-name">${esc(a)} · ${esc(name)} · ${esc(S.fileRight[g.right] ?? g.right)}</p><div class="ds-grant-key"><span>${TR.grant.precise}</span>${keyChips(g)}</div>`
          + `<p class="ds-grant-reach"><span class="ds-grant-until" data-decision="allow"><i data-icon="klok"></i>${esc(S.promptDocUntil(a))}</span></p></div>`
          + `<div class="ds-grant-actions"><button class="ds-btn" data-variant="secondary" type="button" data-end-doc="${i}" aria-label="${esc(`${T.instellingen.privacy.revoke}: ${a}, ${name}`)}">${esc(T.instellingen.privacy.revoke)}</button></div></li>`
      }).join(''))
    pdocList.addEventListener('click', async (e) => {
      const b = e.target.closest('[data-end-doc]'); if (!b) return
      const g = viaPrompt[+b.dataset.endDoc], name = files.get(g.data)?.name ?? g.data
      if (await ctx.revokeFile(g.app, g.data)) { redraw(); ctx.live(X.revoked(appName(g.app), name)); (pane.querySelector('[data-end-doc]') ?? pane.querySelector('[data-revoke-file]') ?? pane.querySelector('#jr-acc-h'))?.focus() }
    })
    const pdocs = viaPrompt.length ? [el('h4', { class: 'jr-set-h', id: 'jr-pdoc-h' }, S.promptDocsTitle), el('p', { class: 'jr-lead' }, B().r3.promptDocsLead), pdocList] : []
    pane.replaceChildren(h,
      el('h4', { class: 'jr-set-h', id: 'jr-perm-h', tabindex: '-1' }, S.permsTitle), el('p', { class: 'jr-lead', 'data-sim-scope': '', 'data-sim-fn': '' }, S.permsLead, ' ', ctx.feas('now'), ' ', X.modelNative, ' ', ctx.feas('feasible')), perms, liveG,
      el('h4', { class: 'jr-set-h', id: 'jr-acc-h', tabindex: '-1' }, X.grantsTitle), el('p', { class: 'jr-lead', 'data-sim-scope': '', 'data-sim-fn': '' }, X.grantsLead, ' ', X.modelNow, ' ', ctx.feas('now')), sess, ...pdocs)
    return true
  }
  if (page === 'opslag') {
    const O = T.instellingen.opslag
    const bar = (id, label, used, total, ratio, p) => el('div', { class: 'ds-progress-field' }, el('div', { class: 'ds-progress-label' }, el('span', { id }, badge(p, label)), el('span', { class: 'ds-num' }, O.used(used, total))),
      el('div', { class: 'ds-progress', role: 'progressbar', 'aria-labelledby': id, 'aria-valuemin': '0', 'aria-valuemax': '100', 'aria-valuenow': String(Math.round(ratio * 100)), style: `--ds-value: ${Math.round(ratio * 100)}%` }, el('span', { class: 'ds-progress-bar' })))
    pane.replaceChildren(h, el('p', { class: 'jr-lead' }, S.storageLead),
      bar('jr-use-dev', O.device, DATA.device?.used ?? '212 GB', DATA.device?.total ?? '512 GB', DATA.device?.ratio ?? 0.41, 'device'),
      el('div', { 'data-sim-scope': '' }, bar('jr-use-nas', DATA.nas.name, DATA.nas.used, DATA.nas.total, DATA.nas.ratio, 'nas'),
        el('p', { 'data-sim-fn': '' }, O.nasPaired, ' ', ctx.feas('simulated')), el('p', { 'data-sim-fn': '' }, O.nasEncrypted, ' ', ctx.feas('simulated'))),
      el('p', { class: 'jr-pop-note', 'data-sim-scope': '', 'data-sim-fn': '' }, S.pairLater, ' ', ctx.feas('feasible')),
      externalStorage(S, ctx, redraw, pane))
    return true
  }
  if (page === 'updates') {
    const U = T.instellingen.updates
    const state = upd.phase ?? ctx.sim.updates
    const check = el('button', { class: 'ds-btn', 'data-variant': 'secondary', type: 'button', disabled: state === 'checking', on: { click: () => {
      upd.phase = 'checking'; redraw(); ctx.live(S.checking)
      setTimeout(() => { upd.phase = null; redraw(); ctx.live(ctx.sim.updates === 'error' ? S.errTitle : U.current) }, 800)
    } } }, U.check)
    const body = state === 'checking' ? el('div', { 'aria-busy': 'true', 'data-sim-scope': '' }, el('p', { 'data-sim-fn': '' }, S.checking, ' ', ctx.feas('simulated')), el('div', { class: 'ds-progress', role: 'progressbar', 'data-state': 'indeterminate', 'aria-label': S.checking }, el('span', { class: 'ds-progress-bar' })))
      : state === 'error' ? el('div', { class: 'ds-banner', 'data-tone': 'error', role: 'status', 'data-sim-scope': '', 'data-sim-fn': '' }, icon('warn', { size: 18 }), el('p', {}, el('strong', {}, S.errTitle), ' ', S.errHint, ' ', ctx.feas('simulated')))
      : el('div', { class: 'ds-banner', 'data-tone': 'success', role: 'status', 'data-sim-scope': '', 'data-sim-fn': '' }, icon('check', { size: 18 }), el('p', {}, U.current, '. ', U.last(`vandaag ${DATA.now.clock}`), ' ', ctx.feas('simulated')))
    pane.replaceChildren(h, body, el('div', { class: 'jr-row' }, check))
    return true
  }
  return false
}
