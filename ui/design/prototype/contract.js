// Shared contract of the BaryonOS prototype: DOM helper, event bus, in-memory
// state and the file store. Simulation only: state lives in this page and is
// lost on reload. Apps receive a ctx from the shell (see shell/windows.js):
//   { open(appId, args), close(), setTitle(s), notify({title, body, actions}),
//     ask(request) -> Promise<'once'|'session'|'deny'>, files, grants, store, t }
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

// Files with versions and per-app grants. Every mutation emits 'files'.
const state = { files: clone(DATA.files), collections: clone(DATA.collections), trash: [] }
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

// Grants are stored on files; this view lists and revokes them per app.
export const grants = {
  forFile: (id) => files.get(id)?.grants ?? [],
  all: () => state.files.flatMap((f) => f.grants.map((g) => ({ ...g, file: f.id, name: f.name }))),
  has: (fileId, app) => grants.forFile(fileId).some((g) => g.app === app),
  grant(fileId, app, rights = 'readWrite', until = 'close') {
    const f = files.get(fileId)
    if (!grants.has(fileId, app)) f.grants.push({ app, rights, until })
    changed({ id: fileId, op: 'grant', app })
  },
  revoke(fileId, app) {
    const f = files.get(fileId)
    f.grants = f.grants.filter((g) => g.app !== app)
    changed({ id: fileId, op: 'revoke', app })
    bus.emit('grant:revoked', { file: fileId, app })
  },
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
