// UX-E connected prototype journey, bootstrap (slice A). Browser simulation of BaryonOS:
// design evidence, not a working OS. Router, theme and tier-2 axes, global keys, confirm
// dialog and the ?test=1 hook. Station logic lives in journey/*.js.
import { T, DATA, el, icon, store, bus, grants } from './contract.js'
import { STATIONS, byId, parse, go as route, serialize, SIM_DEFAULTS, isBuilt } from './journey/state.js'
import { createThrottle } from './journey/throttle.js'
import { match, swallowRepeat } from './journey/keys.js'
import { initBand } from './journey/band.js'
import { initSession } from './journey/session.js'
import { initShell } from './journey/shell.js'

const J = T.journey
const root = document.documentElement
const $ = (s) => document.querySelector(s)
const mq = (q) => globalThis.matchMedia?.(q).matches ?? false
const KERN = { dark: 'light', light: 'dark', contrast: 'contrast' }
const FEAS_ICON = { now: 'check', feasible: 'klok', research: 'fwd', simulated: 'dot' }

const ctx = {
  J, phase: 'locked', signedIn: false, station: 'aanmelden', seen: new Set(), pending: null,
  sim: { ...SIM_DEFAULTS }, axes: {}, scale: 1,
  kern: () => KERN[root.dataset.theme] ?? 'light',
  feasText: (k) => (k === 'simulated' ? J.feas.simulated : T.trusted.feasibility[k]),
  // Visible feasibility chip (E4); neutral colours only, icon plus text.
  feas: (k) => el('span', { class: 'jr-feas', 'data-feas': k }, icon(FEAS_ICON[k], { size: 14 }), ctx.feasText(k)),
  live(msg, region = '#jr-live') {
    const r = $(region)
    r.textContent = ''
    setTimeout(() => { r.textContent = msg }, 40)
  },
}

// Theme and tier-2 axes: independent root attributes; '' = follow the device.
const deviceTheme = () => (mq('(prefers-contrast: more)') ? 'contrast' : mq('(prefers-color-scheme: light)') ? 'light' : 'dark')
function applyAxes() {
  for (const axis of Object.keys(J.a11y.axes)) {
    if (axis === 'theme') continue
    const v = ctx.axes[axis]
    if (v) root.setAttribute(`data-${axis}`, v); else root.removeAttribute(`data-${axis}`)
  }
  root.dataset.theme = ctx.axes.theme || deviceTheme()
  if (ctx.scale === 2) root.dataset.scale = '2'; else root.removeAttribute('data-scale')
  store.set('theme', root.dataset.theme)
  store.set('reducedMotion', root.dataset.motion === 'reduce' || (root.dataset.motion !== 'full' && mq('(prefers-reduced-motion: reduce)')))
  for (const k of document.querySelectorAll('[data-kern]')) k.dataset.theme = ctx.kern() // theme changes apply at once, never cross-faded
  bus.emit('jr:axes')
}
const queue = createThrottle(applyAxes)
ctx.setAxis = (axis, v) => { ctx.axes[axis] = v; queue() }
ctx.setScale = (n) => { ctx.scale = n; queue() }
ctx.resetAxes = () => { const before = { ...ctx.axes }; ctx.axes = {}; queue(); return () => { ctx.axes = before; queue() } }
ctx.setSim = (group, v) => { ctx.sim[group] = v; bus.emit('jr:sim', group) }
// The NAS group acts out a network event: the shell reads it through the store (Orbit status, Snelle instellingen, Bureaublad).
ctx.sim.nas = store.get('nasOnline') === false ? 'offline' : 'online'
bus.on('jr:sim', (g) => { if (g === 'nas') store.set('nasOnline', ctx.sim.nas === 'online') })

// Confirm dialog: three equal answers, focus on Annuleren, Escape = Annuleren, focus returns to the opener.
ctx.confirm = (text, heading = J.r2.unsavedHeading) => new Promise((resolve) => {
  const dlg = $('#jr-confirm')
  const opener = document.activeElement
  const done = (v) => { dlg.close(); opener?.isConnected ? opener.focus() : ctx.shell.focusFallback(); resolve(v) }
  const b = (v, label) => el('button', { class: 'ds-btn', 'data-variant': 'secondary', type: 'button', on: { click: () => done(v) } }, label)
  const cancel = b('cancel', J.session.cancel)
  dlg.replaceChildren(el('h2', { class: 'ds-dialog-title', id: 'jr-confirm-t' }, heading),
    el('div', { class: 'ds-dialog-body' }, el('p', { class: 'ds-dialog-text', id: 'jr-confirm-d' }, text)),
    el('div', { class: 'ds-dialog-actions' }, b('save', J.session.save), b('discard', J.session.discard), cancel))
  dlg.oncancel = (e) => { e.preventDefault(); done('cancel') }
  dlg.showModal()
  cancel.focus()
})

// Router: go() only opens or focuses (state.js); Back and Forward call it as well.
ctx.go = (id, { push = true } = {}) => {
  const r = route(id, { signedIn: ctx.signedIn && ctx.phase === 'desktop', windows: ctx.shell.count() }) // locked: target becomes pending
  if (r.later) { ctx.live(`${J.stations[r.later].title}: ${J.tour.laterWhy}`); return }
  if (r.pending) ctx.pending = r.pending
  for (const [fx, arg] of r.effects) {
    if (fx === 'lock') ctx.session.lock()
    else if (fx === 'showLock') { if (ctx.phase === 'locked' || ctx.phase === 'desktop') ctx.session.show() }
    else if (fx === 'closePopovers') ctx.shell.closePopovers()
    else if (fx === 'open') ctx.shell.open(arg)
    else if (fx === 'openSearch') ctx.shell.openSearch()
  }
  ctx.station = r.station
  ctx.seen.add(r.station)
  const h = serialize(r.station)
  if (location.hash !== h) history[push ? 'pushState' : 'replaceState'](null, '', h)
  document.title = `${J.stations[r.station].title} · BaryonOS-prototype`
  // a lock caused by Back or a route says so in the same message: the station line would otherwise replace it (repair 2)
  const locked = r.effects.some(([fx]) => fx === 'lock')
  ctx.live(`${J.tour.station(byId(r.station).n, J.stations[r.station].title)}${locked ? ` ${J.session.locked}` : ''}`)
  bus.emit('jr:station', r.station)
}
addEventListener('popstate', () => ctx.go(location.hash.replace(/^#\/?/, ''), { push: false }))

ctx.signedInDone = () => {
  const first = !ctx.signedIn
  ctx.signedIn = true
  ctx.phase = 'desktop'
  ctx.session.hide()
  if (first) ctx.band.toggle('tour', false) // the Rondleiding stays one click away in the band; the shell gets the room
  ctx.shell.show()
  const next = ctx.pending || (first ? 'bureaublad' : ctx.station === 'aanmelden' ? 'bureaublad' : ctx.station)
  if (ctx.pending) ctx.live(J.tour.continueTo(J.stations[ctx.pending].title))
  ctx.pending = null
  ctx.go(next)
  bus.emit('jr:session')
}
// Sign-out never drops an unsaved draft silently: Opslaan / Niet opslaan / Annuleren first (the system screen asks the same).
ctx.signOut = async ({ ask = true } = {}) => {
  if (ask && ctx.shell.dirty()) {
    const v = await ctx.confirm(J.session.signOutAsk)
    if (v === 'cancel') return false
    if (v === 'save') ctx.shell.saveAll(); else ctx.shell.discardAll()
  }
  ctx.shell.closeAll(); ctx.shell.discardAll(); grants.logout() // logout ends the app sessions and their temporary grants (contract OQ-5)
  ctx.signedIn = false; ctx.session.show(); ctx.go('aanmelden'); bus.emit('jr:session')
  return true
}

// F6 regions: active window (or desktop), Orbit, prototype band. No trap: Tab also leaves every region.
function region(dir) {
  const regs = [ctx.shell.regionTarget(), $('#jr-desk .ds-orbit button'), $('.jr-band button')].filter((x) => x && !x.closest('[inert],[hidden]'))
  if (!regs.length) return
  const i = regs.findIndex((r) => r.closest('.ds-window, .ds-orbit, .jr-band')?.contains(document.activeElement))
  regs[(i + dir + regs.length) % regs.length].focus()
}

// B8 guard first (window, capture phase): no listener below ever sees a swallowed repeat.
addEventListener('keydown', (e) => { if (swallowRepeat(e)) { e.preventDefault(); e.stopImmediatePropagation() } }, true)

addEventListener('keydown', (e) => {
  const inText = !!e.target.closest?.('input:not([type=radio]):not([type=checkbox]), textarea, [contenteditable]')
  const cmd = match(e, { inText })
  if (!cmd) return
  if (cmd === 'help') { e.preventDefault(); ctx.band.toggle('keys'); return }
  if (cmd === 'regionNext' || cmd === 'regionPrev') { e.preventDefault(); region(cmd === 'regionNext' ? 1 : -1); return }
  if (!ctx.signedIn || ctx.phase !== 'desktop' || $('#jr-confirm').open) return
  e.preventDefault()
  if (cmd === 'system') ctx.session.system()
  else if (cmd === 'lock') ctx.go('aanmelden')
  else ctx.shell.command(cmd, e)
})

ctx.band = initBand(ctx)
ctx.session = initSession(ctx)
ctx.shell = initShell(ctx)
const qTheme = new URLSearchParams(location.search).get('theme') // screenshots: ?theme=dark|light|contrast
if (['dark', 'light', 'contrast'].includes(qTheme)) ctx.axes.theme = qTheme
applyAxes()
for (const q of ['(prefers-color-scheme: light)', '(prefers-contrast: more)', '(prefers-reduced-motion: reduce)']) globalThis.matchMedia?.(q).addEventListener?.('change', applyAxes)

const start = parse(location.hash, { signedIn: false })
ctx.pending = start.pending
ctx.go('aanmelden', { push: false })

// ?test=1: test-only hook for sweep.html (E10). Not part of the simulated product.
if (new URLSearchParams(location.search).has('test')) {
  const tick = () => new Promise((r) => setTimeout(r, 60))
  window.__journey = {
    stations: () => STATIONS.filter(isBuilt).map(({ n, id, states, feas }) => ({ n, id, states, feas })),
    go: ctx.go,
    setSim: ctx.setSim,
    async signIn() { if (!ctx.signedIn) { ctx.session.hide(); ctx.signedInDone() } else if (ctx.phase !== 'desktop') ctx.signedInDone(); await tick() },
    openApp: (id) => ctx.shell.open(id),
    setRoot(attrs) { ctx.axes = {}; ctx.scale = 1; for (const [k, v] of Object.entries(attrs)) { if (k === 'scale') ctx.scale = +v; else ctx.axes[k] = v } applyAxes() },
    async show(station, state) {
      ctx.sim = { ...SIM_DEFAULTS }
      store.set('nasOnline', true)
      ctx.shell.closePopovers()
      if (station === 'aanmelden') {
        if (state === 'success') return this.signIn()
        if (ctx.signedIn) await ctx.signOut({ ask: false })
        ctx.session.show()
        if (state === 'empty') ctx.session.oobe(1)
        else if (state === 'loading') ctx.session.starting(true)
        else { ctx.setSim('signin', 'incorrect'); ctx.session.chooser(); ctx.session.submitForTest() }
        return tick()
      }
      await this.signIn()
      if (station === 'bureaublad') { ctx.setSim('desk', state === 'empty' ? 'empty' : 'filled'); ctx.go('bureaublad') }
      if (station === 'vensters') {
        ctx.shell.closeAll()
        if (state === 'empty') { ctx.shell.command('switchNext', {}); return tick() }
        // test-only: restore the page-load grant (data.js seed) so the success cell measures the Tuinplan body; the product never re-seeds
        if (state === 'success') grants.seedTemporary({ kind: 'ongoing', app: 'notities', right: 'readWrite', data: 'tuinplan', destination: null, scope: 'document' })
        ctx.setSim('launch', state === 'loading' ? 'slow' : state === 'error' ? 'fail' : 'normal')
        ctx.go('vensters')
        if (state === 'success' || state === 'error') await new Promise((r) => setTimeout(r, 700)) // the launch settles after 600 ms
      }
      if (station === 'zoeken') {
        ctx.setSim('search', state === 'loading' ? 'updating' : state === 'error' ? 'unavailable' : 'ready')
        ctx.go('zoeken')
        ctx.shell.searchFor(state === 'empty' ? '' : state === 'none' ? 'qqxz' : 'tuin')
      }
      return tick()
    },
    extras: () => ['menu', 'quick', 'quick-offline', 'notes', 'system', 'recovery', 'instellingen', 'bestanden', 'nogrant', 'panels', 'search-app', 'search-setting'],
    async extra(name) {
      await this.signIn(); ctx.shell.closePopovers()
      if (name === 'quick-offline') { ctx.setSim('nas', 'offline'); ctx.shell.pop('quick') }
      if (['menu', 'quick', 'notes'].includes(name)) ctx.shell.pop(name)
      if (name === 'system') ctx.session.system()
      if (name === 'recovery') { ctx.phase = 'locked'; ctx.session.recovery() }
      if (name === 'instellingen' || name === 'bestanden') { ctx.shell.open(name); await new Promise((r) => setTimeout(r, 700)) }
      if (name === 'nogrant') { ctx.shell.open('notities'); ctx.shell.closeAll(); ctx.shell.open('notities'); await new Promise((r) => setTimeout(r, 700)) } // fail-closed body after the grant ended
      // the preview column for non-file results (an app, a setting): the success cell only highlights a file
      if (name === 'search-app' || name === 'search-setting') { ctx.go('zoeken'); ctx.shell.searchFor(name === 'search-app' ? 'not' : 'thema') }
      if (name === 'panels') for (const p of ['tour', 'status', 'keys', 'sim']) ctx.band.toggle(p, true)
      return tick()
    },
    async reset() { if (ctx.sim.nas !== 'online') ctx.setSim('nas', 'online'); for (const p of ['tour', 'status', 'keys', 'sim']) ctx.band.toggle(p, false); if (ctx.phase !== 'desktop') { ctx.session.hide(); ctx.phase = 'desktop' } ctx.shell.closePopovers(false); return tick() },
    ready: () => Promise.resolve(true),
    errors: () => window.__errs ?? [],
    DATA,
  }
}
