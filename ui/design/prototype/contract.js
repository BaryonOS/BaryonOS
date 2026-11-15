// Shared contract of the BaryonOS prototype: DOM helper, event bus, in-memory
// state and the file store. Simulation only: state lives in this page and is
// lost on reload. Apps receive a ctx from the shell (see shell/windows.js):
//   { open(appId, args), close(), setTitle(s), notify({title, body, actions}),
//     ask(request), files, grants, store, t }   ask: see "Permission prompts" below
import { T } from './strings.js'
import { DATA } from './data.js'
import { ICONS } from './icons.js'

export { T, DATA }

const listeners = new Map()
export const bus = {
  on(type, fn) {
    if (!listeners.has(type)) listeners.set(type, new Set())
    listeners.get(type).add(fn)
    return () => listeners.get(type).delete(fn)
  },
  emit(type, detail) {
    for (const fn of listeners.get(type) ?? []) fn(detail)
  },
}

// el('button', {class: 'x', on: {click}, 'aria-label': 'y'}, 'text', child, [more])
export function el(tag, attrs = {}, ...children) {
  const node = document.createElement(tag)
  for (const [k, v] of Object.entries(attrs)) {
    if (v == null || v === false) continue
    if (k === 'on') for (const [ev, fn] of Object.entries(v)) node.addEventListener(ev, fn)
    else if (k === 'style' && typeof v === 'object') Object.assign(node.style, v)
    else if (k === 'text') node.textContent = v
    else if (k in node && typeof v !== 'string') node[k] = v
    else node.setAttribute(k, v === true ? '' : v)
  }
  for (const c of children.flat(Infinity)) if (c != null && c !== false) node.append(c)
  return node
}

// icon('files', {size: 20, label: 'Bestanden'}): stroke icons on a 24-unit grid
// from icons.js; decorative unless a label is given.
export function icon(name, { size = 20, label } = {}) {
  const ns = 'http://www.w3.org/2000/svg'
  const svg = document.createElementNS(ns, 'svg')
  svg.setAttribute('viewBox', '0 0 24 24')
  svg.setAttribute('width', size)
  svg.setAttribute('height', size)
  svg.setAttribute('class', `icon icon-${name}`)
  if (label) { svg.setAttribute('role', 'img'); svg.setAttribute('aria-label', label) }
  else svg.setAttribute('aria-hidden', 'true')
  svg.innerHTML = ICONS[name] ?? ICONS.dot
  return svg
}

const clone = (v) => JSON.parse(JSON.stringify(v))
const prefersReduced = globalThis.matchMedia?.('(prefers-reduced-motion: reduce)').matches ?? false

// Shell-wide state. set() emits 'store' and 'store:<key>'; the shell applies
// theme/scale/motion to <html data-theme data-scale data-motion>.
export const store = {
  state: {
    theme: 'dark', // 'dark' | 'light' | 'contrast'
    scale: 1, // 1 | 2
    reducedMotion: prefersReduced,
    nasOnline: DATA.nas.online,
    focusMode: false,
    wifi: true,
    bluetooth: false,
    nightLight: false,
    notifications: clone(DATA.notifications),
    installed: new Set(),
    pinned: DATA.apps.filter((a) => a.pinned).map((a) => a.id),
    internetSearch: false,
    accessHistory: clone(DATA.accessHistory),
    activity: null, // privacy indicator override: {app, kind} or null ("Op dit apparaat")
  },
  get(k) { return this.state[k] },
  set(k, v) {
    this.state[k] = v
    bus.emit(`store:${k}`, v)
    bus.emit('store', { key: k, value: v })
  },
}

// Files with versions. Every mutation emits 'files'.
// grants live in their own store (see Permission prompts); files keep no stale copy
const state = { files: clone(DATA.files).map(({ grants: _, ...f }) => f), collections: clone(DATA.collections), trash: [] }
const changed = (detail) => bus.emit('files', detail)

export const files = {
  collections: () => state.collections,
  trash: () => state.trash,
  get: (id) => state.files.find((f) => f.id === id) ?? state.trash.find((f) => f.id === id),
  // filter: {place, collection, favourite, recent, query}
  list(filter = {}) {
    let out = state.files
    if (filter.place) out = out.filter((f) => f.place === filter.place)
    if (filter.collection) out = out.filter((f) => f.collection === filter.collection)
    if (filter.favourite) out = out.filter((f) => f.favourite)
    if (filter.recent) out = out.filter((f) => f.recent > 0).sort((a, b) => a.recent - b.recent)
    if (filter.storage) out = out.filter((f) => f.storage !== 'local')
    if (filter.query) {
      const q = filter.query.toLowerCase()
      out = out.filter((f) => f.name.toLowerCase().includes(q))
    }
    return out
  },
  // Sync label for a file, honouring the simulated NAS state.
  sync(f) {
    if (f.storage === 'local') return 'ok'
    if (!store.get('nasOnline')) return f.storage === 'nas' ? 'unreachable' : 'waiting'
    return f.sync
  },
  rename(id, name) {
    const f = files.get(id)
    const from = f.name
    f.name = name
    changed({ id, op: 'rename', from })
    return from
  },
  move(id, collection) { files.get(id).collection = collection; changed({ id, op: 'move' }) },
  favourite(id, on) { files.get(id).favourite = on; changed({ id, op: 'favourite' }) },
  remove(id) {
    const i = state.files.findIndex((f) => f.id === id)
    if (i < 0) return
    state.trash.push(...state.files.splice(i, 1))
    changed({ id, op: 'remove' })
  },
  unremove(id) {
    const i = state.trash.findIndex((f) => f.id === id)
    if (i < 0) return
    state.files.push(...state.trash.splice(i, 1))
    changed({ id, op: 'unremove' })
  },
  latest: (id) => files.get(id).versions.at(-1),
  // Save a new version (Notities autosave). Returns its number.
  save(id, body) {
    const f = files.get(id)
    const n = f.versions.at(-1).n + 1
    f.versions.push({ n, when: `vandaag ${DATA.now.clock}`, body })
    f.changed = `Vandaag ${DATA.now.clock}`
    changed({ id, op: 'save', n })
    return n
  },
  // Restoring never loses data: the chosen body becomes a new latest version.
  restore(id, n) {
    const f = files.get(id)
    const old = f.versions.find((v) => v.n === n)
    const newN = files.save(id, old.body)
    f.versions.at(-1).restoredFrom = n
    changed({ id, op: 'restore', from: n, n: newN })
    return newN
  },
  undoRestore(id) {
    const f = files.get(id)
    if (f.versions.at(-1)?.restoredFrom == null) return
    f.versions.pop()
    changed({ id, op: 'undo-restore' })
  },
}

// Permission prompts (owner decision of 14 November 2026; docs/design/DESIGN-SYSTEM.md §11).
// Two kinds, three equal choices each:
//   'ongoing' (access that lasts while the app runs)  -> 'untilClose' | 'always' | 'deny'
//   'action'  (one discrete operation)                -> 'once'       | 'always' | 'deny'
// A request is { kind, app, resource, right, scope, to? }. A standing grant is keyed on exactly
// app + resource + right + scope and carries until 'close' | 'always'. The scope includes the destination
// of a right that sends data off the device: 'send-copy' requires `to` (for example 'tuinplanner.example'),
// so a grant for one destination never covers another; for 'connect' the resource is the destination itself
// (destination() below; open item O-G9). 'close' ends when the app closes (so also at logout and restart);
// 'once' is consumed by its single action and never stored; 'deny' and Escape store nothing. A request whose
// exact key holds a standing grant resolves without a prompt to the stored duration ('untilClose' or
// 'always', whatever the request's kind; callers treat every result except 'deny' as permission); any
// other key (other resource, broader right, wider scope, other destination) prompts again, and a grant
// never broadens (shortcut: exact key match, no "readWrite covers read" ordering; open item O-G3).
// SIMULATION ONLY: ask() suppresses a repeated prompt; it is not the policy service, which must still
// check capability, object scope and authorisation on every request. Grants live in this page and are
// lost on reload; persistence of 'always' across a system restart is NOT simulated here (future
// implementation requirement; native persistent grants need a project ADR first).
export const CHOICES = { ongoing: ['untilClose', 'always', 'deny'], action: ['once', 'always', 'deny'] }
const UNTIL = { untilClose: 'close', always: 'always' }
const KEY = ['app', 'resource', 'right', 'scope']
const SENDS = new Set(['send-copy']) // rights that move data off the device; their scope names a destination
const same = (a, b) => KEY.every((k) => a[k] === b[k]) && (SENDS.has(a.right) ? a.to === b.to : true)
// Where a right can carry data beyond this device: the named destination, the domain itself for 'connect', else null.
export const destination = (g) => (SENDS.has(g.right) ? g.to : g.right === 'connect' ? g.resource : null)
// data.js seeds per-file grants as {app, rights, until}; normalise them to the four-field key.
const standing = DATA.files.flatMap((f) => (f.grants ?? []).map((g) => ({ app: g.app, resource: f.id, right: g.rights, scope: 'document', until: g.until })))
const grantsChanged = (detail) => bus.emit('grants', detail)

export const grants = {
  all: () => standing.map((g) => ({ ...g })),
  forFile: (id) => grants.all().filter((g) => g.resource === id),
  covering: (request) => standing.find((g) => same(g, request)) ?? null,
  revoke(key) {
    const i = standing.findIndex((g) => same(g, key))
    if (i < 0) return null
    const [g] = standing.splice(i, 1)
    grantsChanged({ op: 'revoke', grant: { ...g } })
    bus.emit('grant:revoked', { ...g })
    return g
  },
  // The app closed (also logout and restart): its 'close' grants end; 'always' grants stay.
  appClosed(app) {
    for (const g of standing.filter((x) => x.app === app && x.until === 'close')) {
      standing.splice(standing.indexOf(g), 1)
      grantsChanged({ op: 'expire', grant: { ...g } }) // an expiry, not a user revocation
    }
  },
}

// The shell (or the gallery) supplies the prompt UI: present(request) -> Promise<choice>. Without one,
// every prompt answers 'deny'. ask(request) -> Promise<choice>; 'once' tells the caller to run its action now.
let present = () => Promise.resolve('deny')
export const setPrompt = (fn) => { present = fn }

export async function ask(request) {
  const allowed = CHOICES[request.kind]
  if (!allowed) throw new TypeError(`ask: kind must be 'ongoing' or 'action', got ${request.kind}`)
  for (const k of KEY) if (!request[k]) throw new TypeError(`ask: request.${k} is required`)
  if (SENDS.has(request.right) && !request.to) throw new TypeError(`ask: right '${request.right}' needs request.to (destination)`)
  const held = grants.covering(request)
  if (held) return held.until === 'close' ? 'untilClose' : 'always' // covered: no prompt
  let choice = await present(request)
  if (!allowed.includes(choice)) choice = 'deny' // Escape, closing or an unknown answer = Weigeren
  if (UNTIL[choice]) {
    const g = { ...Object.fromEntries(KEY.map((k) => [k, request[k]])), ...(SENDS.has(request.right) && { to: request.to }), until: UNTIL[choice] }
    standing.push(g)
    grantsChanged({ op: 'grant', grant: { ...g } })
  }
  return choice
}

// Time helpers used by every module so motion follows the user's setting.
export const motionMs = (ms) => (store.get('reducedMotion') ? 0 : ms)
export const wait = (ms) => new Promise((r) => setTimeout(r, ms))

// App registry: apps/*.js default-export { id, title, icon, minSize, render(root, ctx) }.
const registry = new Map()
export const apps = {
  register: (def) => registry.set(def.id, def),
  get: (id) => registry.get(id),
  all: () => [...registry.values()],
}
