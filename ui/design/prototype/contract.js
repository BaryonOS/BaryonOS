// Shared contract of the BaryonOS prototype: DOM helper, event bus, in-memory
// state and the file store. Simulation only: state lives in this page and is
// lost on reload. Apps receive a ctx from the shell (see shell/windows.js):
//   { open(appId, args), close(), setTitle(s), notify({title, body, actions}),
//     ask(request, {gesture}), files, grants, store, t }   ask: see "Permission prompts" below
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
// decisions live in the permission stores (see Permission prompts); files keep no stale copy
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

// Permission prompts: owner model of 15 November 2026 (Toestaan / Weigeren + "Keuze onthouden"), readings R2 and R3,
// owner decisions of 18-19 November 2026 (docs/design/DESIGN-SYSTEM.md §11.1). PROTOTYPE ONLY: every store below lives
// in this page and is lost on reload; BaryonOS does not enforce remembered or temporary decisions (an ADR comes first,
// docs/verification/uxc-run2-20261119/PROPOSED-ADR.md), and a suppressed prompt never replaces the per-request policy check.
//
// A request: { kind: 'ongoing' | 'action', app, right, data, destination, scope, irreversible?, context?, reason? }.
// The key is exactly five fields: app, right, data (one exact item, never a class), destination (null when nothing
// leaves the device) and scope. Changing any one field is a new question.
// present(request, view) -> Promise<{ choice: 'allow' | 'deny' | anything else = cancel, remember, confirmed }>,
// view = { compact, rememberAllow, confirmAllow, reason }.
// ask(request, { gesture }) -> { answer: 'allow' | 'deny', via, remembered, until }
//   via: 'prompt' | 'cancel' | 'remembered' | 'covered' | 'temporary-deny' | 'error' (the prompt could not be shown)
export const KEY = ['app', 'right', 'data', 'destination', 'scope']
export const RIGHTS = new Set(['read', 'readWrite', 'save-copy', 'connect', 'send-copy', 'read-connect', 'camera', 'microphone', 'location'])
const NEEDS_DEST = new Set(['connect', 'send-copy', 'read-connect']) // rights that reach a named place beyond the device
const COMBINED = new Set(['read-connect']) // a document the app holds plus a route to the internet (LOCAL-PRIVACY-CONTRACT §4)
const LOW_RISK = new Set(['read', 'save-copy']) // the only rights a compact repeat prompt may serve (MF-5 risk rule)
const BOUNDED_SCOPES = new Set(['document', 'item'])
export const keyOf = (r) => JSON.stringify(KEY.map((k) => r[k] ?? null))
const pick = (r) => Object.fromEntries(KEY.map((k) => [k, r[k] ?? null]))
// OQ-1: an irreversible single action that sends data off the device can never be remembered as Toestaan.
export const isExternalSend = (r) => r.kind === 'action' && r.right === 'send-copy' && !!r.destination
export const rememberAllowOffered = (r) => !(isExternalSend(r) && r.irreversible !== false)
// OQ-1: a remembered Toestaan for a combined document-and-internet right needs a named destination, one exact bounded item
// and a bounded scope, and an extra confirmation; without all of them nothing is stored.
export const needsConfirm = (r) => COMBINED.has(r.right)
const combinedBounded = (r) => !!r.destination && typeof r.data === 'string' && BOUNDED_SCOPES.has(r.scope)
// MF-5 risk rule: never for an external send, an irreversible action, any destination, camera, microphone, a combined right
// or any right outside LOW_RISK.
export const lowRisk = (r) => LOW_RISK.has(r.right) && !r.destination && !r.irreversible && !COMBINED.has(r.right)
// OQ-6: an optional app reason is shown only when it is a string of 1-120 characters without control, format (bidi) or
// line-break characters; anything else is not shown at all (never truncated). Markup characters are plain text here and
// are rendered with textContent, so "<b>" appears literally.
export function appReason(s) {
  if (typeof s !== 'string') return null
  const n = [...s].length
  if (n < 1 || n > 120 || /[\p{Cc}\p{Cf}\p{Zl}\p{Zp}]/u.test(s)) return null
  return s
}

export function validate(r) {
  if (r?.kind !== 'ongoing' && r?.kind !== 'action') throw new TypeError(`ask: kind must be 'ongoing' or 'action', got ${r?.kind}`)
  for (const k of ['app', 'right', 'data', 'scope']) if (typeof r[k] !== 'string' || !r[k]) throw new TypeError(`ask: request.${k} is required`)
  if (!RIGHTS.has(r.right)) throw new TypeError(`ask: unknown right '${r.right}' (camera and microphone are separate rights)`)
  if (/[*?]/.test(r.data)) throw new TypeError('ask: data names one exact item, never a class or pattern')
  if (NEEDS_DEST.has(r.right) && !r.destination) throw new TypeError(`ask: right '${r.right}' needs request.destination`)
  if (!NEEDS_DEST.has(r.right) && r.destination) throw new TypeError(`ask: right '${r.right}' has no destination`)
}

export function createPermissions({ emit = () => {} } = {}) {
  let present = () => Promise.resolve({ choice: 'cancel' }) // no prompt UI: every request fails closed
  const remembered = new Map() // key -> { ...fields, kind, irreversible, decision: 'allow' | 'deny' }  (prototype "lasting")
  const temporary = new Map() // key -> fields; ongoing grants until the logical app session ends (O-G12, OQ-5)
  const tempDenied = new Map() // key -> { ...fields, retries }; volatile, until the app closes (R2)
  const eligible = new Map() // key -> { context }; volatile record of a temporary allow, for the compact form only (MF-5)
  const pending = new Map() // key -> Promise; identical concurrent requests share one prompt (no duplicate grant)
  let tail = null // the last prompt still open or waiting; a distinct request waits its turn and never cancels it (OQ-4)

  const view = (r, k) => ({
    compact: lowRisk(r) && eligible.has(k) && eligible.get(k).context === (r.context ?? null),
    rememberAllow: rememberAllowOffered(r),
    confirmAllow: needsConfirm(r),
    reason: appReason(r.reason),
  })

  async function decide(r, k) {
    const v = view(r, k)
    let res
    // a prompt that cannot be shown (no copy, a render error) fails closed for this request only: nothing granted, nothing
    // stored, no temporary denial and no R2 suppression, because the user never saw a question
    try { res = await present({ ...r }, v) } catch {
      emit('permission', { op: 'error', key: k })
      return { answer: 'deny', via: 'error', remembered: false, until: 'action' }
    }
    const choice = res?.choice, remember = res?.remember === true
    // MF-5: only an allow that stays temporary keeps the compact-form record; any later refusal, cancellation or remembered
    // decision is a changed circumstance, so the next identical request gets the full prompt
    if (choice !== 'allow') eligible.delete(k)
    if (choice !== 'allow' && choice !== 'deny') { // Escape, closing or an unknown answer (owner decision 1)
      tempDenied.set(k, { ...pick(r), retries: 0 })
      emit('permission', { op: 'cancel', key: k })
      return { answer: 'deny', via: 'cancel', remembered: false, until: 'app-session' }
    }
    if (choice === 'deny') {
      if (remember) {
        tempDenied.delete(k)
        remembered.set(k, { ...pick(r), kind: r.kind, irreversible: !!r.irreversible, decision: 'deny' })
        emit('permission', { op: 'remember', key: k, decision: 'deny' })
        return { answer: 'deny', via: 'prompt', remembered: true, until: 'remembered' }
      }
      tempDenied.set(k, { ...pick(r), retries: 0 })
      emit('permission', { op: 'temporary-deny', key: k })
      return { answer: 'deny', via: 'prompt', remembered: false, until: 'app-session' }
    }
    tempDenied.delete(k)
    const store = remember && v.rememberAllow && (!v.confirmAllow || (res.confirmed === true && combinedBounded(r)))
    if (store) {
      eligible.delete(k)
      remembered.set(k, { ...pick(r), kind: r.kind, irreversible: !!r.irreversible, decision: 'allow' })
      emit('permission', { op: 'remember', key: k, decision: 'allow' })
      return { answer: 'allow', via: 'prompt', remembered: true, until: 'remembered' }
    }
    eligible.set(k, { context: r.context ?? null })
    if (r.kind === 'ongoing') {
      temporary.set(k, pick(r))
      emit('permission', { op: 'temporary-allow', key: k })
      return { answer: 'allow', via: 'prompt', remembered: false, until: 'app-session' }
    }
    emit('permission', { op: 'action-allow', key: k })
    return { answer: 'allow', via: 'prompt', remembered: false, until: 'action' }
  }

  async function ask(r, { gesture = false } = {}) {
    validate(r)
    const k = keyOf(r)
    const held = remembered.get(k)
    // kind and irreversible are not key fields (OQ-4), so a stored Toestaan answers only a request for which remembering
    // Toestaan is offered (OQ-1); otherwise, e.g. an irreversible single external send, the request is asked normally.
    // A remembered Weigeren always answers: it can only narrow access.
    if (held && (held.decision === 'deny' || rememberAllowOffered(r))) return { answer: held.decision, via: 'remembered', remembered: true, until: 'remembered' } // never re-prompts
    if (temporary.has(k)) return { answer: 'allow', via: 'covered', remembered: false, until: 'app-session' }
    const td = tempDenied.get(k)
    if (td && !gesture) { // R2: automatic identical re-request, refused quietly; only a new deliberate gesture asks again
      td.retries++
      emit('permission', { op: 'quiet-deny', key: k, retries: td.retries, request: pick(r) })
      return { answer: 'deny', via: 'temporary-deny', remembered: false, until: 'app-session' }
    }
    if (pending.has(k)) return pending.get(k)
    const p = (tail ? tail.then(() => decide(r, k)) : decide(r, k)).finally(() => pending.delete(k)) // decide never rejects
    tail = p; p.then(() => { if (tail === p) tail = null })
    pending.set(k, p)
    return p
  }

  const endSession = (match, op) => {
    for (const m of [temporary, tempDenied]) for (const [k, g] of m) if (match(g)) m.delete(k)
    if (op !== 'app-closed') eligible.clear()
    emit('permission', { op })
  }
  const grants = {
    remembered: () => [...remembered.values()].map((g) => ({ ...g })),
    temporary: () => [...temporary.values()].map((g) => ({ ...g })),
    temporaryDenials: () => [...tempDenied.values()].map((g) => ({ ...g })),
    eligible: (r) => view(r, keyOf(r)).compact,
    // revocation removes exactly the matching five-field key and nothing else
    revoke(key) {
      const k = keyOf(key), g = remembered.get(k)
      if (!g) return null
      remembered.delete(k)
      eligible.delete(k)
      emit('permission', { op: 'revoke', key: k })
      return { ...g }
    },
    // change a remembered decision in Appmachtigingen; a change to Toestaan obeys the same OQ-1 rules as the prompt, so an
    // irreversible external send or a combined right (which needs the prompt's confirmation) cannot be switched to Toestaan
    change(key, decision) {
      const k = keyOf(key), g = remembered.get(k)
      if (!g || (decision !== 'allow' && decision !== 'deny')) return false
      if (decision === 'allow' && (!rememberAllowOffered(g) || !rememberAllowOffered({ ...g, ...key }) || needsConfirm(g))) return false
      g.decision = decision
      eligible.delete(k)
      emit('permission', { op: 'change', key: k, decision })
      return true
    },
    canChangeToAllow: (g) => rememberAllowOffered(g) && !needsConfirm(g),
    // owner decision of 23 Nov 2026 (file close / user revoke): end exactly one temporary grant while its app keeps running.
    // Removes only that five-field key from the temporary grants and its compact-form record (a revocation is a changed
    // circumstance, MF-5); remembered decisions, temporary denials (R2) and every other key are left as they are.
    endTemporary(key) {
      const k = keyOf(key), g = temporary.get(k)
      if (!g) return null
      temporary.delete(k)
      eligible.delete(k)
      emit('permission', { op: 'end-temporary', key: k })
      return { ...g }
    },
    // OQ-5 end events. A temporary grant ends with the logical app session (explicit termination), logout or a user-session
    // switch; hiding or minimising a window ends nothing, and there is no timeout. Temporary denials end with the app (R2).
    appClosed: (app) => endSession((g) => g.app === app, 'app-closed'),
    logout: () => endSession(() => true, 'logout'),
    sessionSwitch: () => endSession(() => true, 'session-switch'),
    windowHidden: () => {},
    seedTemporary: (r) => { validate(r); temporary.set(keyOf(r), pick(r)) },
    seedRemembered: (r, decision) => { validate(r); remembered.set(keyOf(r), { ...pick(r), kind: r.kind, irreversible: !!r.irreversible, decision }) },
  }
  return { ask, grants, setPrompt: (fn) => { present = fn } }
}

// the page's instance; data.js seeds per-file grants {app, rights, until: 'close'} as temporary grants of the open app
export const permissions = createPermissions({ emit: (type, d) => bus.emit(type, d) })
for (const f of DATA.files) for (const g of f.grants ?? []) {
  const r = { kind: 'ongoing', app: g.app, right: g.rights, data: f.id, destination: null, scope: 'document' }
  if (g.until === 'close') permissions.grants.seedTemporary(r); else permissions.grants.seedRemembered(r, 'allow')
}
export const grants = permissions.grants
export const ask = (r, o) => permissions.ask(r, o)
export const setPrompt = (fn) => permissions.setPrompt(fn)

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
